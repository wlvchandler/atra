# AtraGateway

Real-time Elixir gateway service for a Atra system. Provides real-time connectivity and state management for backend engine.

## build notes

```bash
# Get dependencies
mix deps.get

# Compile (includes proto compilation from ../atra-proto)
mix compile

# Start the gateway
iex -S mix
```

## usage with the matching engine

```elixir
# Create a new order
order = AtraGateway.Orders.new(100.0, 1.0, :bid)

# Place order
AtraGateway.MatchingEngine.place_order(order)
```

## Project Structure

```
lib/
├── atra_gateway/
│   ├── application.ex     # OTP application setup
│   ├── matching_engine.ex # gRPC client to engine
│   └── orders.ex          # Order handling utilities
├── mix/
│   └── tasks/
│       └── compile_protos.ex  # Proto compilation task
└── orderbook.pb.ex            # Generated protobuf code - TODO: untrack
```

## Dependencies

- Atra engine running on port 50051
- Protocol definitions from `../atra-proto`
- Elixir 1.15+

## Development

Follows standard Mix project structure. Key commands:
```bash
mix deps.clean --all  # Clean all dependencies
mix deps.get         # Get dependencies
mix compile         # Compile project
mix test            # Run tests
```

**TODO:**

- Real-time capabilities (WebSocket/live updates)
- State management (caching/position tracking)
- Multiple protocol support (REST/WS/FIX)
- Client session management
- Advanced features (market making/risk)

