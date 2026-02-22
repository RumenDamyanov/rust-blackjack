//! # rumenx-blackjack
//!
//! A simple Blackjack card game engine and REST API server written in Rust.
//!
//! ## Modules
//!
//! - [`engine`] — Pure game logic: cards, deck, hand value, game state machine.
//! - [`api`] — Axum HTTP server with REST endpoints.
//! - [`config`] — Environment-based server configuration.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rumenx_blackjack::engine::game::Game;
//!
//! let mut game = Game::new().expect("should deal");
//! println!("Player: {:?}", game.player_hand().cards());
//!
//! // Hit
//! if !game.status().is_terminal() {
//!     let card = game.hit().expect("should hit");
//!     println!("Drew: {card}");
//! }
//! ```

pub mod api;
pub mod config;
pub mod engine;
