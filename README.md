# Kobara 

A high-performance trading system

[![forthebadge](https://forthebadge.com/images/badges/powered-by-electricity.svg)](https://forthebadge.com)
[![CI/CD](https://github.com/wlvchandler/kobara/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/wlvchandler/kobara/actions/workflows/ci.yml)

## Components

| Component | Description | Language | Role |
|-----------|-------------|----------|------|
| `kobara-ob` | Core matching engine | Rust | High-performance order matching and book management |
| `kobara-cli` | Command-line interface | Python | User interaction and order entry |
| `kobara-proto` | Protocol definitions | Protobuf | Shared data contracts and API specifications |
| `kobara_gateway` | Real-time gateway | Elixir | Connection management and real-time engine access  |


## Quick Start

```bash
# Build all containers
docker compose build
docker build -t kobara-cli -f kobara-cli/Dockerfile .

# Start the matching engine
docker compose up -d orderbook

# Place orders using the CLI wrapper
./invm buy 50@15 25@14 sell 19@11.20
```

## CLI Usage

The `invm` script provides a user-friendly interface to the trading system:

```bash
./invm COMMAND [OPTIONS]

Commands:
    book DEPTH              # View order book to specified depth
    (buy|sell) ORDERS...    # Place one or more orders

Order Format:
    AMOUNT[@PRICE]         # Market order if price omitted

Examples:
    ./invm book 10         # Show top 10 levels
    ./invm sell 100@10     # Limit sell 100 @ 10
    ./invm buy 300         # Market buy 300
```

## Development Setup

### Matching Engine (Rust)
```bash
cd kobara-ob
cargo build

# Run tests
cargo nextest run

# Start engine
cargo run --bin server
```

### CLI (Python)
```bash
cd kobara-cli
./scripts/setup_dev
source venv/bin/activate
```

### Gateway (Elixir)
```bash
cd kobara_gateway
mix deps.get
mix compile
iex -S mix
```

## Project Structure
```
├── kobara-ob/        # Rust matching engine
├── kobara-cli/       # Python CLI
├── kobara-proto/     # Shared protocol definitions
└── kobara_gateway/   # Elixir gateway service
```

## Documentation

Detailed documentation available in the `doc/` directory:
- Architecture diagrams
- Component specifications
- Data flow descriptions
- Performance analysis
