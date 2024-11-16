use rust_decimal_macros::dec;
use kobara_ob::{OrderBook, Order, Side};

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

    let order1 = Order::new(1, dec!(100.0), dec!(10.0), Side::Bid, 1000);
    let order2 = Order::new(1, dec!(101.0), dec!(20.0), Side::Ask, 1001);

    assert!(book.add_order(order1).is_ok());
    assert!(book.add_order(order2).is_err());
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
