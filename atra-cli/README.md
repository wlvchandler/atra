I used python3.10 but I believe this should work for other nearby versions as well.

### dev/testing setup

Set up venv and generate grpc connectors:
```
python scripts/setup_dev.py
```

then activate venv:
```
source venv/bin/activate
```


cli examples (subject to change of course):
```
# ./cli.py {book|place}
# ./cli.py place {id} {price} {qty} {bid|ask} {limit|market}

(venv) will@DESKTOP-71HHMI5:~/Projects/kote/atra-cli
$ python cli.py place 1 100.00 10.00 bid limit
Order placed: ID=1, Status=0

(venv) will@DESKTOP-71HHMI5:~/Projects/kote/atra-cli
$ python cli.py place 2 99.50 5.00 bid limit

Order placed: ID=2, Status=0
(venv) will@DESKTOP-71HHMI5:~/Projects/kote/atra-cli
$ python cli.py place 3 101.00 7.00 ask limit
Order placed: ID=3, Status=0

(venv) will@DESKTOP-71HHMI5:~/Projects/kote/atra-cli
$ python cli.py book 10

Bids:
  100: 10
  99.5: 5

Asks:
  101: 7


# now placing some orders that should match...

(venv) will@DESKTOP-71HHMI5:~/Projects/kote/atra-cli
$ python cli.py place 4 101.00 3.00 bid limit
Order placed: ID=4, Status=2

(venv) will@DESKTOP-71HHMI5:~/Projects/kote/atra-cli
$ python cli.py book 10

Bids:
  100: 10
  99.5: 5

Asks:
  101: 4  # yay

(venv) will@DESKTOP-71HHMI5:~/Projects/kote/atra-cli
$ python cli.py place 5 99.50 2.00 ask limit
Order placed: ID=5, Status=2

(venv) will@DESKTOP-71HHMI5:~/Projects/kote/atra-cli
$ python cli.py book 10

Bids:
  100: 8  # yay
  99.5: 5

Asks:
  101: 4

(venv) will@DESKTOP-71HHMI5:~/Projects/kote/atra-cli
$ python cli.py place 6 0 2.00 bid market
Order placed: ID=6, Status=2

(venv) will@DESKTOP-71HHMI5:~/Projects/kote/atra-cli
$ python cli.py book 10

Bids:
  100: 8
  99.5: 5

Asks:
  101: 2  # yay

```