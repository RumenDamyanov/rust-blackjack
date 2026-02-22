//! API endpoint handlers.
//!
//! Each function is a thin wrapper that delegates to the engine.
//! Handlers validate input, call engine methods, and return JSON responses.

use axum::Json;
use axum::extract::{Path, State};

use super::errors::ApiError;
use super::models::{GameResponse, GameSummary, HealthResponse, HistoryResponse};
use super::state::SharedState;
use crate::engine::game::Game;

/// `GET /health` — Health check.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// `POST /api/games` — Create a new game and deal initial hands.
pub async fn create_game(State(state): State<SharedState>) -> Result<Json<GameResponse>, ApiError> {
    let game = Game::new().map_err(ApiError::from)?;
    let response = GameResponse::from_game(&game);
    let id = game.id().to_string();

    state.games.write().await.insert(id, game);

    Ok(Json(response))
}

/// `GET /api/games` — List all active games.
pub async fn list_games(State(state): State<SharedState>) -> Json<Vec<GameSummary>> {
    let games = state.games.read().await;
    let summaries: Vec<GameSummary> = games.values().map(GameSummary::from_game).collect();
    Json(summaries)
}

/// `GET /api/games/{id}` — Get game state.
pub async fn get_game(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<GameResponse>, ApiError> {
    let games = state.games.read().await;
    let game = games.get(&id).ok_or(ApiError::GameNotFound(id))?;
    Ok(Json(GameResponse::from_game(game)))
}

/// `DELETE /api/games/{id}` — Delete a game.
pub async fn delete_game(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut games = state.games.write().await;
    games
        .remove(&id)
        .ok_or(ApiError::GameNotFound(id.clone()))?;
    Ok(Json(serde_json::json!({ "deleted": id })))
}

/// `POST /api/games/{id}/hit` — Player hits (draws a card).
pub async fn hit(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<GameResponse>, ApiError> {
    let mut games = state.games.write().await;
    let game = games.get_mut(&id).ok_or(ApiError::GameNotFound(id))?;
    game.hit().map_err(ApiError::from)?;
    Ok(Json(GameResponse::from_game(game)))
}

/// `POST /api/games/{id}/stand` — Player stands.
pub async fn stand(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<GameResponse>, ApiError> {
    let mut games = state.games.write().await;
    let game = games.get_mut(&id).ok_or(ApiError::GameNotFound(id))?;
    game.stand().map_err(ApiError::from)?;
    Ok(Json(GameResponse::from_game(game)))
}

/// `POST /api/games/{id}/double` — Player doubles down.
pub async fn double_down(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<GameResponse>, ApiError> {
    let mut games = state.games.write().await;
    let game = games.get_mut(&id).ok_or(ApiError::GameNotFound(id))?;
    game.double_down().map_err(ApiError::from)?;
    Ok(Json(GameResponse::from_game(game)))
}

/// `POST /api/games/{id}/split` — Player splits a pair.
pub async fn split(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<GameResponse>, ApiError> {
    let mut games = state.games.write().await;
    let game = games.get_mut(&id).ok_or(ApiError::GameNotFound(id))?;
    game.split().map_err(ApiError::from)?;
    Ok(Json(GameResponse::from_game(game)))
}

/// `GET /api/games/{id}/history` — Get action history.
pub async fn history(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<HistoryResponse>, ApiError> {
    let games = state.games.read().await;
    let game = games.get(&id).ok_or(ApiError::GameNotFound(id))?;

    let actions: Vec<String> = game.history().iter().map(|a| a.to_string()).collect();

    Ok(Json(HistoryResponse {
        id: game.id().to_string(),
        actions,
    }))
}
