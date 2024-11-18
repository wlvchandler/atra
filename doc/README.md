# Documentation

## Components


### Core / kobara-OB

#### Matching Engine (matchingengine.rs)

The central component handling order matching logic. It processes both limit and market orders, respecting price-time priority, and records the trade history.


`match_order(order)` :  core matching logic

`place_order(order)` :  passthru entry point for new orders

`place_limit_order(order)` : shockingly - handles limit orders

`place_market_order(order)` :  :-)


#### Order Book (orderbook.rs)

The structure responsible for maintaining bid and ask sides of the book. Uses `BTreeMap` for price levels and `VecDeque` for orders at each level.

We provide operations for:

- Adding/Removing orders

- Querying (best bid/ask, book to a certain depth, order-by-id)


#### Transaction History (trade_history.rs)

Record of all executed trades in chronological order. It supports querying recent trades as well.

`add_trade(trade)` : Records new trades

`get_trades()` :     Retrieves all trades

`get_recent_trades(N)`: Retrieve `N` most recent trades


### Interface Layer

Protobuf/gRPC was chosen for both simplicity (despite amount of generated mumbo jumbo it makes) and ease of use between languages.

#### Protobuf (orderbook.proto)

This file defines the gRPC service interface and provides the following message types:

| Message Type | Description | Fields | Field Types |
|-------------|-------------|---------|-------------|
| `OrderRequest` | New order submission | `id`<br>`price`<br>`quantity`<br>`side`<br>`order_type` | `uint64`<br>`string`<br>`string`<br>`Side` enum (BID=0, ASK=1)<br>`OrderType` enum (LIMIT=0, MARKET=1) |
| `OrderResponse` | Order status update | `id`<br>`price`<br>`quantity`<br>`remaining_quantity`<br>`side`<br>`order_type`<br>`status`<br>`timestamp` | `uint64`<br>`string`<br>`string`<br>`string`<br>`Side` enum<br>`OrderType` enum<br>`OrderStatus` enum (PENDING=0, PARTIALLY_FILLED=1, FILLED=2, CANCELLED=3)<br>`Timestamp` |
| `GetOrderBookRequest` | Book depth query | `depth` | `uint32` |
| `OrderBookResponse` | Current book state | `bids`<br>`asks` | List of `OrderBookLevel`:<br>- `price` (string)<br>- `quantity` (string) |
| `GetTradeHistoryRequest` | Trade history query | `limit` | `uint32` |
| `TradeHistoryResponse` | Recent trades | `trades` | List of `Trade`:<br>- `maker_order_id` (uint64)<br>- `taker_order_id` (uint64)<br>- `price` (string)<br>- `quantity` (string)<br>- `side` (Side enum)<br>- `timestamp` (Timestamp) |

Notes on that:

- Price/Qty use string representation to maintain decimal precision
- protobuf has a builtin timestamp message, I didn't know that
- Lists are dynamically sized
- Enums start at 0
- `OrderBookLevel` represents the aggregated qty at a price level


#### GRPC service (service.rs)

API access to the core backend services. Maintains logical separation (and performance characteristics) from the backend. So far handles:

- Order placement for either side, market or limit
- Queries (Order Book, Trade History, Order Status)



### Client Tools

#### Raw CLI (cli.py)

CLI for interacting with the backend. Meant to be a more raw but programmable API.

| Command |  Description |
| ------ | --------- |
| place  | Submit new orders|
|book | View current order book state|
|trades| View recent trade history|


```
./cli.py {book|place}
./cli.py place {id} {price} {qty} {bid|ask} {limit|market}
```

Note: for `market` orders, price must be set as `0.0`


#### kobaraVM CLI (`invm`)

```
./invm COMMAND [OPTIONS]

Commands:
    book DEPTH
    (buy|sell) ORDERS...   | Multiple orders can be combined in a single command

Order format:
    AMOUNT[@PRICE]	   | If price is omitted order is executed at market
```

```
Examples:

## View order book to a certain depth
./invm book 10
# ./cli.py book 10

## See Trade History
./invm trades 10
# ./cli.py trades 10


## Order placement examples
##

./invm sell 100@10
# ./cli.py place 10.00 100.00 ask limit

./invm buy 300
# ./cli.py place 0.00  100.00 buy market

./invm buy 50@15 25@14 sell 19@11.20
# ./cli.py place 15.00 50.00  buy  limit
# ./cli.py place 14.00 25.00  buy  limit
# ./cli.py place 11.20 19.00  sell limit
```


## Technical details

### Incoming Orders

1. Market orders are matched immediately at best available price, or rejected if no available sells.
2. Limit orders are matched if they cross with existing orders
3. Remaining quantity on limit orders is added to the book


### Price-Time Priority:

1. Orders are matched at the best price first
2. Within each price level, older orders are matched first

### Trade Generation

1. A Trade is created when orders are filled, and recorded when the order is finished processing.
2. Each trade records:
- Maker and taker order IDs
- Price
- Qty
- Side (BID/ASK)
- Timestamp


### Data structures

#### Order Book

| Field | Type |
| ----- | ---- |
| asks  | `BTreeMap<Decimal, VecDeque<Order>>` |
| bids  | `BTreeMap<Decimal, VecDeque<Order>>` |
| orders  | `HashMap<u64, Order>` |


#### Order

| Field | Type |
| ----- | ---- |
| id    | u64 |
| price  | Decimal |
| quantity  | Decimal |
| remaining_quantity | Decimal |
| side | Side(enum) |
| order_type| OrderType |
| status | OrderStatus |
| timestamp | DateTime<Utc> |



### Performance Considerations

1. Data structure choices
- BTreeMap for price levels  : `O(log n)` lookups
- VecDeque for order queues  : `O(1)` push/pop
- BTreeMap for order lookups : `O(1)` access

2. Trade recording
- Trades are recorded in memory with fixed capacity
- For persistent data in production we would use Kafka/Redis to send to a commit service


## Future improvements

1.  Technical
- Persistent storage for trades
- WebSocket API for real-time updates
- Rate limiting, order validation
- Support for other order types

2. Operational
- Observability/Monitoring
- Order book snapshots
- Trade reconciliation*
- Multi-asset support

3. Perf. Optimizations
- Order book compression (level-based, likely)
- Batch processing
- Memory usage optimization
- Lock-free structures



#### *trade reconciliation

Trades are recorded but there's no systematic reconciliation process to ensure consistency. This is mandatory in traditional finance but not sure about crypto, that being said it's bad practice NOT to, I'd think.

We'd want to:

1. Match all trades with orders
2. Verify quantities match
3. Check position changes
4. Validate price levels
5. Generate report




### proposed optimization explanations

#### orderbook compression
The order book runs in memory and stores every order individually at each price level, so that could get RAM intensive.

We could do level-based compression:

```rust
pub struct CompressedLevel {
    price: Decimal,
    total_quantity: Decimal,
    order_count: u32,
    timestamp_range: (DateTime<Utc>, DateTime<Utc>),
}

pub struct CompressionConfig {
    min_orders_to_compress: u32,
    max_timestamp_spread: Duration,
    price_level_tolerance: Decimal,
}
```

and follow an algorithm like:

```rust
impl OrderBook{
    //...

    fn compress_level(&mut self, price: Decimal) {
        // 1. id similar price levels
	// 2. agg orders within tolerance
	// 3. maintain the time priority approximation
	// 4. update indexes
    }

    // and its equivalent:
    fn decompress_for_matching(&self, price: Decimal) -> VecDeque<Order> { ... }

    //...
}
```

#### lock-free structures

Right now we're locking with mutexes:

```rust
pub struct OrderBookService {
    engine: Arc<Mutex<MatchingEngine>>,
}
```

Which means:
- All order operations are serialized through this lock
- Only 1 thread can process orders at a time
- Long running ops block other orders


We have a few ways to handle this lock-free:

1. Lock-free order book (this is half pseudocode)

```rust
pub struct LockFreeOrderBook {
use crossbeam::epoch;
use crossbeam::ebr::Guard;

pub struct LockFreeOrderBook {
    bids: Arc<crossbeam_skiplist::SkipList<PriceKey, LevelNode>>,
    asks: Arc<crossbeam_skiplist::SkipList<PriceKey, LevelNode>>,
    orders: crossbeam_hashmap::HashMap<OrderId, OrderRef>,
}

struct LevelNode {
    orders: crossbeam::queue::SegQueue<Order>,
    total_quantity: AtomicDecimalType,
}
}
```

- `crossbeam_skiplist::SkipList` seems to be a good concurrent skip list that provides insertion/deletion/traversal with atomic CAS operations.
-- https://docs.rs/crossbeam-skiplist/latest/crossbeam_skiplist/
- we could give each `LevelNode` a segmented queue for storing orders, also looks like it provides good atomic operations to manage the head/tail pointers
-- https://docs.rs/crossbeam/latest/crossbeam/queue/struct.SegQueue.html
- We'd need to find a good atomic decimal type or spin our own. not too hard.
- hash_map isn'tthe actual name but there is https://docs.rs/concache/latest/concache/crossbeam/index.html
- Epoch-based garbage collection. Meaning threads operate in 'epochs' and memory is reclaimed only when all threads have exited the relevant epoch, so no thread accesses freed memory. Basically smart pointers for threads.
-- https://docs.rs/crossbeam/latest/crossbeam/epoch/index.html

2. multi-version concurrency control

```rust
pub struct MVCCOrderBook {
    current: AtomicPtr<LockFreeOrderBook>,
    versions: crossbeam::queue::SegQueue<OrderBookVersion>,
}
```

Could ostensibly be used in tandem with a skip list to give use fast price level access (via Skip list) and consistent snapshots (via MVCC):

```rust
pub struct HybridOrderBook {
    levels: Arc<crossbeam_skiplist::SkipList<PriceKey, LevelNode>>,
    versions: crossbeam::queue::SegQueue<OrderBookVersion>,
    current_version: AtomicU64, // mvcc gives us point-in-time queries? not totally sure if I'm thinking about this correctly.
}
```


3. There is also lock-free applied to matching:

```
    fn match_order(&self, order: &Order) -> Order {
        loop {
            let guard = epoch::pin();
            let book_snapshot = self.order_book.get_snapshot(&guard);
            let result = self.try_match(order, book_snapshot);

            if self.commit_match(result, &guard) {
                return result;
            }
            // retry instead of block if concurrent modification detected
        }
        ...
    }
```
