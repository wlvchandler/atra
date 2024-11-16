use rust_decimal_macros::dec;
//use chrono::Utc;
use kobara_ob::{OrderBook, Order, Side, OrderType, OrderStatus};


fn create_test_order(id: u64, price: rust_decimal::Decimal, quantity: rust_decimal::Decimal, side: Side, order_type: OrderType) -> Order {
    Order::new(id, price, quantity, side, order_type)
}


#[test]
fn test_place_order() {
    let mut book = OrderBook::new();

    let order = create_test_order(1, dec!(100.0), dec!(10.0), Side::Bid, OrderType::Limit);
    let result = book.place_order(order.clone());

    assert_eq!(result.status, OrderStatus::Pending);
    assert_eq!(book.get_order_status(1), Some(&result));
}


#[test]
fn test_duplicate_order_id() {
    let mut book = OrderBook::new();

    book.place_order(create_test_order(1, dec!(100.0), dec!(10.0), Side::Bid, OrderType::Limit));

    let result = book.place_order(create_test_order(1, dec!(101.0), dec!(20.0), Side::Ask, OrderType::Limit));

    assert_eq!(result.status, OrderStatus::Pending);
    assert_eq!(book.get_order_status(1), Some(&result));
}


#[test]
fn test_best_bid_ask() {
    let mut book = OrderBook::new();

    book.place_order(create_test_order(1, dec!(98.0), dec!(10.0), Side::Bid, OrderType::Limit));
    book.place_order(create_test_order(2, dec!(99.0), dec!(10.0), Side::Bid, OrderType::Limit));
    book.place_order(create_test_order(3, dec!(101.0), dec!(10.0), Side::Ask, OrderType::Limit));
    book.place_order(create_test_order(4, dec!(102.0), dec!(10.0), Side::Ask, OrderType::Limit));

    assert_eq!(book.best_bid(), Some(dec!(99.0)));
    assert_eq!(book.best_ask(), Some(dec!(101.0)));
}


#[test]
fn test_orders_at_price() {
    let mut book = OrderBook::new();

    let order1 = create_test_order(1, dec!(100.0), dec!(10.0), Side::Bid, OrderType::Limit);
    let order2 = create_test_order(2, dec!(100.0), dec!(20.0), Side::Bid, OrderType::Limit);

    book.place_order(order1.clone());
    book.place_order(order2.clone());

    let orders = book.orders_at_price(dec!(100.0), Side::Bid);
    assert_eq!(orders.len(), 2);
    assert!(orders.contains(&order1));
    assert!(orders.contains(&order2));
}
