defmodule KobaraGateway.Orders do
  @moduledoc """
  Helper functions for working with orders and order responses.
  """

  @doc """
  Creates a new order map with required fields.
  """
  def new(price, quantity, side, type \\ :limit) do
    # we will want a better ID generation strategy
    id = System.unique_integer([:positive, :monotonic])

    %{
      id: id,
      price: price,
      quantity: quantity,
      side: side,
      type: type
    }
  end

  @doc """
  Converts a protobuf OrderResponse into a more usable map.
  """
  def from_proto(%Orderbook.OrderResponse{} = response) do
    %{
      id: response.id,
      price: String.to_float(response.price),
      quantity: String.to_float(response.quantity),
      remaining_quantity: String.to_float(response.remaining_quantity),
      side: atom_from_proto_side(response.side),
      type: atom_from_proto_order_type(response.order_type),
      status: atom_from_proto_status(response.status),
      timestamp: proto_timestamp_to_datetime(response.timestamp)
    }
  end

  defp atom_from_proto_side(:BID), do: :bid
  defp atom_from_proto_side(:ASK), do: :ask

  defp atom_from_proto_order_type(:LIMIT), do: :limit
  defp atom_from_proto_order_type(:MARKET), do: :market

  defp atom_from_proto_status(:PENDING), do: :pending
  defp atom_from_proto_status(:PARTIALLY_FILLED), do: :partially_filled
  defp atom_from_proto_status(:FILLED), do: :filled
  defp atom_from_proto_status(:CANCELLED), do: :cancelled

  defp proto_timestamp_to_datetime(%Google.Protobuf.Timestamp{seconds: seconds, nanos: nanos}) do
    DateTime.from_unix!(seconds, :second)
    |> DateTime.add(nanos, :nanosecond)
  end
end
