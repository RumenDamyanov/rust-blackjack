# rust-blackjack 🃏

A simple Blackjack card game engine and REST API server written in Rust.

[![CI](https://github.com/RumenDamyanov/rust-blackjack/actions/workflows/ci.yml/badge.svg)](https://github.com/RumenDamyanov/rust-blackjack/actions/workflows/ci.yml)
[![CodeQL](https://github.com/RumenDamyanov/rust-blackjack/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/RumenDamyanov/rust-blackjack/actions/workflows/github-code-scanning/codeql)
[![Dependabot](https://github.com/RumenDamyanov/rust-blackjack/actions/workflows/dependabot/dependabot-updates/badge.svg)](https://github.com/RumenDamyanov/rust-blackjack/actions/workflows/dependabot/dependabot-updates)
[![codecov](https://codecov.io/gh/RumenDamyanov/rust-blackjack/graph/badge.svg)](https://codecov.io/gh/RumenDamyanov/rust-blackjack)
[![crates.io](https://img.shields.io/crates/v/rumenx-blackjack.svg)](https://crates.io/crates/rumenx-blackjack)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE.md)

## Overview

**rust-blackjack** implements a standard single-deck Blackjack game as a backend engine with a RESTful API. It's designed to be consumed by frontend applications (web, mobile, etc.) and serves as a demo/educational project showcasing Rust and its ecosystem.

Part of a family of Blackjack implementations across languages (Rust, Go, TypeScript) sharing the same API contract.

### Features

- 🃏 Standard Blackjack rules (hit, stand, double down)
- 🎯 Pure game engine — no I/O, fully testable
- 🌐 REST API with Axum
- 🔒 Dealer hole card hidden during player's turn
- 📜 Full action history tracking
- 🐳 Docker support (< 10 MB image)
- ✅ Comprehensive tests (unit + integration)

## Quick Start

### Run the Server

```bash
cargo run
# Server starts on http://localhost:8083
```

### Play a Game

```bash
# Create a new game
curl -s -X POST http://localhost:8083/api/games | jq

# Hit
curl -s -X POST http://localhost:8083/api/games/{id}/hit | jq

# Stand
curl -s -X POST http://localhost:8083/api/games/{id}/stand | jq
```

### Use as a Library

```rust
use rumenx_blackjack::engine::game::Game;

let mut game = Game::new().expect("should deal");
println!("Player: {:?}", game.player_hand().cards());

let card = game.hit().expect("should hit");
println!("Drew: {card}");

let result = game.stand().expect("should stand");
println!("Outcome: {result}");
```

## API Reference

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `POST` | `/api/games` | Create new game (deal) |
| `GET` | `/api/games` | List active games |
| `GET` | `/api/games/{id}` | Get game state |
| `DELETE` | `/api/games/{id}` | Delete game |
| `POST` | `/api/games/{id}/hit` | Player hits |
| `POST` | `/api/games/{id}/stand` | Player stands |
| `POST` | `/api/games/{id}/double` | Double down |
| `POST` | `/api/games/{id}/split` | Split pair (v2) |
| `GET` | `/api/games/{id}/history` | Action history |

### Example Response

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "playing",
  "player_hand": {
    "cards": [
      { "rank": "A", "suit": "spades" },
      { "rank": "10", "suit": "hearts" }
    ],
    "value": 21,
    "soft": true
  },
  "dealer_hand": {
    "cards": [
      { "rank": "K", "suit": "clubs" },
      { "rank": "?", "suit": "?" }
    ],
    "value": 10,
    "visible_value": 10
  },
  "deck_remaining": 48,
  "actions": ["hit", "stand", "double"]
}
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt --all

# All checks (format + lint + test)
make check

# Coverage report
make coverage

# Run example
cargo run --example basic_game
```

## Docker

```bash
# Build image
docker build -t rust-blackjack .

# Run container
docker run -p 8083:8083 --rm rust-blackjack

# Check image size (target: < 10 MB)
docker images rust-blackjack
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8083` | Server port |
| `HOST` | `0.0.0.0` | Bind address |
| `RUST_LOG` | `rumenx_blackjack=info` | Log level |

## Project Structure

```
src/
├── main.rs          # Server entry point
├── lib.rs           # Library root
├── config.rs        # Environment configuration
├── engine/          # Pure game logic (no I/O)
│   ├── types.rs     # Card, Rank, Suit, GameStatus
│   ├── deck.rs      # 52-card deck with shuffle
│   ├── hand.rs      # Hand value calculation
│   └── game.rs      # Game state machine
└── api/             # REST API layer
    ├── router.rs    # Route definitions
    ├── handlers.rs  # Endpoint handlers
    ├── models.rs    # JSON DTOs
    ├── state.rs     # Shared state
    └── errors.rs    # Error types
```

## License

[MIT](LICENSE.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Security

See [SECURITY.md](SECURITY.md) for reporting vulnerabilities.
