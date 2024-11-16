use rustler::{Encoder, Env, Error, NifResult, NifStruct, ResourceArc, Term};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Mutex;

mod orderbook;
mod types;

use types::{Order, OrderType, Side, OrderStatus};

#[derive(NifStruct)]
#[module = "OrderBookService.Types.Order"]
pub struct NifOrder {
    pub id: u64,
    pub price: String,
    pub quantity: String,
    pub remaining_quantity: String,
    pub side: String,
    pub order_type: String,
    pub status: String,
}

impl From<Order> for NifOrder {
    fn from(order: Order) -> Self {
        NifOrder {
            id: order.id,
            price: order.price.to_string(),
            quantity: order.quantity.to_string(),
            remaining_quantity: order.remaining_quantity.to_string(),
            side: match order.side {
                Side::Bid => "bid".to_string(),
                Side::Ask => "ask".to_string(),
            },
            order_type: match order.order_type {
                OrderType::Limit => "limit".to_string(),
                OrderType::Market => "market".to_string(),
            },
            status: match order.status {
                OrderStatus::Pending => "pending".to_string(),
                OrderStatus::PartiallyFilled => "partially_filled".to_string(),
                OrderStatus::Filled => "filled".to_string(),
                OrderStatus::Cancelled => "cancelled".to_string(),
            },
        }
    }
}

struct OrderBookResource {
    book: Mutex<orderbook::OrderBook>,
}

#[rustler::nif]
fn new() -> NifResult<ResourceArc<OrderBookResource>> {
    Ok(ResourceArc::new(OrderBookResource {
        book: Mutex::new(orderbook::OrderBook::new()),
    }))
}

#[rustler::nif]
fn place_order(book_ref: ResourceArc<OrderBookResource>, order: NifOrder) -> NifResult<NifOrder> {
    let price = Decimal::from_str(&order.price).map_err(|_| Error::BadArg)?;
    let quantity = Decimal::from_str(&order.quantity).map_err(|_| Error::BadArg)?;

    let side = match order.side.as_str() {
        "bid" => Side::Bid,
        "ask" => Side::Ask,
        _ => return Err(Error::BadArg),
    };

    let order_type = match order.order_type.as_str() {
        "limit" => OrderType::Limit,
        "market" => OrderType::Market,
        _ => return Err(Error::BadArg),
    };

    let order = Order::new(
        order.id,
        price,
        quantity,
        side,
        order_type,
    );

    let mut book = book_ref.book.lock().map_err(|_| Error::RaiseAtom("lock_error"))?;
    let result = book.place_order(order);
    Ok(result.into())
}

#[rustler::nif]
fn get_order_book(book_ref: ResourceArc<OrderBookResource>, depth: usize) -> NifResult<(Vec<(String, String)>, Vec<(String, String)>)> {
    let book = book_ref.book.lock().map_err(|_| Error::RaiseAtom("lock_error"))?;
    let (bids, asks) = book.get_order_book(depth);

    let bids = bids.into_iter()
        .map(|(p, q)| (p.to_string(), q.to_string()))
        .collect();
    let asks = asks.into_iter()
        .map(|(p, q)| (p.to_string(), q.to_string()))
        .collect();

    Ok((bids, asks))
}

#[rustler::nif]
fn get_order_status(book_ref: ResourceArc<OrderBookResource>, order_id: u64) -> NifResult<Option<NifOrder>> {
    let book = book_ref.book.lock().map_err(|_| Error::RaiseAtom("lock_error"))?;
    Ok(book.get_order_status(order_id).map(|order| order.clone().into()))
}

rustler::init! {
    "Elixir.OrderBookService.Native",
    [new, place_order, get_order_book, get_order_status],
    load = load
}

fn load(env: Env, _: Term) -> bool {
    rustler::resource!(OrderBookResource, env);
    true
}
