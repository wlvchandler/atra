use rust_decimal_macros::dec;
//use chrono::Utc;
use kobara_ob::core::{OrderBook, Order, Side, OrderType, OrderStatus, MatchingEngine};


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

#[test]
fn test_matching_creates_trade() {
    let mut book = MatchingEngine::new();

    book.place_order(create_test_order(1, dec!(100.0), dec!(10.0), Side::Bid, OrderType::Limit));
    book.place_order(create_test_order(2, dec!(100.0), dec!(5.0), Side::Ask, OrderType::Limit));

    let trades = book.get_trade_history(Some(10));
    assert_eq!(trades.len(), 1);

    let trade = &trades[0];
    assert_eq!(trade.maker_order_id, 1);
    assert_eq!(trade.taker_order_id, 2);
    assert_eq!(trade.price, dec!(100.0));
    assert_eq!(trade.quantity, dec!(5.0));
    assert_eq!(trade.side, Side::Ask);
}


#[test]
fn test_multiple_trades() {
    let mut book = MatchingEngine::new();

    // multiple resting buy orders vs large matching sell order
    book.place_order(create_test_order(1, dec!(100.0), dec!(10.0), Side::Bid, OrderType::Limit));
    book.place_order(create_test_order(2, dec!(101.0), dec!(10.0), Side::Bid, OrderType::Limit));
    book.place_order(create_test_order(3, dec!(100.0), dec!(15.0), Side::Ask, OrderType::Limit));

    let trades = book.get_trade_history(Some(10));
    assert_eq!(trades.len(), 2);

    // Since get_trade_history returns newest first,
    // the lower price (executed second) comes first
    assert_eq!(trades[0].maker_order_id, 1);
    assert_eq!(trades[0].price, dec!(100.0));
    assert_eq!(trades[0].quantity, dec!(5.0));

    assert_eq!(trades[1].maker_order_id, 2);
    assert_eq!(trades[1].price, dec!(101.0));
    assert_eq!(trades[1].quantity, dec!(10.0));
}

#[test]
fn test_trade_history_limit() {
    let mut book = MatchingEngine::new();

    for i in 1..=5 {
        book.place_order(create_test_order(i*2-1, dec!(100.0), dec!(10.0), Side::Bid, OrderType::Limit));
        book.place_order(create_test_order(i*2, dec!(100.0), dec!(10.0), Side::Ask, OrderType::Limit));
    }

    // check only 3
    let trades = book.get_trade_history(Some(3));
    assert_eq!(trades.len(), 3);

    assert_eq!(trades[0].maker_order_id, 9);
    assert_eq!(trades[1].maker_order_id, 7);
    assert_eq!(trades[2].maker_order_id, 5);
}


#[test]
fn test_market_order_trades() {
    let mut book = MatchingEngine::new();

    // resting sell order fills market buy
    book.place_order(create_test_order(1, dec!(100.0), dec!(10.0), Side::Ask, OrderType::Limit));
    book.place_order(create_test_order(2, dec!(0.0), dec!(5.0), Side::Bid, OrderType::Market));

    let trades = book.get_trade_history(Some(10));
    assert_eq!(trades.len(), 1);

    let trade = &trades[0];
    assert_eq!(trade.maker_order_id, 1);
    assert_eq!(trade.taker_order_id, 2);
    assert_eq!(trade.price, dec!(100.0));
    assert_eq!(trade.quantity, dec!(5.0));
    assert_eq!(trade.side, Side::Bid);
}

#[test]
fn test_get_all_trades() {
    let mut book = MatchingEngine::new();

    for i in 1..=3 {
        book.place_order(create_test_order(i*2-1, dec!(100.0), dec!(10.0), Side::Bid, OrderType::Limit));
        book.place_order(create_test_order(i*2, dec!(100.0), dec!(10.0), Side::Ask, OrderType::Limit));
    }

    // all trades
    let trades = book.get_trade_history(None);
    assert_eq!(trades.len(), 3);
}



#[test]
fn test_price_time_priority() {
    let mut book = MatchingEngine::new();

    // orders at same price ==> time priority should apply
    book.place_order(create_test_order(1, dec!(100.0), dec!(10.0), Side::Bid, OrderType::Limit));
    book.place_order(create_test_order(2, dec!(100.0), dec!(10.0), Side::Bid, OrderType::Limit));

    // higher price order - should get matched first despite being later in time
    book.place_order(create_test_order(3, dec!(101.0), dec!(10.0), Side::Bid, OrderType::Limit));

    // sell order that will match against all three buy orders
    book.place_order(create_test_order(4, dec!(100.0), dec!(30.0), Side::Ask, OrderType::Limit));

    let trades = book.get_trade_history(None);
    assert_eq!(trades.len(), 3, "Should have three trades");

    // trade1 should be against the highest price (101.0)
    assert_eq!(trades[0].maker_order_id, 3, "First trade should be against highest price order");
    assert_eq!(trades[0].price, dec!(101.0));
    assert_eq!(trades[0].quantity, dec!(10.0));

    // trade2 should be against the earliest order at 100.0
    assert_eq!(trades[1].maker_order_id, 1, "Second trade should be against earliest order at lower price");
    assert_eq!(trades[1].price, dec!(100.0));
    assert_eq!(trades[1].quantity, dec!(10.0));

    // trade3 should be against the later order at 100.0
    assert_eq!(trades[2].maker_order_id, 2, "Third trade should be against later order at lower price");
    assert_eq!(trades[2].price, dec!(100.0));
    assert_eq!(trades[2].quantity, dec!(10.0));
}
