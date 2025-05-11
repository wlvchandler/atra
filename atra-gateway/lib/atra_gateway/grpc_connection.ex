defmodule AtraGateway.GrpcConnection do
  use GenServer
  require Logger

  def start_link(_) do
    GenServer.start_link(__MODULE__, nil)
  end

  def init(_) do
    case GRPC.Stub.connect("localhost:50051", pool_size: 1) do
      {:ok, channel} ->
        Logger.info("Connected to matching engine")
        {:ok, %{channel: channel}}
      {:error, reason} ->
        Logger.error("Failed to connect to matching engine: #{inspect(reason)}")
        {:stop, reason}
    end
  end

  def get_channel(pid) do
    GenServer.call(pid, :get_channel)
  end

  def handle_call(:get_channel, _from, %{channel: channel} = state) do
    {:reply, channel, state}
  end
end
