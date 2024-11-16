defmodule OrderBookService.MixProject do
  use Mix.Project

  def project do
    [
      app: :orderbook_service,
      version: "0.1.0",
      elixir: "~> 1.14",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      compilers: [:rustler] ++ Mix.compilers(),
      rustler_crates: rustler_crates()
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {OrderBookService.Application, []}
    ]
  end

  defp deps do
    [
      {:grpc, "~> 0.5.0"},
      {:rustler, "~> 0.29.1"},
      {:protobuf, "~> 0.10.0"}
    ]
  end

  defp rustler_crates do
    [
      orderbook_core: [
        path: "native/orderbook_core",
        mode: (if Mix.env() == :prod, do: :release, else: :debug)
      ]
    ]
  end
end
