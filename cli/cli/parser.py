import argparse

def create_parser():
    parser = argparse.ArgumentParser(description='atraCLI - Order Book Interface')

    parser.add_argument('--format', choices=['csv', 'json', 'pretty'], default='csv',
                       help='Output format (default: csv)')
    parser.add_argument('--docker', action='store_true', help='Use Docker for execution')
    parser.add_argument('--local', action='store_true', help='Force local execution')

    subparsers = parser.add_subparsers(dest='command')

    init_parser = subparsers.add_parser('init', help='Initialize the environment')

    book_parser = subparsers.add_parser('book', help='Show order book')
    book_parser.add_argument('depth', type=int, nargs='?', default=10)

    trades_parser = subparsers.add_parser('trades', help='Show recent trades')
    trades_parser.add_argument('limit', type=int, nargs='?', default=10)

    orders_parser = subparsers.add_parser('orders', help='Place multiple orders')
    orders_parser.add_argument('orders', nargs='+', help='Orders in format: buy/sell QUANTITY[@PRICE] ...')

    buy_parser = subparsers.add_parser('buy', help='Place buy orders')
    buy_parser.add_argument('orders', nargs='+', help='Orders in format: QUANTITY[@PRICE] ...')

    sell_parser = subparsers.add_parser('sell', help='Place sell orders')
    sell_parser.add_argument('orders', nargs='+', help='Orders in format: QUANTITY[@PRICE] ...')

    cancel_parser = subparsers.add_parser('cancel', help='Cancel an existing order')
    cancel_parser.add_argument('order_id', type=int, help='ID of the order to cancel')

    return parser
