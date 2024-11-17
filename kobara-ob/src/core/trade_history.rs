use std::collections::VecDeque;
use crate::domain::Trade;

pub struct TxnHistory {
    trades: VecDeque<Trade>,
}

impl TxnHistory {
    pub fn new() -> Self {
	Self {
	    trades: VecDeque::new()
	}
    }

    pub fn add_trade(&mut self, trade: Trade) {
	self.trades.push_back(trade);
    }

    pub fn get_trades(&self) -> Vec<Trade> {
	self.trades.iter().cloned().collect()
    }
}
