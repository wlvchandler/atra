use rust_decimal::Decimal;


/*
 - Debug: enables enum to be formatted with {:?}  in `println!` or other
 - Clone: allows enum to be explicitly dup'd with `.clone()`
 - Copy:  makes copyable vs being moved (std::move default?)
 - PartialEq:  allows E == E and E |= E
 - Eq: equality comps are defined everywhere
 */


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone, PartialEq, Eq)] // def dont want copy
pub enum Order {
    pub id: u64,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: Side,
    pub timestamp: 64,
}

impl Order {
    pub fn new(id: u64, price: Decimal, quantity: Decimal, side: Side, timestamp: u64) -> Self {
	Self {
	    id,
	    price,
	    quantity,
	    side,
	    timestamp,
	}
    }
}
