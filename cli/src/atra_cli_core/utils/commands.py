import sys
import random
from typing import List, Tuple, Dict
import grpc

from .parser import create_parser

from ..orderbook.formatter import OrderBookFormatter
from ..orderbook.client import OrderBookClient


def generate_order_id() -> int:
    return random.randint(1, 1000000)

def parse_order(order_str: str) -> Tuple[str, float, float]:
    if '@' in order_str:
        quantity, price = order_str.split('@')
        return ('limit', float(price), float(quantity))
    else:
        return ('market', 0.0, float(order_str))

def parse_compound_orders(args: List[str]) -> List[Dict]:
    orders = []
    current_side = None

    i = 0
    while i < len(args):
        arg = args[i].lower()

        if arg in ['buy', 'sell']:
            current_side = 'bid' if arg == 'buy' else 'ask'
            i += 1
            continue

        if not current_side:
            raise ValueError("Must specify buy or sell before quantities")

        try:
            order_type, price, quantity = parse_order(args[i])
            orders.append({
                'id': generate_order_id(),
                'side': current_side,
                'type': order_type,
                'price': price,
                'quantity': quantity
            })
        except ValueError:
            raise ValueError(f"Invalid order format: {args[i]}")

        i += 1

    return orders

def run_cli(args=None):
    if args is None:
        args = sys.argv[1:]
    
    parser = create_parser()
    parsed_args = parser.parse_args(args)
    
    if not parsed_args.command:
        parser.print_help()
        return
    
    if parsed_args.command == 'init':
        print("The 'init' command should be run directly as 'atra init'.")
        return
    
    formatter = OrderBookFormatter(output_format=parsed_args.format)
    client = OrderBookClient(formatter, use_docker=parsed_args.docker and not parsed_args.local)
    
    try:
        if parsed_args.command == 'cancel':
            print(client.cancel_order(parsed_args.order_id))
        elif parsed_args.command == 'book':
            print(client.get_orderbook(parsed_args.depth))
        elif parsed_args.command == 'trades':
            print(client.get_trades(parsed_args.limit))
        elif parsed_args.command == 'orders':
            orders = parse_compound_orders(parsed_args.orders)
            for order in orders:
                print(client.place_order(order))
        elif parsed_args.command in ['buy', 'sell']:
            full_args = [parsed_args.command] + parsed_args.orders
            orders = parse_compound_orders(full_args)
            for order in orders:
                print(client.place_order(order))
    except ValueError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    except grpc.RpcError as e:
        print(f"Error: {e.details()}", file=sys.stderr)
        sys.exit(1)
