defmodule AtraGateway.Application do
  @moduledoc false
  use Application
  require Logger
  
  @impl true
  def start(_type, _args) do
    pool_opts = [
      name: {:local, :grpc_pool},
      worker_module: AtraGateway.GrpcConnection,
      size: 20,
      max_overflow: 10
    ]

    gateway_port = String.to_integer(System.get_env("GATEWAY_PORT", "50052"))
    
    children = [
      # Connection pool for GRPC
      :poolboy.child_spec(:grpc_pool, pool_opts, []),
      # Order processing pipeline supervisor
      {DynamicSupervisor, strategy: :one_for_one, name: AtraGateway.PipelineSupervisor},
      AtraGateway.MatchingEngine,
      {GRPC.Server.Supervisor, endpoint: AtraGateway.Server, port: gateway_port}
    ]
    
    Logger.info("atra.gateway.start server:grpc port:#{gateway_port}")
    
    opts = [strategy: :one_for_one, name: AtraGateway.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
