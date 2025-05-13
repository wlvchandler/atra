import json
from decimal import Decimal
from datetime import datetime

class OrderBookFormatter:
    def __init__(self, output_format='csv'):
        """
        output_format: 'csv' (default), 'json', or 'pretty'
        """
        self.output_format = output_format

    def format_order_response(self, response):
        if self.output_format == 'json':
            return json.dumps({
                "id": response.id,
                "status": response.status
            })
        elif self.output_format == 'pretty':
            return f"Order placed: ID={response.id}, Status={response.status}"
        else:  # csv
            return f"order,{response.id},{response.status}"

    def format_cancel_response(self, response):
        if self.output_format == 'json':
            return json.dumps({
                "id": response.id,
                "status": response.status
            })
        elif self.output_format == 'pretty':
            return f"Order cancelled: ID={response.id}, Status={response.status}"
        else:  # csv
            return f"cancel,{response.id},{response.status}"

    def format_orderbook(self, response, depth):
        if self.output_format == 'json':
            return json.dumps({
                "bids": [{"price": level.price, "quantity": level.quantity}
                        for level in sorted(response.bids, key=lambda x: Decimal(x.price), reverse=True)],
                "asks": [{"price": level.price, "quantity": level.quantity}
                        for level in sorted(response.asks, key=lambda x: Decimal(x.price))]
            })
        elif self.output_format == 'pretty':
            output = [
                f"\natraOB (Max depth {depth}):",
                f"{'Price':>10} {'Quantity':>10} {'Side':>6}",
                "-" * 30
            ]

            for level in sorted(response.bids, key=lambda x: Decimal(x.price), reverse=True):
                price = Decimal(level.price).quantize(Decimal('0.01'))
                quantity = Decimal(level.quantity).quantize(Decimal('0.01'))
                output.append(f"{price:>10} {quantity:>10} {'BID':>6}")

            output.append("-" * 30)

            for level in sorted(response.asks, key=lambda x: Decimal(x.price)):
                price = Decimal(level.price).quantize(Decimal('0.01'))
                quantity = Decimal(level.quantity).quantize(Decimal('0.01'))
                output.append(f"{price:>10} {quantity:>10} {'ASK':>6}")

            return "\n".join(output)
        else:  # csv format
            lines = []
            # {type,side,price,quantity}
            for level in sorted(response.bids, key=lambda x: Decimal(x.price), reverse=True):
                lines.append(f"level,bid,{level.price},{level.quantity}")
            for level in sorted(response.asks, key=lambda x: Decimal(x.price)):
                lines.append(f"level,ask,{level.price},{level.quantity}")
            return "\n".join(lines)

    def format_trades(self, response, limit):
        if self.output_format == 'json':
            return json.dumps({
                "trades": [{
                    "timestamp": trade.timestamp.seconds + trade.timestamp.nanos / 1e9,
                    "price": trade.price,
                    "quantity": trade.quantity,
                    "side": "BID" if trade.side == 0 else "ASK",
                    "maker_order_id": trade.maker_order_id,
                    "taker_order_id": trade.taker_order_id
                } for trade in response.trades]
            })
        elif self.output_format == 'pretty':
            output = [
                f"\nRecent Trades (Last {limit}):",
                f"{'Time':>19} {'Price':>10} {'Quantity':>10} {'Side':>6} {'Maker ID':>10} {'Taker ID':>10}",
                "-" * 70
            ]

            for trade in response.trades:
                ts = datetime.fromtimestamp(trade.timestamp.seconds + trade.timestamp.nanos / 1e9)
                price = Decimal(trade.price).quantize(Decimal('0.01'))
                quantity = Decimal(trade.quantity).quantize(Decimal('0.01'))
                side = "BID" if trade.side == 0 else "ASK"

                output.append(
                    f"{ts.strftime('%Y-%m-%d %H:%M:%S'):>19} "
                    f"{price:>10} "
                    f"{quantity:>10} "
                    f"{side:>6} "
                    f"{trade.maker_order_id:>10} "
                    f"{trade.taker_order_id:>10}"
                )
            return "\n".join(output)
        else:  # csv
            # {type,timestamp,price,quantity,side,maker_id,taker_id}
            return "\n".join(
                f"trade,{trade.timestamp.seconds + trade.timestamp.nanos / 1e9},"
                f"{trade.price},{trade.quantity},"
                f"{'bid' if trade.side == 0 else 'ask'},"
                f"{trade.maker_order_id},{trade.taker_order_id}"
                for trade in response.trades
            )
