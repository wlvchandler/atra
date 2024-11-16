use std::collections::{BTreeMap, HashMap};
use rust_decimal::Decimal;
use super::types::{Order, Side, OrderType, OrderStatus};

#[derive(Debug, Default)]
pub struct OrderBook {
    asks:   BTreeMap<Decimal, Vec<Order>>,
    bids:   BTreeMap<Decimal, Vec<Order>>,
    orders: HashMap<u64, Order>,
}

impl OrderBook {
    pub fn new() -> Self {
	Self::default()
    }


    pub fn place_order(&mut self, order: Order) -> Order {
	match order.order_type {
            OrderType::Limit  => self.place_limit_order(order),
            OrderType::Market => self.place_market_order(order),
	}
    }

    // try to match with existing order first and add remaining qty to book
    fn place_limit_order(&mut self, order: Order) -> Order {
	let matched_order = self.match_order(&order);
	if matched_order.remaining_quantity > Decimal::ZERO {
	    let price_map = match order.side {
		Side::Ask => &mut self.asks,
		Side::Bid => &mut self.bids,
	    };

	    let orders = price_map.entry(order.price).or_insert_with(Vec::new);
	    orders.push(matched_order.clone());
	    orders.sort_unstable(); // maintain time priority
	}

	self.orders.insert(matched_order.id.clone(), matched_order.clone());
	matched_order
    }


    fn place_market_order(&mut self, mut order: Order) -> Order {
	order = self.match_order(&order);
	self.orders.insert(order.id.clone(), order.clone());
	order
    }

    fn match_order(&mut self, order: &Order) -> Order {
        let mut matched_order = order.clone();

        while matched_order.remaining_quantity > Decimal::ZERO {
            // Get the appropriate order book side and best price
            let best_price = match order.side {
                Side::Bid => self.asks.keys().next().cloned(),
                Side::Ask => self.bids.keys().next_back().cloned(),
            };

            // Check if we should match at this price level
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
                        // Get mutable access to the appropriate order book side
                        let orders = match order.side {
                            Side::Bid => self.asks.get_mut(&price),
                            Side::Ask => self.bids.get_mut(&price),
                        };

                        if let Some(resting_orders) = orders {
                            // Match at this price level
                            let mut i = 0;
                            while i < resting_orders.len() && matched_order.remaining_quantity > Decimal::ZERO {
                                let fill_quantity = matched_order.remaining_quantity
                                    .min(resting_orders[i].remaining_quantity);

                                // Update quantities
                                matched_order.remaining_quantity -= fill_quantity;
                                resting_orders[i].remaining_quantity -= fill_quantity;

                                // Update status of resting order
                                if resting_orders[i].remaining_quantity == Decimal::ZERO {
                                    let filled_order = resting_orders.remove(i);
                                    self.orders.insert(filled_order.id.clone(), filled_order);
                                } else {
                                    resting_orders[i].status = OrderStatus::PartiallyFilled;
                                    i += 1;
                                }
                            }

                            // Remove the price level if it's empty
                            if resting_orders.is_empty() {
                                match order.side {
                                    Side::Bid => self.asks.remove(&price),
                                    Side::Ask => self.bids.remove(&price),
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

        // Update order status
        matched_order.status = if matched_order.remaining_quantity == Decimal::ZERO {
            OrderStatus::Filled
        } else if matched_order.remaining_quantity < matched_order.quantity {
            OrderStatus::PartiallyFilled
        } else {
            OrderStatus::Pending
        };

        matched_order
    }


    pub fn get_order_book(&self, depth: usize) -> (Vec<(Decimal,Decimal)>, Vec<(Decimal, Decimal)>) {
	let bids = self.bids.iter()
	    .rev()
	    .take(depth)
	    .map(|(price,orders)| (*price, orders.iter().map(|order|order.remaining_quantity).sum()))
	    .collect();

	let asks = self.asks.iter()
	    .take(depth)
	    .map(|(price,orders)| (*price, orders.iter().map(|order|order.remaining_quantity).sum()))
	    .collect();

	(bids, asks)
    }

    pub fn get_order_status(&self, order_id: u64) -> Option<&Order> {
	self.orders.get(&order_id)
    }

    pub fn orders_at_price(&self, price: Decimal, side: Side) -> Vec<Order> {
	match side {
	    Side::Ask => self.asks.get(&price),
	    Side::Bid => self.bids.get(&price),
	}
	.map(|orders| orders.clone())
	.unwrap_or_default()
    }

    pub fn get_best_opposite_price(&self, side: Side) -> Option<Decimal> {
	match side {
	    Side::Bid => self.asks.keys().next(),
	    Side::Ask => self.bids.keys().next_back(),
	}.cloned()
    }

    pub fn best_bid(&self) -> Option<Decimal> {
	self.bids.keys().next_back().cloned()
    }

    pub fn best_ask(&self) -> Option<Decimal> {
	self.asks.keys().next().cloned()
    }

}
