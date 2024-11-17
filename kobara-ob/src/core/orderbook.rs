use std::collections::{BTreeMap, HashMap, VecDeque};
use rust_decimal::Decimal;
use super::types::{Order, Side};

#[derive(Debug, Default)]
pub struct OrderBook {
    pub(crate) asks: BTreeMap<Decimal, VecDeque<Order>>,
    pub(crate) bids: BTreeMap<Decimal, VecDeque<Order>>,
    pub(crate) orders: HashMap<u64, Order>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self::default()
    }

    /// -------------
    pub fn place_order(&mut self, order: Order) -> Order {
	let order_clone = order.clone();
        let price_map = match order.side {
            Side::Ask => &mut self.asks,
            Side::Bid => &mut self.bids,
        };

        let orders = price_map.entry(order.price).or_insert_with(VecDeque::new);
        orders.push_back(order.clone());
        //orders.sort_unstable(); // this should maintain time priority - need to verify
        self.orders.insert(order.id, order);
	order_clone
    }

    /// -------------
    pub fn remove_order(&mut self, order_id: u64) -> Option<Order> {
        if let Some(order) = self.orders.remove(&order_id) {
            let price_map = match order.side {
                Side::Ask => &mut self.asks,
                Side::Bid => &mut self.bids,
            };

            if let Some(orders) = price_map.get_mut(&order.price) {
                orders.retain(|o| o.id != order_id);
                if orders.is_empty() {
                    price_map.remove(&order.price);
                }
            }
            Some(order)
        } else {
            None
        }
    }

    /// current state of the order book up to a certain depth.
    pub fn get_order_book(&self, depth: usize) -> (Vec<(Decimal, Decimal)>, Vec<(Decimal, Decimal)>) {
        let bids = self.bids.iter()
            .rev()
            .take(depth)
            .map(|(price, orders)| (*price, orders.iter().map(|order| order.remaining_quantity).sum()))
            .collect();

        let asks = self.asks.iter()
            .take(depth)
            .map(|(price, orders)| (*price, orders.iter().map(|order| order.remaining_quantity).sum()))
            .collect();

        (bids, asks)
    }

    /// order status by id
    pub fn get_order_status(&self, order_id: u64) -> Option<&Order> {
        self.orders.get(&order_id)
    }

    /// get all orders at a specific price & side
    pub fn orders_at_price(&self, price: Decimal, side: Side) -> VecDeque<Order> {
        match side {
            Side::Ask => self.asks.get(&price),
            Side::Bid => self.bids.get(&price),
        }
        .map(|orders| orders.clone())
        .unwrap_or_default()
    }

    /// -------------
    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().cloned()
    }
    /// -------------
    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().cloned()
    }
}
