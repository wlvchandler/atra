[![forthebadge](https://forthebadge.com/images/badges/powered-by-electricity.svg)](https://forthebadge.com)

[![CI/CD](https://github.com/wlvchandler/kobara/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/wlvchandler/kobara/actions/workflows/ci.yml)

----

### build/test instructions

Build the containers
```
docker compose build
docker build -t kobara-cli -f kobara-cli/Dockerfile .
```

Once built, start the matching engine
```
docker compose up -d orderbook
```

The `invm` script is a wrapper to the raw cli that adds some syntax sugar
```
./invm COMMAND [OPTIONS]

Commands:
    book DEPTH
    (buy|sell) ORDERS...   | Multiple orders can be combined in a single command

Order format:
    AMOUNT[@PRICE]	   | If price is omitted order is executed at market

Examples:
    ./invm book 10
    ./invm sell 100@10
    ./invm buy 300
    ./invm buy 50@15 25@14 sell 19@11.20
```

To build the parts individually

Matching Engine:
```
cd kobara-ob
cargo build

# unit tests
cargo nextest run

# start matching engine and API
cargo run --bin server
```

CLI:
```
cd kobara-cli
./scripts/setup_dev
source venv/bin/activate
```


Assuming the MENG API is running, the raw CLI is used like:
```
# ./invm book 10
./cli.py book 10

# ./invm sell 100@10
./cli.py place 10.00 100.00 ask limit

# ./invm buy 300
./cli.py place 0.00  100.00 buy market

# ./invm buy 50@15 25@14 sell 19@11.20
./cli.py place 15.00 50.00  buy  limit
./cli.py place 14.00 25.00  buy  limit
./cli.py place 11.20 19.00  sell limit
```

### test output
```
will@DESKTOP-71HHMI5:~/Projects/kote/kobara-ob$ cargo nextest run
   Compiling kobara-ob v0.1.0 (/home/will/Projects/kote/kobara-ob)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.28s
------------
 Nextest run ID 4e3a11fa-cf2c-41b3-9edf-7f8dc21f6055 with nextest profile: default
    Starting 10 tests across 3 binaries
        PASS [   0.002s] kobara-ob::order_book_tests test_best_bid_ask
        PASS [   0.002s] kobara-ob::order_book_tests test_duplicate_order_id
        PASS [   0.002s] kobara-ob::order_book_tests test_get_all_trades
        PASS [   0.002s] kobara-ob::order_book_tests test_market_order_trades
        PASS [   0.002s] kobara-ob::order_book_tests test_matching_creates_trade
        PASS [   0.002s] kobara-ob::order_book_tests test_multiple_trades
        PASS [   0.002s] kobara-ob::order_book_tests test_orders_at_price
        PASS [   0.002s] kobara-ob::order_book_tests test_price_time_priority
        PASS [   0.002s] kobara-ob::order_book_tests test_trade_history_limit
        PASS [   0.003s] kobara-ob::order_book_tests test_place_order
------------
     Summary [   0.005s] 10 tests run: 10 passed, 0 skipped
```

