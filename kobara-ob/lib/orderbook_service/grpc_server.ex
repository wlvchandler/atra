defmodule OrderBookService.GrpcServer do
  use GRPC.Server, service: Orderbook.OrderBookService.Service

  alias Orderbook.{
    Order,
    OrderResponse,
    OrderBookRequest,
    OrderBookResponse,
    OrderStatusRequest,
    OrderStatusResponse,
    PriceLevel
  }

  alias OrderBookService.OrderBook
  alias OrderBookService.Types

  @spec place_order(Order.t(), GRPC.Server.Stream.t()) :: OrderResponse.t()
  def place_order(request, _stream) do
    # Create our internal order type
    internal_order = %Types.Order{
      id: request.id,
      price: request.price,
      quantity: request.quantity,
      side: request.side,
      order_type: request.order_type,
      remaining_quantity: request.quantity,
      status: "pending"
    }

    case OrderBook.place_order(internal_order) do
      {:ok, order} ->
        OrderResponse.new(
          id: order.id,
          status: order.status
        )
      {:error, reason} ->
        OrderResponse.new(
          id: request.id,
          status: "error: #{reason}"
        )
    end
  end

  @spec get_order_book(OrderBookRequest.t(), GRPC.Server.Stream.t()) :: OrderBookResponse.t()
  def get_order_book(%OrderBookRequest{depth: depth}, _stream) do
    case OrderBook.get_order_book(depth) do
      {:ok, {bids, asks}} ->
        OrderBookResponse.new(
          bids: Enum.map(bids, fn {price, quantity} ->
            PriceLevel.new(price: to_string(price), quantity: to_string(quantity))
          end),
          asks: Enum.map(asks, fn {price, quantity} ->
            PriceLevel.new(price: to_string(price), quantity: to_string(quantity))
          end)
        )
      {:error, _reason} ->
        OrderBookResponse.new(bids: [], asks: [])
    end
  end

  @spec get_order_status(OrderStatusRequest.t(), GRPC.Server.Stream.t()) :: OrderStatusResponse.t()
  def get_order_status(%OrderStatusRequest{order_id: order_id}, _stream) do
    case OrderBook.get_order_status(order_id) do
      {:ok, order} ->
        OrderStatusResponse.new(
          id: order.id,
          status: order.status,
          filled_quantity: to_string(order.quantity - order.remaining_quantity),
          remaining_quantity: to_string(order.remaining_quantity)
        )
      {:error, _reason} ->
        OrderStatusResponse.new(
          id: order_id,
          status: "not_found",
          filled_quantity: "0",
          remaining_quantity: "0"
        )
    end
  end
end
