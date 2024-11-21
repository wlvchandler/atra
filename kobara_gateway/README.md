# KobaraGateway

Real-time Elixir gateway service for a Kobara system. Provides real-time connectivity and state management for backend engine.

## build notes

```bash
# Get dependencies
mix deps.get

# Compile (includes proto compilation from ../kobara-proto)
mix compile

# Start the gateway
iex -S mix
```

## usage with the matching engine

```elixir
# Create a new order
order = KobaraGateway.Orders.new(100.0, 1.0, :bid)

# Place order
KobaraGateway.MatchingEngine.place_order(order)
```

## Project Structure

```
lib/
├── kobara_gateway/
│   ├── application.ex     # OTP application setup
│   ├── matching_engine.ex # gRPC client to engine
│   └── orders.ex          # Order handling utilities
├── mix/
│   └── tasks/
│       └── compile_protos.ex  # Proto compilation task
└── orderbook.pb.ex            # Generated protobuf code - TODO: untrack
```

## Dependencies

- Kobara engine running on port 50051
- Protocol definitions from `../kobara-proto`
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

