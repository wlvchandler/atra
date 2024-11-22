defmodule AtraGateway.MixProject do
  use Mix.Project

  def project do
    [
      app: :atra_gateway,
      version: "0.1.0",
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      aliases: aliases()      
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {AtraGateway.Application, []}
    ]
  end

  defp deps do
    [
      {:grpc, "~> 0.7.0"},
      {:protobuf, "~> 0.12.0"},
      {:google_protos, "~> 0.3.0"},
    ]
  end

  defp aliases do
    [
      "compile.protos": ["compile_protos"],
      compile: ["compile.protos", "compile"]
    ]
  end
end
