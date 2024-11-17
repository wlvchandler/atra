#!/usr/bin/env python3
import argparse
import grpc
import os
from decimal import Decimal
from generated.orderbook_pb2 import (
    OrderRequest, GetOrderBookRequest, GetOrderStatusRequest,
    Side, OrderType
)
from generated.orderbook_pb2_grpc import OrderBookServiceStub

def connect():
    host = os.getenv('kobara_OB_HOST', '127.0.0.1')  # localhost if not in container
    port = os.getenv('kobara_OB_PORT', '50051')
    channel = grpc.insecure_channel(f'{host}:{port}')
    return OrderBookServiceStub(channel)

def place_order(stub, args):
    request = OrderRequest(
        id=args.id,
        price=str(Decimal(args.price)),
        quantity=str(Decimal(args.quantity)),
        side=Side.BID if args.side.upper() == "BID" else Side.ASK,
        order_type=OrderType.LIMIT if args.type.upper() == "LIMIT" else OrderType.MARKET
    )
    response = stub.PlaceOrder(request)
    print(f"Order placed: ID={response.id}, Status={response.status}")

def get_book(stub, args):
    request = GetOrderBookRequest(depth=args.depth)
    response = stub.GetOrderBook(request)

    print("\nkobaraOB (Max depth {}):".format(args.depth))
    print(f"{'Price':>10} {'Quantity':>10} {'Side':>6}")
    print("-" * 30)

    # bids
    for level in sorted(response.bids, key=lambda x: Decimal(x.price), reverse=True):
        price = Decimal(level.price).quantize(Decimal('0.01'))
        quantity = Decimal(level.quantity).quantize(Decimal('0.01'))
        print(f"{price:>10} {quantity:>10} {'BID':>6}")

    print("-" * 30)

    # asks
    for level in sorted(response.asks, key=lambda x: Decimal(x.price)):
        price = Decimal(level.price).quantize(Decimal('0.01'))
        quantity = Decimal(level.quantity).quantize(Decimal('0.01'))
        print(f"{price:>10} {quantity:>10} {'ASK':>6}")

def main():
    parser = argparse.ArgumentParser(description='OrderBook CLI')
    subparsers = parser.add_subparsers(dest='command')

    place_parser = subparsers.add_parser('place')
    place_parser.add_argument('id', type=int)
    place_parser.add_argument('price', type=float)
    place_parser.add_argument('quantity', type=float)
    place_parser.add_argument('side', choices=['bid', 'ask'])
    place_parser.add_argument('type', choices=['limit', 'market'])

    book_parser = subparsers.add_parser('book')
    book_parser.add_argument('depth', type=int)

    args = parser.parse_args()
    if not args.command:
        parser.print_help()
        return

    try:
        stub = connect()
        if args.command == 'place':
            place_order(stub, args)
        elif args.command == 'book':
            get_book(stub, args)
    except grpc.RpcError as e:
        print(f"Error: {e.details()}")

if __name__ == '__main__':
    main()
