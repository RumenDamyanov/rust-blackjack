//! REST API layer for the Blackjack engine.
//!
//! Axum-based HTTP server with JSON endpoints, CORS, and structured errors.

pub mod errors;
pub mod handlers;
pub mod models;
pub mod router;
pub mod state;
