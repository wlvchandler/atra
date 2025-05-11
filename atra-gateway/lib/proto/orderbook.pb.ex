defmodule Orderbook.Side do
  @moduledoc false

  use Protobuf, enum: true, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :BID, 0
  field :ASK, 1
end

defmodule Orderbook.OrderType do
  @moduledoc false

  use Protobuf, enum: true, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :LIMIT, 0
  field :MARKET, 1
end

defmodule Orderbook.OrderStatus do
  @moduledoc false

  use Protobuf, enum: true, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :PENDING, 0
  field :PARTIALLY_FILLED, 1
  field :FILLED, 2
  field :CANCELLED, 3
end

defmodule Orderbook.OrderRequest do
  @moduledoc false

  use Protobuf, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :id, 1, type: :uint64
  field :price, 2, type: :string
  field :quantity, 3, type: :string
  field :side, 4, type: Orderbook.Side, enum: true
  field :order_type, 5, type: Orderbook.OrderType, json_name: "orderType", enum: true
end

defmodule Orderbook.OrderResponse do
  @moduledoc false

  use Protobuf, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :id, 1, type: :uint64
  field :price, 2, type: :string
  field :quantity, 3, type: :string
  field :remaining_quantity, 4, type: :string, json_name: "remainingQuantity"
  field :side, 5, type: Orderbook.Side, enum: true
  field :order_type, 6, type: Orderbook.OrderType, json_name: "orderType", enum: true
  field :status, 7, type: Orderbook.OrderStatus, enum: true
  field :timestamp, 8, type: Google.Protobuf.Timestamp
end

defmodule Orderbook.CancelOrderRequest do
  @moduledoc false

  use Protobuf, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :order_id, 1, type: :uint64, json_name: "orderId"
end

defmodule Orderbook.GetOrderBookRequest do
  @moduledoc false

  use Protobuf, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :depth, 1, type: :uint32
end

defmodule Orderbook.OrderBookLevel do
  @moduledoc false

  use Protobuf, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :price, 1, type: :string
  field :quantity, 2, type: :string
end

defmodule Orderbook.OrderBookResponse do
  @moduledoc false

  use Protobuf, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :bids, 1, repeated: true, type: Orderbook.OrderBookLevel
  field :asks, 2, repeated: true, type: Orderbook.OrderBookLevel
end

defmodule Orderbook.GetOrderStatusRequest do
  @moduledoc false

  use Protobuf, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :order_id, 1, type: :uint64, json_name: "orderId"
end

defmodule Orderbook.GetTradeHistoryRequest do
  @moduledoc false

  use Protobuf, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :limit, 1, type: :uint32
end

defmodule Orderbook.Trade do
  @moduledoc false

  use Protobuf, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :maker_order_id, 1, type: :uint64, json_name: "makerOrderId"
  field :taker_order_id, 2, type: :uint64, json_name: "takerOrderId"
  field :price, 3, type: :string
  field :quantity, 4, type: :string
  field :side, 5, type: Orderbook.Side, enum: true
  field :timestamp, 6, type: Google.Protobuf.Timestamp
end

defmodule Orderbook.TradeHistoryResponse do
  @moduledoc false

  use Protobuf, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :trades, 1, repeated: true, type: Orderbook.Trade
end

defmodule Orderbook.OrderBatchRequest do
  @moduledoc false

  use Protobuf, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :orders, 1, repeated: true, type: Orderbook.OrderRequest
end

defmodule Orderbook.OrderBatchResponse do
  @moduledoc false

  use Protobuf, syntax: :proto3, protoc_gen_elixir_version: "0.13.0"

  field :orders, 1, repeated: true, type: Orderbook.OrderResponse
end

defmodule Orderbook.OrderBookService.Service do
  @moduledoc false

  use GRPC.Service, name: "orderbook.OrderBookService", protoc_gen_elixir_version: "0.13.0"

  rpc :place_order, Orderbook.OrderRequest, Orderbook.OrderResponse

  rpc :cancel_order, Orderbook.CancelOrderRequest, Orderbook.OrderResponse

  rpc :get_order_book, Orderbook.GetOrderBookRequest, Orderbook.OrderBookResponse

  rpc :get_order_status, Orderbook.GetOrderStatusRequest, Orderbook.OrderResponse

  rpc :get_trade_history, Orderbook.GetTradeHistoryRequest, Orderbook.TradeHistoryResponse

  rpc :place_orders, Orderbook.OrderBatchRequest, Orderbook.OrderBatchResponse
end

defmodule Orderbook.OrderBookService.Stub do
  @moduledoc false

  use GRPC.Stub, service: Orderbook.OrderBookService.Service
end