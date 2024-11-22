defmodule AtraGatewayTest do
  use ExUnit.Case
  doctest AtraGateway

  test "greets the world" do
    assert AtraGateway.hello() == :world
  end
end
