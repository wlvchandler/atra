defmodule AtraGateway.MixProject do
  use Mix.Project

  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_), do: ["lib/proto", "lib"]
  
  def project do
    [
      app: :atra_gateway,
      version: "0.1.0",
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      aliases: aliases(),
      elixirc_paths: elixirc_paths(Mix.env())  
      #compilers: [:proto] ++ Mix.compilers()
    ]
  end

  def application do
    [
      extra_applications: [:logger, :grpc, :cowlib],
      mod: {AtraGateway.Application, []}
    ]
  end

  defp deps do
    [
      {:grpc, "~> 0.7.0"},
      {:protobuf, "~> 0.12.0"},
      {:google_protos, "~> 0.3.0"},
      {:cowlib, "~> 2.12", override: true},
      {:poolboy, "~> 1.5"},
    ]
  end

  defp aliases do
    [
      "proto.compile": ["proto.compile"],
      compile: ["proto.compile", "compile"]
    ]
  end
end
