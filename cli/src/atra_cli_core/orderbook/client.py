import os
import grpc
from decimal import Decimal
from typing import Optional, Dict

from atra_cli_core.generated.orderbook_pb2 import (
    OrderRequest, GetOrderBookRequest, GetOrderStatusRequest,
    GetTradeHistoryRequest, Side, OrderType, CancelOrderRequest
)
from atra_cli_core.generated.orderbook_pb2_grpc import OrderBookServiceStub

from atra_cli_core.orderbook.formatter import OrderBookFormatter

class OrderBookClient:
    def __init__(self, formatter: OrderBookFormatter, use_docker: bool = False):
        self.formatter = formatter
        self.use_docker = use_docker
        self._stub: Optional[OrderBookServiceStub] = None

    @property
    def stub(self):
        if self._stub is None:
            self._stub = self._connect()
        return self._stub

    def _connect(self):
        if self.use_docker:
            host = 'orderbook'
        else:
            host = os.getenv('atra_OB_HOST', '127.0.0.1')
        port = os.getenv('atra_OB_PORT', '50051')
        channel = grpc.insecure_channel(f'{host}:{port}')
        return OrderBookServiceStub(channel)

    def place_order(self, order: Dict):
        request = OrderRequest(
            id=order['id'],
            price=str(Decimal(order['price'])),
            quantity=str(Decimal(order['quantity'])),
            side=Side.BID if order['side'].upper() == "BID" else Side.ASK,
            order_type=OrderType.LIMIT if order['type'].upper() == "LIMIT" else OrderType.MARKET
        )
        response = self.stub.place_order(request)
        return self.formatter.format_order_response(response)

    def cancel_order(self, order_id: int):
        request = CancelOrderRequest(order_id=order_id)
        response = self.stub.cancel_order(request)
        return self.formatter.format_cancel_response(response)

    def get_orderbook(self, depth: int):
        request = GetOrderBookRequest(depth=depth)
        response = self.stub.get_order_book(request)
        return self.formatter.format_orderbook(response, depth)

    def get_trades(self, limit: int):
        request = GetTradeHistoryRequest(limit=limit)
        response = self.stub.get_trade_history(request)
        return self.formatter.format_trades(response, limit)
