defmodule AtraGateway.Application do
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      AtraGateway.MatchingEngine
    ]

    opts = [strategy: :one_for_one, name: AtraGateway.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
