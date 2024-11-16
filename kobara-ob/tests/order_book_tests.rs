use rust_decimal_macros::dec;
use chrono::Utc;
use kobara_ob::{OrderBook, Order, Side, OrderType, OrderStatus};


fn create_test_order(id: &str, price: rust_decimal::Decimal, quantity: rust_decimal::Decimal, side: Side, order_type: OrderType) -> Order {
    Order::new(id.to_string(), price, quantity, side, order_type,);
}


#[test]
fn test_add_order() {
    let mut book = OrderBook::new();

    let order = Order::new(1, dec!(100.0), dec!(10.0), Side::Bid, 1000);

    assert!(book.add_order(order.clone()).is_ok());
    assert_eq!(book.get_order(1), Some(&order));
}


#[test]
fn test_duplicate_order_id() {
    let mut book = OrderBook::new();

    book.place_order(create_test_order(1, dec!(100.0), dec!(10.0), Side::Bid, OrderType::Limit));

    let result = book.place_order(create_test_order(1, dec!(101.0), dec!(20.0), Side::Ask, OrderType::Limit));

    assert_eq!(result.id, "1");
    assert_eq!(result.price, dec!(100.0));
}


#[test]
fn test_cancel_order() {
    let mut book = OrderBook::new();

    let order = Order::new(1, dec!(100.0), dec!(10.0), Side::Bid, 1000);
    book.add_order(order.clone()).unwrap();

    let cancelled = book.cancel_order(1).unwrap();

    assert_eq!(cancelled, order);
    assert!(book.get_order(1).is_none());
}


#[test]
fn test_best_bid_ask() {
    let mut book = OrderBook::new();

    book.add_order(create_test_order("1", dec!(98.0), dec!(10.0),  Side::Bid, OrderType::Limit));
    book.add_order(create_test_order("2", dec!(99.0), dec!(10.0),  Side::Bid, OrderType::Limit));
    book.add_order(create_test_order("3", dec!(101.0), dec!(10.0), Side::Ask, OrderType::Limit));
    book.add_order(create_test_order("4", dec!(102.0), dec!(10.0), Side::Ask, OrderType::Limit));

    let (bids, asks) = book.get_order_book(2);

    assert_eq!(bids[0].0, dec!(99.0));
    assert_eq!(asks[0].0, dec!(101.0));
}


#[test]
fn test_orders_at_price() {
    let mut book = OrderBook::new();

    let order1 = Order::new(1, dec!(100.0), dec!(10.0), Side::Bid, OrderType::Limit);
    let order2 = Order::new(2, dec!(100.0), dec!(20.0), Side::Bid, OrderType::Limit);
    book.add_order(order1.clone()).unwrap();
    book.add_order(order2.clone()).unwrap();

    let orders = book.orders_at_price(dec!(100.0), Side::Bid);
    assert_eq!(orders.len(), 2);
    assert!(orders.contains(&order1));
    assert!(orders.contains(&order2));
}
