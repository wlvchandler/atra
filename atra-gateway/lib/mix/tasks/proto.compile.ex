defmodule Mix.Tasks.Proto.Compile do
  use Mix.Task

  @shortdoc "Compiles protocol buffer definitions"
  def run(_) do
    if Mix.env() != :test do
      proto_path = "../atra-proto/proto"
      proto_file = "#{proto_path}/orderbook.proto"

      if !File.exists?(proto_file) do
        Mix.raise("Cannot find orderbook.proto at #{proto_file}")
      end

      # Create output directory
      File.mkdir_p!("lib/proto")

      case System.cmd("protoc", [
        "--elixir_out=plugins=grpc:./lib/proto",
        "--proto_path=#{proto_path}",
        "--proto_path=#{:code.priv_dir(:google_protos)}/protos",
        proto_file
      ]) do
        {_, 0} -> Mix.shell().info("Protos compiled successfully!")
        {error, _} -> Mix.raise("Failed to compile protos: #{error}")
      end
    end
  end
end
