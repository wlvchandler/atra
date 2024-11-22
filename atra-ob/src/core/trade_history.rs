use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use crate::core::Side;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trade {
    pub maker_order_id: u64,
    pub taker_order_id: u64,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: Side,
    pub timestamp: DateTime<Utc>,
}

impl Trade {
    pub fn new(maker_order_id: u64, taker_order_id: u64, price: Decimal, quantity: Decimal, side: Side) -> Self {
        Self {
            maker_order_id,
            taker_order_id,
            price,
            quantity,
            side,
            timestamp: Utc::now(),
        }
    }
}

pub struct TxnHistory {
    trades: VecDeque<Trade>,
}

impl TxnHistory {
    pub fn new() -> Self {
	Self {
	    trades: VecDeque::with_capacity(1024)
	}
    }

    /// ------------------
    pub fn add_trade(&mut self, trade: Trade) {
	self.trades.push_back(trade);
    }

    /// ------------------
    pub fn get_trades(&self) -> Vec<Trade> {
	self.trades.iter().cloned().collect()
    }

    /// -------------------
    pub fn get_recent_trades(&self, limit: usize) -> Vec<Trade> {
        self.trades.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}
