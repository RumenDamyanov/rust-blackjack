//! Request/response DTOs for the REST API.
//!
//! These are thin JSON-serializable structs separate from the engine types.
//! They control the exact API surface sent to clients.

use serde::Serialize;

use crate::engine::game::Game;
use crate::engine::types::{Card, GameStatus};

/// JSON response for a single card.
#[derive(Serialize)]
pub struct CardResponse {
    pub rank: String,
    pub suit: String,
}

/// JSON response for a hand (player or dealer).
#[derive(Serialize)]
pub struct HandResponse {
    pub cards: Vec<CardResponse>,
    pub value: u8,
    pub soft: bool,
}

/// JSON response for the dealer's hand (may hide the hole card).
#[derive(Serialize)]
pub struct DealerHandResponse {
    pub cards: Vec<CardResponse>,
    pub value: u8,
    pub visible_value: u8,
}

/// Full game state response.
#[derive(Serialize)]
pub struct GameResponse {
    pub id: String,
    pub status: String,
    pub player_hand: HandResponse,
    pub dealer_hand: DealerHandResponse,
    pub deck_remaining: usize,
    pub actions: Vec<String>,
}

/// Minimal game summary for listing.
#[derive(Serialize)]
pub struct GameSummary {
    pub id: String,
    pub status: String,
    pub player_value: u8,
}

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Action history response.
#[derive(Serialize)]
pub struct HistoryResponse {
    pub id: String,
    pub actions: Vec<String>,
}

// ---------------------------------------------------------------------------
// Conversions from engine types
// ---------------------------------------------------------------------------

impl CardResponse {
    /// Create from an engine `Card`.
    pub fn from_card(card: &Card) -> Self {
        Self {
            rank: card.rank.symbol().to_string(),
            suit: card.suit.name().to_string(),
        }
    }

    /// A hidden card (dealer's hole card).
    pub fn hidden() -> Self {
        Self {
            rank: "?".to_string(),
            suit: "?".to_string(),
        }
    }
}

impl GameResponse {
    /// Build a game response from a `Game`, respecting dealer card visibility.
    pub fn from_game(game: &Game) -> Self {
        let player_val = game.player_hand().value();
        let dealer_visible = game.dealer_visible_cards();
        let dealer_full_val = game.dealer_hand().value();

        let dealer_cards: Vec<CardResponse> = dealer_visible
            .iter()
            .map(|opt| match opt {
                Some(card) => CardResponse::from_card(card),
                None => CardResponse::hidden(),
            })
            .collect();

        let player_cards: Vec<CardResponse> = game
            .player_hand()
            .cards()
            .iter()
            .map(CardResponse::from_card)
            .collect();

        // Visible value is only the face-up card during player's turn.
        let visible_value = game.dealer_visible_value();

        // Full dealer value is only shown when game is over.
        let dealer_value = if game.status() == GameStatus::Playing {
            visible_value
        } else {
            dealer_full_val.value
        };

        Self {
            id: game.id().to_string(),
            status: game.status().to_string(),
            player_hand: HandResponse {
                cards: player_cards,
                value: player_val.value,
                soft: player_val.soft,
            },
            dealer_hand: DealerHandResponse {
                cards: dealer_cards,
                value: dealer_value,
                visible_value,
            },
            deck_remaining: game.deck_remaining(),
            actions: game.available_actions(),
        }
    }
}

impl GameSummary {
    /// Build a summary from a `Game`.
    pub fn from_game(game: &Game) -> Self {
        Self {
            id: game.id().to_string(),
            status: game.status().to_string(),
            player_value: game.player_hand().value().value,
        }
    }
}
