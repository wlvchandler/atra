defmodule OrderBookService.OrderBook do
  use GenServer
  alias OrderBookService.Native
  alias OrderBookService.Types.Order

  def start_link(_opts \\ []) do
    GenServer.start_link(__MODULE__, :ok, name: __MODULE__)
  end

  def init(:ok) do
    {:ok, Native.new()}
  end

  # Client API
  def place_order(%Order{} = order) do
    GenServer.call(__MODULE__, {:place_order, order})
  end

  def get_order_book(depth) when is_integer(depth) and depth > 0 do
    GenServer.call(__MODULE__, {:get_order_book, depth})
  end

  def get_order_status(order_id) when is_integer(order_id) do
    GenServer.call(__MODULE__, {:get_order_status, order_id})
  end

  # Server callbacks
  def handle_call({:place_order, order}, _from, state) do
    try do
      result = Native.place_order(state, order)
      {:reply, {:ok, result}, state}
    rescue
      e -> {:reply, {:error, Exception.message(e)}, state}
    end
  end

  def handle_call({:get_order_book, depth}, _from, state) do
    try do
      result = Native.get_order_book(state, depth)
      {:reply, {:ok, result}, state}
    rescue
      e -> {:reply, {:error, Exception.message(e)}, state}
    end
  end

  def handle_call({:get_order_status, order_id}, _from, state) do
    try do
      case Native.get_order_status(state, order_id) do
        nil -> {:reply, {:error, :not_found}, state}
        order -> {:reply, {:ok, order}, state}
      end
    rescue
      e -> {:reply, {:error, Exception.message(e)}, state}
    end
  end
end
