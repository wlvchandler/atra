defmodule Mix.Tasks.CompileProtos do
  use Mix.Task

  @shortdoc "Compiles protocol buffer definitions"
  def run(_) do
    proto_path = "../kobara-proto/proto"
    proto_file = "#{proto_path}/orderbook.proto"

    if !File.exists?(proto_file) do
      Mix.raise("Cannot find orderbook.proto at #{proto_file}")
    end

    File.mkdir_p!("priv/protos/google/protobuf")
    timestamp_proto = "priv/protos/google/protobuf/timestamp.proto"

    if !File.exists?(timestamp_proto) do
      Mix.shell().info("Downloading timestamp.proto...")
      
      System.cmd("curl", [
        "-o",
        timestamp_proto,
        "https://raw.githubusercontent.com/protocolbuffers/protobuf/main/src/google/protobuf/timestamp.proto"
      ])
    end

    case System.cmd("protoc", [
      "--elixir_out=plugins=grpc:./lib",
      "--proto_path=#{proto_path}",
      "--proto_path=priv/protos",
      proto_file
    ]) do
      {_, 0} -> Mix.shell().info("Protos compiled successfully!")
      {error, _} -> Mix.raise("Failed to compile protos: #{error}")
    end
  end
end
