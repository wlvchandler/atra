defmodule OrderBookService.Application do
  use Application

  def start(_type, _args) do
    children = [
      OrderBookService.OrderBook,
      {GRPC.Server.Supervisor,
        endpoint: OrderBookService.GrpcServer,
        port: 50051,
        start_server: true
      }
    ]

    opts = [strategy: :one_for_one, name: OrderBookService.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
