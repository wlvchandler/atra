# Atra CLI

## Installation & Setup

### Quick Start (Users)

```bash
# Install from PyPI
pip install atra-cli

# Use the CLI
atra book 10
# or
atra book 10
```

### Development Setup

```bash
# Initialize the environment (creates venv and installs dependencies)
./atra init

# Use the CLI
./atra book 10
./atra trades 5
./atra buy 10@100
```

## Docker Usage

```bash
# Using docker-compose
docker-compose run cli book 10
```

## Common Commands

- View the order book: `atra book [depth]`
- View recent trades: `atra trades [limit]`
- Place buy orders: `atra buy 10@100`
- Place sell orders: `atra sell 5@95`
- Cancel an order: `atra cancel [order_id]`

For more information on available commands:
```bash
atra -h
```

cli examples (subject to change of course):

```
# General command structure:
# atra <command> [arguments...] [--format {csv|json|pretty}]
```

### Basic examples

```
$ atra buy 10.00@100.00 --format pretty
Order placed: ID=1, Status=PENDING

$ atra buy 5.00@99.50 --format pretty
Order placed: ID=2, Status=PENDING

$ atra sell 7.00@101.00 --format pretty
Order placed: ID=3, Status=PENDING

$ atra book 10 --format pretty

atraOB (Max depth 10):
     Price   Quantity   Side
------------------------------
    100.00      10.00    BID
     99.50       5.00    BID
------------------------------
    101.00       7.00    ASK
```

#### now placing some orders that should match...

```
$ atra buy 3.00@101.00 --format pretty
Order placed: ID=4, Status=FILLED

$ atra book 10 --format pretty

atraOB (Max depth 10):
     Price   Quantity   Side
------------------------------
    100.00      10.00    BID
     99.50       5.00    BID
------------------------------
    101.00       4.00    ASK # yay

$ atra sell 2.00@99.50 --format pretty
Order placed: ID=5, Status=FILLED

$ atra book 10 --format pretty

atraOB (Max depth 10):
     Price   Quantity   Side
------------------------------
    100.00       8.00    BID # yay
     99.50       5.00    BID
------------------------------
    101.00       4.00    ASK

$ atra buy 2.00 --format pretty 
Order placed: ID=6, Status=FILLED

$ atra book 10 --format pretty

atraOB (Max depth 10):
     Price   Quantity   Side
------------------------------
    100.00       8.00    BID
     99.50       5.00    BID
------------------------------
    101.00       2.00    ASK # yay
```    