defmodule AtraGateway.Server do
  use GRPC.Server, service: Orderbook.OrderBookService.Service
  require Logger

  def place_order(request, _stream) do
    Logger.info("atra.gateway.request.place_order id:#{inspect(request.id)}")
    case AtraGateway.MatchingEngine.place_order(AtraGateway.Orders.new(
      String.to_float(request.price),
      String.to_float(request.quantity),
      atom_from_proto_side(request.side),
      atom_from_proto_order_type(request.order_type)
    )) do
      {:ok, _} ->
        # forwarding messages directly to the matcher isn't good practice. we'll use INQ later.
        :poolboy.transaction(:grpc_pool, fn pid ->
          channel = AtraGateway.GrpcConnection.get_channel(pid)
          case Orderbook.OrderBookService.Stub.place_order(channel, request) do
            {:ok, response} -> response
            {:error, reason} ->
	      Logger.error("atra.gateway.error.request  request:place_order note:'#{inspect(reason)}'")
              raise GRPC.RPCError, status: :internal, message: "Internal error"
          end
        end)
      {:error, reason} ->
	Logger.error("atra.gateway.error.request  request:place_order note:'#{inspect(reason)}'")
        raise GRPC.RPCError, status: :internal, message: "Internal error"
    end
  end

  def cancel_order(request, _stream) do
    Logger.info("atra.gateway.request.cancel_order id:#{inspect(request.order_id)}")
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

  def get_order_book(request, _stream) do
    Logger.info("atra.gateway.request.get_order_book depth:#{inspect(request.depth)}")
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
    Logger.info("atra.gateway.request.order_status id:#{inspect(request.order_id)}")
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
    Logger.info("atra.gateway.request.get_trade_history limit:#{inspect(request.limit)}")
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

  def place_orders(request, _stream) do
    Logger.info("atra.gateway.request.place_order batch:#{length(request.orders)}")
    :poolboy.transaction(:grpc_pool, fn pid ->
      channel = AtraGateway.GrpcConnection.get_channel(pid)
      case Orderbook.OrderBookService.Stub.place_orders(channel, request) do
        {:ok, response} -> response
        {:error, reason} ->
	  Logger.error("atra.gateway.error.request request:place_orders note:'#{inspect(reason)}'")
          raise GRPC.RPCError, status: :internal, message: "Internal error"
      end
    end)
  end

  # convert proto enums to atoms
  defp atom_from_proto_side(:BID), do: :bid
  defp atom_from_proto_side(:ASK), do: :ask

  defp atom_from_proto_order_type(:LIMIT), do: :limit
  defp atom_from_proto_order_type(:MARKET), do: :market
end
