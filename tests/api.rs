//! Integration tests for the Blackjack REST API.
//!
//! Uses Axum's test client to make HTTP requests without starting a real server.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use rumenx_blackjack::api::router::create_router;
use rumenx_blackjack::api::state::AppState;
use tower::ServiceExt;

/// Build a test app (router + fresh state).
fn app() -> axum::Router {
    let state = AppState::new();
    create_router(state)
}

/// Helper: make a GET request and return (status, body as JSON value).
async fn get(app: &axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

/// Helper: make a POST request and return (status, body as JSON value).
async fn post(app: &axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

/// Helper: make a DELETE request and return (status, body as JSON value).
async fn delete(app: &axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("DELETE")
        .uri(uri)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_endpoint() {
    let app = app();
    let (status, body) = get(&app, "/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());
}

// ---------------------------------------------------------------------------
// Game CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_game() {
    let app = app();
    let (status, body) = post(&app, "/api/games").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["id"].is_string());
    assert!(body["player_hand"]["cards"].is_array());
    assert_eq!(body["player_hand"]["cards"].as_array().unwrap().len(), 2);
    assert!(body["dealer_hand"]["cards"].is_array());
    assert_eq!(body["dealer_hand"]["cards"].as_array().unwrap().len(), 2);
    assert!(body["actions"].is_array());
}

#[tokio::test]
async fn test_list_games_empty() {
    let app = app();
    let (status, body) = get(&app, "/api/games").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_list_games_after_create() {
    let app = app();
    post(&app, "/api/games").await;
    let (status, body) = get(&app, "/api/games").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_get_game() {
    let app = app();
    let (_, created) = post(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = get(&app, &format!("/api/games/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], id);
}

#[tokio::test]
async fn test_get_nonexistent_game() {
    let app = app();
    let (status, body) = get(&app, "/api/games/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "game_not_found");
}

#[tokio::test]
async fn test_delete_game() {
    let app = app();
    let (_, created) = post(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = delete(&app, &format!("/api/games/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["deleted"], id);

    // Verify it's gone.
    let (status, _) = get(&app, &format!("/api/games/{id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_nonexistent_game() {
    let app = app();
    let (status, _) = delete(&app, "/api/games/no-such-game").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Player Actions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_hit() {
    let app = app();
    let (_, created) = post(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    // Skip if natural blackjack.
    if created["status"] == "blackjack" {
        return;
    }

    let (status, body) = post(&app, &format!("/api/games/{id}/hit")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["player_hand"]["cards"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_stand() {
    let app = app();
    let (_, created) = post(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    if created["status"] == "blackjack" {
        return;
    }

    let (status, body) = post(&app, &format!("/api/games/{id}/stand")).await;
    assert_eq!(status, StatusCode::OK);
    // Game should be terminal after stand.
    let game_status = body["status"].as_str().unwrap();
    assert_ne!(game_status, "playing");
    // All dealer cards visible.
    let dealer_cards = body["dealer_hand"]["cards"].as_array().unwrap();
    for card in dealer_cards {
        assert_ne!(
            card["rank"], "?",
            "all dealer cards should be visible after stand"
        );
    }
}

#[tokio::test]
async fn test_double_down() {
    let app = app();
    let (_, created) = post(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    if created["status"] == "blackjack" {
        return;
    }

    let (status, body) = post(&app, &format!("/api/games/{id}/double")).await;
    assert_eq!(status, StatusCode::OK);
    // Player should have 3 cards.
    assert_eq!(body["player_hand"]["cards"].as_array().unwrap().len(), 3);
    // Game should be terminal.
    let game_status = body["status"].as_str().unwrap();
    assert_ne!(game_status, "playing");
}

#[tokio::test]
async fn test_cannot_hit_after_stand() {
    let app = app();
    let (_, created) = post(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    if created["status"] == "blackjack" {
        return;
    }

    post(&app, &format!("/api/games/{id}/stand")).await;
    let (status, body) = post(&app, &format!("/api/games/{id}/hit")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "invalid_action");
}

#[tokio::test]
async fn test_dealer_hole_card_hidden_during_play() {
    let app = app();
    let (_, created) = post(&app, "/api/games").await;

    if created["status"] == "blackjack" {
        return;
    }

    let dealer_cards = created["dealer_hand"]["cards"].as_array().unwrap();
    assert_eq!(dealer_cards.len(), 2);
    // Second card should be hidden (rank "?").
    assert_eq!(dealer_cards[1]["rank"], "?");
    assert_eq!(dealer_cards[1]["suit"], "?");
    // First card should be visible.
    assert_ne!(dealer_cards[0]["rank"], "?");
}

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_history() {
    let app = app();
    let (_, created) = post(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = get(&app, &format!("/api/games/{id}/history")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], id);
    let actions = body["actions"].as_array().unwrap();
    assert!(!actions.is_empty(), "should have at least the Deal action");
    assert_eq!(actions[0], "Deal");
}

#[tokio::test]
async fn test_history_after_hit_and_stand() {
    let app = app();
    let (_, created) = post(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    if created["status"] == "blackjack" {
        return;
    }

    post(&app, &format!("/api/games/{id}/hit")).await;

    // Check if game is still playing (might have busted).
    let (_, game_state) = get(&app, &format!("/api/games/{id}")).await;
    if game_state["status"] == "playing" {
        post(&app, &format!("/api/games/{id}/stand")).await;
    }

    let (_, body) = get(&app, &format!("/api/games/{id}/history")).await;
    let actions = body["actions"].as_array().unwrap();

    // Should have at least: Deal, Hit <card>
    assert!(actions.len() >= 2, "should have multiple actions recorded");
    assert_eq!(actions[0], "Deal");
    assert!(
        actions[1].as_str().unwrap().starts_with("Hit"),
        "second action should be a hit"
    );
}

#[tokio::test]
async fn test_history_nonexistent_game() {
    let app = app();
    let (status, body) = get(&app, "/api/games/no-such-game/history").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "game_not_found");
}
