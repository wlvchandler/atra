defmodule AtraGateway.MatchingEngine do
  use GenServer
  require Logger
  alias AtraGateway.Orders
  
  def start_link(_) do
    GenServer.start_link(__MODULE__, nil, name: __MODULE__)
  end
  
  def init(_) do
    :ets.new(:order_buffer, [:named_table, :public, write_concurrency: true])
    schedule_batch_processing()
    {:ok, %{batch_size: 1000, batch_interval_ms: 100}}
  end

  def place_order(order_params) do
    # put order in ets buffer
    true = :ets.insert(:order_buffer, {System.monotonic_time(), order_params})
    {:ok, %{status: :accepted}}
  end

  defp schedule_batch_processing do
    Process.send_after(self(), :process_batch, 100)
  end

  def handle_info(:process_batch, %{batch_size: batch_size} = state) do
    # take <= batch_size orders from the buffer
    orders = :ets.take(:order_buffer, batch_size)
    
    if orders != [] do
      orders
      |> Enum.sort_by(&elem(&1, 0)) # sort by timestamp
      |> Enum.map(&elem(&1, 1))     # take just the order params
      |> Enum.chunk_every(50)        # process in smaller chunks
      |> Task.async_stream(&process_order_chunk/1, 
          max_concurrency: 10,
          timeout: 5000)
      |> Stream.run()
    end

    schedule_batch_processing()
    {:noreply, state}
  end

  defp process_order_chunk(orders) do
    :poolboy.transaction(:grpc_pool, fn pid ->
      channel = AtraGateway.GrpcConnection.get_channel(pid)
      
      # Convert orders to proto format
      proto_requests = Enum.map(orders, fn params ->
	%Orderbook.OrderRequest{
          id: params.id,
          price: to_string(params.price),
          quantity: to_string(params.quantity),
          side: proto_side(params.side),
          order_type: proto_order_type(params.type)} end)
      
      request = %Orderbook.OrderBatchRequest{
	orders: proto_requests
      }
      
      case Orderbook.OrderBookService.Stub.place_orders(channel, request) do
	{:ok, %Orderbook.OrderBatchResponse{orders: responses}} ->
          Enum.map(responses, &Orders.from_proto/1)
	{:error, reason} ->
          Logger.error("Batch order placement failed: #{inspect(reason)}")
          {:error, reason}
      end
    end)
  end

  defp proto_side(:bid), do: :BID
  defp proto_side(:ask), do: :ASK

  defp proto_order_type(:limit), do: :LIMIT
  defp proto_order_type(:market), do: :MARKET

end
