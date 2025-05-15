defmodule AtraGateway.GrpcServerWrapper do
  use GenServer
  require Logger

  def start_link(opts) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  def init(opts) do
    port = Keyword.get(opts, :port, 50052)
    
    case GRPC.Server.start(AtraGateway.Server, port) do
      {:ok, server, actual_port} ->
        Logger.info("gRPC server started on port #{actual_port}")
        Process.link(server)
        {:ok, %{server: server, port: actual_port}}
      {:error, reason} ->
        {:stop, reason}
    end
  end
end
