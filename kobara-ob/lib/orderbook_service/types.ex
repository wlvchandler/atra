defmodule OrderBookService.Types do
  defmodule Order do
    @enforce_keys [:id, :price, :quantity, :side, :order_type]
    defstruct [:id, :price, :quantity, :side, :order_type, remaining_quantity: nil, status: "pending"]
  end
end
