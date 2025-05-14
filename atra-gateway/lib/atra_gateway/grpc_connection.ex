defmodule AtraGateway.GrpcConnection do
  use GenServer
  require Logger

  def start_link(_) do
    GenServer.start_link(__MODULE__, nil)
  end

  def init(_) do
    matcher_host = System.get_env("MATCHER_HOST", "localhost")
    matcher_port = System.get_env("MATCHER_PORT", "50051")

    connection_string = "#{matcher_host}:#{matcher_port}"
    Logger.info("Connecting to matcher at #{connection_string}")
    
    case GRPC.Stub.connect(connection_string, pool_size: 1) do
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
