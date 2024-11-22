defmodule AtraGateway.MatchingEngine do
  use GenServer
  require Logger
  alias AtraGateway.Orders

  def start_link(_) do
    GenServer.start_link(__MODULE__, nil, name: __MODULE__)
  end

  def init(_) do
    case GRPC.Stub.connect("localhost:50051") do
      {:ok, channel} ->
        Logger.info("Connected to matching engine")
        {:ok, %{channel: channel}}
      {:error, reason} ->
        Logger.error("Failed to connect to matching engine: #{inspect(reason)}")
        {:stop, reason}
    end
  end

  def place_order(order_params) do
    GenServer.call(__MODULE__, {:place_order, order_params})
  end
  
  def handle_call({:place_order, params}, _from, state) do
    Logger.debug("Placing order: #{inspect(params)}")
    
    # Convert the params to protobuf format
    proto_request = %Orderbook.OrderRequest{
      id: params.id,
      price: to_string(params.price),
      quantity: to_string(params.quantity),
      side: if(params.side == :bid, do: 0, else: 1),
      order_type: if(params.type == :limit, do: 0, else: 1)
    }

    case Orderbook.OrderBookService.Stub.place_order(state.channel, proto_request) do
      {:ok, proto_response} ->
        response = Orders.from_proto(proto_response)
        Logger.debug("Order placed successfully: #{inspect(response)}")
        {:reply, {:ok, response}, state}
      {:error, reason} = error ->
        Logger.error("Order placement failed: #{inspect(reason)}")
        {:reply, error, state}
    end
  end
end
