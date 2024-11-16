use std::collections::{BTreeMap, HashMap};
use rust_decimal::Decimal;
use crate::types::{Order, Side};

#[derive(Debug, Default)]
pub struct OrderBook {
    asks: BTreeMap<Decimal, Vec<Order>>,
    bids: BTreeMap<Decimal, Vec<Order>>,
    orders: HashMap<u64, Order>,
}

impl OrderBook {
    pub fn new() -> Self {
	Self {
	    asks:   BTreemap::new(),
	    bids:   BTreemap::new(),
	    orders: BTreemap::new(),
	}
    }

    pub fn add_order(&mut self, order: Order) -> Result<(), String> {
	if self.orders.contains_key(&order.id) {
	    return Err("Order ID already exists".to_string());
	}

	let price_map = match order.side {
	    Side::Ask => &mut self.asks,
	    Side::Bid => &mut self.bids,
	};

	price_map
	    .entry(order.price)
	    .or_insert_with(Vec::new)
	    .push(order.clone());

	self.orders.insert(order.id, order);
	Ok(())
    }


    pub fn cancel_order(&mut self, order_id: u64) -> Result<Order, String> {
	let order = self.orders
	    .remove(&order_id)
	    .ok_or("Order not found")?;

	let price_map = match order.side {
	    Side::Ask => &mut self.asks,
	    Side::Bid => &mut self.bids,
	};

	if let Some(orders) = price_map.get_mut(&order.price) {
	    orders.retain(|order| order.id != order_id);
	    if orders.is_empty() {
		price_map.remove(&order.price);
	    }
	}

	Ok(order)
    }
}
