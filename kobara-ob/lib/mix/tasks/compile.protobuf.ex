defmodule Mix.Tasks.Compile.Protobuf do
  use Mix.Task

  @shortdoc "Compiles protobuf definitions"
  def run(_) do
    Mix.shell().info("Creating proto output directory...")
    File.mkdir_p!("lib/proto")

    args = [
      "-I=.",
      "--elixir_out=plugins=grpc:./lib/proto",
      "proto/orderbook.proto"
    ]

    Mix.shell().info("Running protoc with args: #{Enum.join(args, " ")}")
    {output, status} = System.cmd("protoc", args, stderr_to_stdout: true)

    if status == 0 do
      Mix.shell().info("Protobuf compilation successful!")
      Mix.shell().info(output)
    else
      Mix.shell().error("Protobuf compilation failed:")
      Mix.shell().error(output)
      exit({:shutdown, 1})
    end
  end
end
