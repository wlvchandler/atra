defmodule AtraGateway.Application do
  @moduledoc false
  use Application

  @impl true
  def start(_type, _args) do
    pool_opts = [
      name: {:local, :grpc_pool},
      worker_module: AtraGateway.GrpcConnection,
      size: 20,
      max_overflow: 10
    ]

    children = [
      # Connection pool for GRPC
      :poolboy.child_spec(:grpc_pool, pool_opts, []),
      # Order processing pipeline supervisor
      {DynamicSupervisor, strategy: :one_for_one, name: AtraGateway.PipelineSupervisor},
      AtraGateway.MatchingEngine
    ]

    opts = [strategy: :one_for_one, name: AtraGateway.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
