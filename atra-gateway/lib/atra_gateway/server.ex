defmodule AtraGateway.Server do
  use GRPC.Server, service: Orderbook.OrderBookService.Service
  require Logger

  defp parse_numeric(%{units: units, scale: scale}) do
    units / :math.pow(10, scale)
  end
  
  def place_order(request, _stream) do
    Logger.info("atra.gateway.request.place_order id:#{inspect(request.id)}")
    instrument_id = if request.instrument_id == 0, do: 1, else: request.instrument_id
    case AtraGateway.MatchingEngine.place_order(AtraGateway.Orders.new(
      parse_numeric(request.price),
      parse_numeric(request.quantity),
      atom_from_proto_side(request.side),
      atom_from_proto_order_type(request.order_type),
      instrument_id
    )) do
      {:ok, response} ->
        to_proto_response(response)
      {:error, reason} ->
	Logger.error("atra.gateway.error.request  request:place_order note:'#{inspect(reason)}'")
        raise GRPC.RPCError, status: :internal, message: "Internal error"
    end
  end

  def cancel_order(request, _stream) do
    Logger.info("atra.gateway.request.cancel_order id:#{inspect(request.order_id)} instrument:#{inspect(request.instrument_id)}")
    :poolboy.transaction(:grpc_pool, fn pid ->
      channel = AtraGateway.GrpcConnection.get_channel(pid)
      case Orderbook.OrderBookService.Stub.cancel_order(channel, request) do
        {:ok, response} -> response
        {:error, reason} ->
	  Logger.error("atra.gateway.error.request  request:cancel_order note:'#{inspect(reason)}'")
          raise GRPC.RPCError, status: :internal, message: "Internal error"
      end
    end)
  end

  def cancel_orders(request, _stream) do
    Logger.info("atra.gateway.request.cancel_orders batch:#{length(request.requests)}")
    :poolboy.transaction(:grpc_pool, fn pid ->
      channel = AtraGateway.GrpcConnection.get_channel(pid)
      case Orderbook.OrderBookService.Stub.cancel_orders(channel, request) do
        {:ok, response} -> response
        {:error, reason} ->
          Logger.error("atra.gateway.error.request request:cancel_orders note:'#{inspect(reason)}'")
          raise GRPC.RPCError, status: :internal, message: "Internal error"
      end
    end)
  end

  def get_order_book(request, _stream) do
    Logger.info("atra.gateway.request.get_order_book depth:#{inspect(request.depth)} instrument:#{inspect(request.instrument_id)}")
    :poolboy.transaction(:grpc_pool, fn pid ->
      channel = AtraGateway.GrpcConnection.get_channel(pid)
      case Orderbook.OrderBookService.Stub.get_order_book(channel, request) do
        {:ok, response} -> response
        {:error, reason} ->
	  Logger.error("atra.gateway.error.request  request:get_order_book note:'#{inspect(reason)}'")
          raise GRPC.RPCError, status: :internal, message: "Internal error"
      end
    end)
  end

  def get_order_status(request, _stream) do
    Logger.info("atra.gateway.request.order_status id:#{inspect(request.order_id)} instrument:#{inspect(request.instrument_id)}")
    :poolboy.transaction(:grpc_pool, fn pid ->
      channel = AtraGateway.GrpcConnection.get_channel(pid)
      case Orderbook.OrderBookService.Stub.get_order_status(channel, request) do
        {:ok, response} -> response
        {:error, reason} ->
	  Logger.error("atra.gateway.error.request  request:order_status note:'#{inspect(reason)}'")
          raise GRPC.RPCError, status: :internal, message: "Internal error"
      end
    end)
  end

  def get_trade_history(request, _stream) do
    Logger.info("atra.gateway.request.get_trade_history limit:#{inspect(request.limit)} instrument:#{inspect(request.instrument_id)}")
    :poolboy.transaction(:grpc_pool, fn pid ->
      channel = AtraGateway.GrpcConnection.get_channel(pid)
      case Orderbook.OrderBookService.Stub.get_trade_history(channel, request) do
        {:ok, response} -> response
        {:error, reason} ->
	  Logger.error("atra.gateway.error.request  request:trade_history note:'#{inspect(reason)}'")
          raise GRPC.RPCError, status: :internal, message: "Internal error"
      end
    end)
  end

  def stream_order_book(_request, _stream) do
    raise GRPC.RPCError, status: :unimplemented, message: "stream_order_book is not yet proxied by gateway"
  end

  def stream_trade_history(_request, _stream) do
    raise GRPC.RPCError, status: :unimplemented, message: "stream_trade_history is not yet proxied by gateway"
  end

  def place_orders(request, _stream) do
    Logger.info("atra.gateway.request.place_order batch:#{length(request.orders)}")
    orders =
      request.orders
      |> Enum.map(fn req ->
        instrument_id = if req.instrument_id == 0, do: 1, else: req.instrument_id
        AtraGateway.Orders.new(
          parse_numeric(req.price),
          parse_numeric(req.quantity),
          atom_from_proto_side(req.side),
          atom_from_proto_order_type(req.order_type),
          instrument_id
        )
      end)

    case AtraGateway.MatchingEngine.place_orders(orders) do
      {:ok, responses} ->
        %Orderbook.OrderBatchResponse{
          orders: Enum.map(responses, &to_proto_response/1)
        }

      {:error, reason} ->
        Logger.error("atra.gateway.error.request request:place_orders note:'#{inspect(reason)}'")
        raise GRPC.RPCError, status: :internal, message: "Internal error"
    end
  end

  # convert proto enums to atoms
  defp atom_from_proto_side(:BID), do: :bid
  defp atom_from_proto_side(:ASK), do: :ask

  defp atom_from_proto_order_type(:LIMIT), do: :limit
  defp atom_from_proto_order_type(:MARKET), do: :market

  defp to_proto_response(response) do
    %Orderbook.OrderResponse{
      id: response.id,
      price: to_proto_decimal(response.price),
      quantity: to_proto_decimal(response.quantity),
      remaining_quantity: to_proto_decimal(response.remaining_quantity),
      side: proto_side(response.side),
      order_type: proto_order_type(response.type),
      status: proto_status(response.status),
      instrument_id: response.instrument_id,
      sequence_number: response.sequence_number,
      ingress_timestamp_ns: response.ingress_timestamp_ns,
      idempotency_key: response.idempotency_key
    }
  end

  defp proto_side(:bid), do: :BID
  defp proto_side(:ask), do: :ASK

  defp proto_order_type(:limit), do: :LIMIT
  defp proto_order_type(:market), do: :MARKET

  defp proto_status(:pending), do: :PENDING
  defp proto_status(:partially_filled), do: :PARTIALLY_FILLED
  defp proto_status(:filled), do: :FILLED
  defp proto_status(:cancelled), do: :CANCELLED

  defp to_proto_decimal(value) do
    scaled = round(value * 100_000_000)
    %Orderbook.DecimalValue{units: scaled, scale: 8}
  end
end

