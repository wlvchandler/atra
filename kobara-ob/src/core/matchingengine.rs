use rust_decimal::Decimal;
use super::orderbook::OrderBook;
use super::types::{Order, Side, OrderType, OrderStatus};

pub struct MatchingEngine {
    order_book: OrderBook,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            order_book: OrderBook::new(),
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
            self.order_book.insert_order(matched_order.clone());
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

    /// core matcher against the book
    fn match_order(&mut self, order: &Order) -> Order {
        let mut matched_order = order.clone();

        while matched_order.remaining_quantity > Decimal::ZERO {
            // so we get the best price from the opposite side...
            let best_price = match order.side {
                Side::Bid => self.order_book.asks.keys().next().cloned(),
                Side::Ask => self.order_book.bids.keys().next_back().cloned(),
            };

            // then check if we should match at this price level.
            match best_price {
                Some(price) => {
                    let should_match = match order.order_type {
                        OrderType::Market => true,
                        OrderType::Limit => match order.side {
                            Side::Ask => order.price <= price,
                            Side::Bid => order.price >= price,
                        },
                    };

                    if should_match {
                        // (get mutable access to the opposite side's orders)
                        let orders = match order.side {
                            Side::Bid => self.order_book.asks.get_mut(&price),
                            Side::Ask => self.order_book.bids.get_mut(&price),
                        };

                        if let Some(resting_orders) = orders {
                            // now, we actually fill @ this price level...
                            let mut i = 0;
                            while i < resting_orders.len() && matched_order.remaining_quantity > Decimal::ZERO {
                                let fill_quantity = matched_order.remaining_quantity
                                    .min(resting_orders[i].remaining_quantity);

                                // ...reflect those qty changes...
                                matched_order.remaining_quantity -= fill_quantity;
                                resting_orders[i].remaining_quantity -= fill_quantity;

                                // ...update status of resting order...
                                if resting_orders[i].remaining_quantity == Decimal::ZERO {
                                    let filled_order = resting_orders.remove(i);
                                    self.order_book.orders.insert(filled_order.id, filled_order);
                                } else {
                                    resting_orders[i].status = OrderStatus::PartiallyFilled;
                                    i += 1;
                                }
                            }

                            // ...and remove the price level if it's empty.
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
                None => break,
            }
        }

        // lastly - update order status
        matched_order.status = if matched_order.remaining_quantity == Decimal::ZERO {
            OrderStatus::Filled
        } else if matched_order.remaining_quantity < matched_order.quantity {
            OrderStatus::PartiallyFilled
        } else {
            OrderStatus::Pending
        };

        matched_order
    }

    /// ------------------ Getter/passthru funcs

    /// current state of the order book
    pub fn get_order_book(&self, depth: usize) -> (Vec<(Decimal, Decimal)>, Vec<(Decimal, Decimal)>) {
        self.order_book.get_order_book(depth)
    }

    /// status of a specific order statis by ID
    pub fn get_order_status(&self, order_id: u64) -> Option<&Order> {
        self.order_book.get_order_status(order_id)
    }

    /// all orders at a specific price and side
    pub fn orders_at_price(&self, price: Decimal, side: Side) -> Vec<Order> {
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
}
