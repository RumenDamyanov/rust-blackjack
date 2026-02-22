# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-21

### Added
- Core Blackjack engine: Card, Deck, Hand, Game state machine
- Card types: Rank (2–A), Suit (♥♦♣♠), Card struct
- Deck: standard 52-card deck with shuffle and draw
- Hand: value calculation with soft/hard ace logic, blackjack and bust detection
- Game: state machine with deal, hit, stand, double down actions
- Dealer logic: hits until ≥ 17, stands on soft 17
- Action history tracking
- REST API with Axum:
  - `GET /health` — health check
  - `POST /api/games` — create new game
  - `GET /api/games` — list active games
  - `GET /api/games/{id}` — get game state
  - `DELETE /api/games/{id}` — delete game
  - `POST /api/games/{id}/hit` — player hits
  - `POST /api/games/{id}/stand` — player stands
  - `POST /api/games/{id}/double` — double down
  - `POST /api/games/{id}/split` — split (stub, returns error in v1)
  - `GET /api/games/{id}/history` — action history
- CORS support
- Structured JSON error responses
- Dealer hole card hidden during player's turn
- Comprehensive unit tests (engine) and integration tests (API)
- Docker support (multi-stage build, minimal image)
- CI pipeline (GitHub Actions): format, lint, test, coverage, Docker
- Publish workflow for crates.io
