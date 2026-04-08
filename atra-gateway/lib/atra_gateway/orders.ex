defmodule AtraGateway.Orders do
  @moduledoc """
  Helper functions for working with orders and order responses.
  """

  @doc """
  Creates a new order map with required fields.
  """
  def new(price, quantity, side, type \\ :limit, instrument_id \\ 1) do
    # we will want a better ID generation strategy
    id = System.unique_integer([:positive, :monotonic])

    %{
      id: id,
      price: price,
      quantity: quantity,
      instrument_id: instrument_id,
      side: side,
      type: type
    }
  end

  @doc """
  Converts a protobuf OrderResponse into a more usable map.
  """
  def from_proto(response) do
    %{
      id: response.id,
      price: decimal_from_proto(response.price),
      quantity: decimal_from_proto(response.quantity),
      remaining_quantity: decimal_from_proto(response.remaining_quantity),
      side: atom_from_proto_side(response.side),
      type: atom_from_proto_order_type(response.order_type),
      status: atom_from_proto_status(response.status),
      timestamp: proto_timestamp_to_datetime(response.timestamp),
      instrument_id: response.instrument_id,
      sequence_number: response.sequence_number,
      ingress_timestamp_ns: response.ingress_timestamp_ns,
      idempotency_key: response.idempotency_key
    }
  end

  defp atom_from_proto_side(:BID), do: :bid
  defp atom_from_proto_side(:ASK), do: :ask
  defp atom_from_proto_side(1), do: :bid
  defp atom_from_proto_side(2), do: :ask

  defp atom_from_proto_order_type(:LIMIT), do: :limit
  defp atom_from_proto_order_type(:MARKET), do: :market
  defp atom_from_proto_order_type(1), do: :limit
  defp atom_from_proto_order_type(2), do: :market

  defp atom_from_proto_status(:PENDING), do: :pending
  defp atom_from_proto_status(:PARTIALLY_FILLED), do: :partially_filled
  defp atom_from_proto_status(:FILLED), do: :filled
  defp atom_from_proto_status(:CANCELLED), do: :cancelled
  defp atom_from_proto_status(1), do: :pending
  defp atom_from_proto_status(2), do: :partially_filled
  defp atom_from_proto_status(3), do: :filled
  defp atom_from_proto_status(4), do: :cancelled

  defp decimal_from_proto(nil), do: 0.0
  defp decimal_from_proto(%{units: units, scale: scale}) do
    units / :math.pow(10, scale)
  end

  defp proto_timestamp_to_datetime(%{seconds: seconds, nanos: nanos}) do
    DateTime.from_unix!(seconds, :second)
    |> DateTime.add(nanos, :nanosecond)
  end
end
