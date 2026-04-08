use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
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
    pub id: u64,
    pub instrument_id: u32,
    pub sequence: u64,
    pub price: Decimal,
    pub quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub side: Side,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub timestamp: Option<DateTime<Utc>>,
    pub ingress_timestamp_ns: Option<u64>,
    pub idempotency_key: Option<String>,
}

impl Order {
    pub fn new(
        id: u64,
        instrument_id: u32,
        sequence: u64,
        price: Decimal,
        quantity: Decimal,
        side: Side,
        order_type: OrderType,
    ) -> Self {
        Self {
            id,
            instrument_id,
            sequence,
            price,
            quantity,
            remaining_quantity: quantity,
            side,
            order_type,
            status: OrderStatus::Pending,
            timestamp: Some(Utc::now()),
            ingress_timestamp_ns: None,
            idempotency_key: None,
        }
    }
}


impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
	match self.price.cmp(&other.price) {
	    Ordering::Equal => self.sequence.cmp(&other.sequence),
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
