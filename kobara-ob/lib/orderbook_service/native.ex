defmodule OrderBookService.Native do
  use Rustler,
    otp_app: :orderbook_service,
    crate: "orderbook_core"

  # These functions are implemented in Rust
  def new(), do: :erlang.nif_error(:nif_not_loaded)
  def place_order(_book, _order), do: :erlang.nif_error(:nif_not_loaded)
  def get_order_book(_book, _depth), do: :erlang.nif_error(:nif_not_loaded)
  def get_order_status(_book, _order_id), do: :erlang.nif_error(:nif_not_loaded)
end
