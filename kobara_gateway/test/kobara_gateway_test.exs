defmodule KobaraGatewayTest do
  use ExUnit.Case
  doctest KobaraGateway

  test "greets the world" do
    assert KobaraGateway.hello() == :world
  end
end
