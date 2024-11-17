use rust_decimal::Decimal;
use super::orderbook::OrderBook;
use super::types::{Order, Side, OrderType, OrderStatus};
use super::trade_history::{TxnHistory, Trade};
use std::collections::VecDeque;

pub struct MatchingEngine {
    order_book: OrderBook,
    trade_history: TxnHistory,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            order_book: OrderBook::new(),
	    trade_history: TxnHistory::new()
        }
    }

    /// ------------------------
    pub fn place_order(&mut self, order: Order) -> Order {
        match order.order_type {
            OrderType::Limit  => self.place_limit_order(order),
            OrderType::Market => self.place_market_order(order),
        }
    }

    /// ------------------------
    fn place_limit_order(&mut self, order: Order) -> Order {
        let matched_order = self.match_order(&order);
        if matched_order.remaining_quantity > Decimal::ZERO {
            self.order_book.place_order(matched_order.clone());
        } else {
            self.order_book.orders.insert(matched_order.id, matched_order.clone());
        }
        matched_order
    }

    /// ------------------------
    fn place_market_order(&mut self, mut order: Order) -> Order {
        order = self.match_order(&order);
        self.order_book.orders.insert(order.id, order.clone());
        order
    }

    /// --------
    /// core matcher against the book
    /// --------

    fn get_best_matching_price(&self, side: Side) -> Option<Decimal> {
        match side {
            Side::Bid => self.order_book.asks.keys().next().cloned(),
            Side::Ask => self.order_book.bids.keys().next_back().cloned(),
        }
    }

    fn should_match(&self, order: &Order, price: Decimal) -> bool {
        match order.order_type {
            OrderType::Market => true,
            OrderType::Limit => match order.side {
                Side::Ask => price >= order.price,
                Side::Bid => price <= order.price,
            },
        }
    }

    fn match_at_price_level(matched_order: &mut Order, price: Decimal, resting_orders: &mut VecDeque<Order>) -> VecDeque<Trade> {
        let mut trades = VecDeque::new();

	// actually fill @ price level
        while let Some(resting_order) = resting_orders.front_mut() {
            if matched_order.remaining_quantity == Decimal::ZERO {
                break;
            }

            let fill_quantity = matched_order.remaining_quantity.min(resting_order.remaining_quantity);

	    // ...create trades but delay recording them in book...
            trades.push_back(Trade::new(
                resting_order.id,
                matched_order.id,
                price,
                fill_quantity,
                matched_order.side,
            ));

	    // ... reflect the qty changes...
            matched_order.remaining_quantity -= fill_quantity;
            resting_order.remaining_quantity -= fill_quantity;

	    // ...update status of resting order...
            if resting_order.remaining_quantity == Decimal::ZERO {
                resting_orders.pop_front();
            } else {
                resting_order.status = OrderStatus::PartiallyFilled;
            }
        }

        trades
    }

    fn update_order_status(order: &Order) -> OrderStatus {
        if order.remaining_quantity == Decimal::ZERO {
            OrderStatus::Filled
        } else if order.remaining_quantity < order.quantity {
            OrderStatus::PartiallyFilled
        } else {
            OrderStatus::Pending
        }
    }

    fn match_order(&mut self, order: &Order) -> Order {
        let mut matched_order = order.clone();
        let mut trades_to_record = VecDeque::new();

        while matched_order.remaining_quantity > Decimal::ZERO {
            let best_price = self.get_best_matching_price(order.side);

            if let Some(price) = best_price {
                if !self.should_match(&matched_order, price) {
                    break;
                }

                let orders = match order.side {
                    Side::Bid => self.order_book.asks.get_mut(&price),
                    Side::Ask => self.order_book.bids.get_mut(&price),
                };

                if let Some(resting_orders) = orders {
		    // actually fill orders @ price level
                    let mut level_trades = Self::match_at_price_level(&mut matched_order, price, resting_orders);
                    trades_to_record.append(&mut level_trades);

		    // ...and remove the price level if it's empty
                    if resting_orders.is_empty() {
                        match order.side {
                            Side::Bid => { self.order_book.asks.remove(&price); },
                            Side::Ask => { self.order_book.bids.remove(&price); },
                        };
                    }
                }
            } else {
                break;
            }
        }

        // batch record all trades from order
	// BTW this would be a good use case for something like kafka or redis i think
        for trade in trades_to_record {
            self.trade_history.add_trade(trade);
        }

        matched_order.status = Self::update_order_status(&matched_order);
        matched_order
    }

    ///
    /// ------------------ Getter/passthru funcs
    ///

    /// current state of the order book
    pub fn get_order_book(&self, depth: usize) -> (Vec<(Decimal, Decimal)>, Vec<(Decimal, Decimal)>) {
        self.order_book.get_order_book(depth)
    }

    /// status of a specific order statis by ID
    pub fn get_order_status(&self, order_id: u64) -> Option<&Order> {
        self.order_book.get_order_status(order_id)
    }

    /// all orders at a specific price and side
    pub fn orders_at_price(&self, price: Decimal, side: Side) -> VecDeque<Order> {
        self.order_book.orders_at_price(price, side)
    }

    /// best bid price
    pub fn best_bid(&self) -> Option<Decimal> {
        self.order_book.best_bid()
    }

    /// best ask price
    pub fn best_ask(&self) -> Option<Decimal> {
        self.order_book.best_ask()
    }

    /// -------------------
    pub fn get_trade_history(&self, limit: Option<usize>) -> Vec<Trade> {
	match limit {
	    Some(n) => self.trade_history.get_recent_trades(n),
	    None => self.trade_history.get_trades()
	}
    }

}
