defmodule AtraGateway.MatchingEngine do
  use GenServer
  require Logger
  alias AtraGateway.Orders

  def start_link(_) do
    GenServer.start_link(__MODULE__, nil, name: __MODULE__)
  end

  def init(_) do
    lane_count = lane_count()
    {:ok, %{next_seq: %{}, lane_count: lane_count}}
  end

  def place_order(order_params) do
    GenServer.call(__MODULE__, {:place_order, order_params}, 5_000)
  end

  def place_orders(order_params_list) do
    GenServer.call(__MODULE__, {:place_orders, order_params_list}, 10_000)
  end

  def handle_call({:place_order, order_params}, _from, state) do
    with {:ok, sequenced, next_state} <- sequence_order(order_params, state),
         {:ok, response} <- grpc_place_order(sequenced) do
      {:reply, {:ok, response}, next_state}
    else
      {:error, reason} ->
        Logger.error("order placement failed: #{inspect(reason)}")
        {:reply, {:error, reason}, state}
    end
  end

  def handle_call({:place_orders, order_params_list}, _from, state) do
    {responses, new_state} =
      Enum.reduce(order_params_list, {[], state}, fn order_params, {acc, st} ->
        case sequence_order(order_params, st) do
          {:ok, sequenced, next_state} ->
            case grpc_place_order(sequenced) do
              {:ok, response} -> {[response | acc], next_state}
              {:error, _} -> {acc, st}
            end

          {:error, _} ->
            {acc, st}
        end
      end)

    {:reply, {:ok, Enum.reverse(responses)}, new_state}
  end

  defp sequence_order(order_params, state) do
    instrument_id = Map.fetch!(order_params, :instrument_id)
    next_seq = Map.get(state.next_seq, instrument_id, 1)
    provided_seq = Map.get(order_params, :sequence_number, nil)
    strict? = strict_sequence_validation?()

    sequence_number =
      cond do
        is_nil(provided_seq) -> next_seq
        strict? and provided_seq != next_seq -> :out_of_order
        true -> provided_seq
      end

    if sequence_number == :out_of_order do
      {:error, :out_of_order_sequence}
    else
      ingress_timestamp_ns = System.os_time(:nanosecond)
      order =
        order_params
        |> Map.put(:sequence_number, sequence_number)
        |> Map.put(:ingress_timestamp_ns, ingress_timestamp_ns)
        |> Map.put(:lane_id, lane_for_instrument(instrument_id, state.lane_count))

      next_expected = max(next_seq, sequence_number + 1)
      next_state = put_in(state.next_seq[instrument_id], next_expected)
      {:ok, order, next_state}
    end
  end

  defp grpc_place_order(params) do
    :poolboy.transaction(:grpc_pool, fn pid ->
      channel = AtraGateway.GrpcConnection.get_channel(pid)

      request = %Orderbook.OrderRequest{
          id: params.id,
          price: to_proto_decimal(params.price),
          quantity: to_proto_decimal(params.quantity),
          side: proto_side(params.side),
          order_type: proto_order_type(params.type),
          instrument_id: params.instrument_id,
          sequence_number: params.sequence_number,
          ingress_timestamp_ns: params.ingress_timestamp_ns,
          idempotency_key: Map.get(params, :idempotency_key)
      }

      case Orderbook.OrderBookService.Stub.place_order(channel, request) do
        {:ok, response} -> {:ok, Orders.from_proto(response)}
        {:error, reason} -> {:error, reason}
      end
    end)
  end

  defp lane_count do
    System.get_env("ATRA_GATEWAY_LANE_COUNT", "4") |> String.to_integer()
  end

  defp strict_sequence_validation? do
    System.get_env("ATRA_GATEWAY_STRICT_SEQUENCE_VALIDATION", "false") in ["1", "true", "TRUE", "yes", "YES"]
  end

  defp lane_for_instrument(instrument_id, lane_count), do: rem(instrument_id, lane_count)

  defp proto_side(:bid), do: :BID
  defp proto_side(:ask), do: :ASK

  defp proto_order_type(:limit), do: :LIMIT
  defp proto_order_type(:market), do: :MARKET

  defp to_proto_decimal(value) do
    scaled = round(value * 100_000_000)
    %Orderbook.DecimalValue{units: scaled, scale: 8}
  end
end
