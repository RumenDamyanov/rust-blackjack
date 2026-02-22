//! Axum router with all routes and middleware.

use axum::Router;
use axum::routing::{get, post};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use super::handlers;
use super::state::SharedState;

/// Build the Axum router with all routes and middleware.
pub fn create_router(state: SharedState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // Game CRUD
        .route(
            "/api/games",
            post(handlers::create_game).get(handlers::list_games),
        )
        .route(
            "/api/games/{id}",
            get(handlers::get_game).delete(handlers::delete_game),
        )
        // Player actions
        .route("/api/games/{id}/hit", post(handlers::hit))
        .route("/api/games/{id}/stand", post(handlers::stand))
        .route("/api/games/{id}/double", post(handlers::double_down))
        .route("/api/games/{id}/split", post(handlers::split))
        // History
        .route("/api/games/{id}/history", get(handlers::history))
        // Middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
