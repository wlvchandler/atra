use std::collections::{BTreeMap, HashMap};
use rust_decimal::Decimal;
use crate::types::{Order, Side, OrderType, OrderStatus};

#[derive(Debug, Default)]
pub struct OrderBook {
    asks:   BTreeMap<Decimal, Vec<Order>>,
    bids:   BTreeMap<Decimal, Vec<Order>>,
    orders: HashMap<String, Order>,
}

impl OrderBook {
    pub fn new() -> Self {
	Self::default()
    }


    pub fn place_order(&mut self, mut order: Order) -> Order {
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
	let opposite_orders = match order.side {
	    Side::Bid => &mut self.asks,
            Side::Ask => &mut self.bids,
	};

	while matched_order.remaining_quantity > Decimal::Zero {
	    match self.get_best_opposite_price(order.side) {
		Some(price) if order.order_type == OrderType::Market || (order.order_type == OrderType::Limit  && self.price_matches(price, order.price, order.side)) => {
		    if let Some(orders) = opposite_orders.get_mut(&price) {
			// match w orders at this price lvl then clean up empty price lvls
			self.match_at_price_level(&mut matched_order, orders);
			if orders.is_empty() {
			    opposite_orders.remove(&price);
			}
		    }
		}
		_ => break,
	    }
	}

	matched_order.status = if matched_order.remaining_quantity == Decimal::ZERO {
	    OrderStatus::Filled
	} else if matched_order.remaining_quantity < matched_order.quantity {
	    OrderStatus::PartiallyFilled
	} else {
	    OrderStatus::Pending
	};

	matched_order
    }

    fn price_matches(&self, market_price: Decimal, order_price: Decimal, side: Side) -> bool {
	match side {
	    Side::Ask => order_price <= market_price,
	    Side::Bid => order_price >= market_price,
	}
    }

    fn match_at_price_level(&mut self, incoming_order: &mut Order, resting_orders: &mut Vec<Order>) {
	let mut i = 0;
	while i < resting_orders.len() && incoming_order.remaining_quantity > Decimal::ZERO {
	    let fill_quantity = incoming_order.remaining_quantity.min(resting_orders[i].remaining_quantity);

	    incoming_order.remaining_quantity -= fill_quantity;
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

    pub fn get_order_status(&self, order_id: &str) -> Option<&Order> {
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
