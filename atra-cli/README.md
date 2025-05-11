I used python3.10 but I believe this should work for other nearby versions as well.

### dev/testing setup

Set up venv and generate grpc connectors:
```
python scripts/setup_env
```

then activate venv:
```
source venv/bin/activate
```


cli examples (subject to change of course):

```
# General command structure:
# invm <command> [arguments...] [--format {csv|json|pretty}]
```

### Basic examples

```
$ invm buy 10.00@100.00 --format pretty
Order placed: ID=1, Status=PENDING

$ invm buy 5.00@99.50 --format pretty
Order placed: ID=2, Status=PENDING

$ invm sell 7.00@101.00 --format pretty
Order placed: ID=3, Status=PENDING

$ invm book 10 --format pretty

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
$ invm buy 3.00@101.00 --format pretty
Order placed: ID=4, Status=FILLED

$ invm book 10 --format pretty

atraOB (Max depth 10):
     Price   Quantity   Side
------------------------------
    100.00      10.00    BID
     99.50       5.00    BID
------------------------------
    101.00       4.00    ASK # yay

$ invm sell 2.00@99.50 --format pretty
Order placed: ID=5, Status=FILLED

$ invm book 10 --format pretty

atraOB (Max depth 10):
     Price   Quantity   Side
------------------------------
    100.00       8.00    BID # yay
     99.50       5.00    BID
------------------------------
    101.00       4.00    ASK

$ invm buy 2.00 --format pretty 
Order placed: ID=6, Status=FILLED

$ invm book 10 --format pretty

atraOB (Max depth 10):
     Price   Quantity   Side
------------------------------
    100.00       8.00    BID
     99.50       5.00    BID
------------------------------
    101.00       2.00    ASK # yay
```    