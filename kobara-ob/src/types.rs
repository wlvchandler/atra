use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use std::cmp::Ordering;

//
// enums
//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
}

//
// structs
//

#[derive(Debug, Clone, PartialEq, Eq)] // def dont want copy
pub struct Order {
    pub id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub side: Side,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub timestamp: DateTime<Utc>,
}

impl Order {
    pub fn new(id: String, price: Decimal, quantity: Decimal, side: Side, order_type: OrderType) -> Self {
	Self {
	    id, price, quantity,
	    remaining_quantity: quantity,
	    side, order_type,
	    status: OrderStatus::Pending,
	    timestamp: Utc::now(),
	}
    }
}


impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
	match self.price.cmp(&other.price) {
	    Ordering::Equal => self.timestamp.cmp(&other.timestamp),
	    ord => match self.side {
		Side::Bid => ord.reverse(),
		Side::Ask => ord,
	    },
	}
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
