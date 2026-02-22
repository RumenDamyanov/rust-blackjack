//! Core Blackjack engine types.
//!
//! Foundational types shared across the engine: cards, hand values,
//! game status, and action history. These are pure data types with
//! no I/O or framework dependencies.

use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// Rank
// ---------------------------------------------------------------------------

/// The thirteen ranks in a standard deck of playing cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    /// All thirteen ranks in order (Two through Ace).
    pub const ALL: [Rank; 13] = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];

    /// Base point value of this rank in Blackjack.
    ///
    /// Face cards (J, Q, K) are worth 10.
    /// Ace is initially valued at 11 (soft); the hand module
    /// handles downgrading to 1 when needed.
    /// Number cards are face value.
    pub fn value(self) -> u8 {
        match self {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 10,
            Rank::Ace => 11,
        }
    }

    /// Short display string (e.g. "A", "K", "10", "2").
    pub fn symbol(self) -> &'static str {
        match self {
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Ace => "A",
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

// ---------------------------------------------------------------------------
// Suit
// ---------------------------------------------------------------------------

/// The four suits in a standard deck of playing cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl Suit {
    /// All four suits.
    pub const ALL: [Suit; 4] = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];

    /// Lowercase name for JSON serialization.
    pub fn name(self) -> &'static str {
        match self {
            Suit::Hearts => "hearts",
            Suit::Diamonds => "diamonds",
            Suit::Clubs => "clubs",
            Suit::Spades => "spades",
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        };
        write!(f, "{symbol}")
    }
}

// ---------------------------------------------------------------------------
// Card
// ---------------------------------------------------------------------------

/// A single playing card with a rank and suit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    /// Create a new card.
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

// ---------------------------------------------------------------------------
// HandValue
// ---------------------------------------------------------------------------

/// The calculated value of a Blackjack hand.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandValue {
    /// Total point value (best possible without busting, if achievable).
    pub value: u8,
    /// Whether an Ace is currently counted as 11 (soft hand).
    pub soft: bool,
}

// ---------------------------------------------------------------------------
// GameStatus
// ---------------------------------------------------------------------------

/// The current status of a Blackjack game.
///
/// Acts as a state machine — only certain transitions are valid.
/// See the instructions.md state machine diagram for the full flow.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    /// Waiting for initial deal.
    Playing,
    /// Dealer is drawing cards (internal, resolved immediately).
    DealerTurn,
    /// Player exceeded 21.
    PlayerBust,
    /// Dealer exceeded 21 — player wins.
    DealerBust,
    /// Player's hand beats dealer's hand.
    PlayerWins,
    /// Dealer's hand beats player's hand.
    DealerWins,
    /// Both hands have equal value.
    Push,
    /// Player has natural 21 on first two cards.
    Blackjack,
}

impl GameStatus {
    /// Whether this status represents a finished game.
    pub fn is_terminal(self) -> bool {
        !matches!(self, GameStatus::Playing)
    }
}

impl fmt::Display for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GameStatus::Playing => "playing",
            GameStatus::DealerTurn => "dealer_turn",
            GameStatus::PlayerBust => "player_bust",
            GameStatus::DealerBust => "dealer_bust",
            GameStatus::PlayerWins => "player_wins",
            GameStatus::DealerWins => "dealer_wins",
            GameStatus::Push => "push",
            GameStatus::Blackjack => "blackjack",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// Action (history tracking)
// ---------------------------------------------------------------------------

/// A recorded player or dealer action for the game history log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Initial deal.
    Deal,
    /// Player drew a card.
    Hit { card: Card },
    /// Player chose to stand.
    Stand,
    /// Player doubled down, drawing one card.
    Double { card: Card },
    /// Dealer drew a card.
    DealerHit { card: Card },
    /// Dealer stood (reached ≥ 17).
    DealerStand,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Deal => write!(f, "Deal"),
            Action::Hit { card } => write!(f, "Hit {card}"),
            Action::Stand => write!(f, "Stand"),
            Action::Double { card } => write!(f, "Double {card}"),
            Action::DealerHit { card } => write!(f, "Dealer hits {card}"),
            Action::DealerStand => write!(f, "Dealer stands"),
        }
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur in the Blackjack engine.
#[derive(Debug, thiserror::Error)]
pub enum GameError {
    /// Attempted an action that isn't valid in the current game state.
    #[error("invalid action: {0}")]
    InvalidAction(String),

    /// The deck ran out of cards (should not happen in single-deck Blackjack).
    #[error("deck is empty")]
    DeckEmpty,

    /// Cannot split — hand does not contain a pair.
    #[error("cannot split: hand does not contain a pair")]
    CannotSplit,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_values() {
        assert_eq!(Rank::Two.value(), 2);
        assert_eq!(Rank::Ten.value(), 10);
        assert_eq!(Rank::Jack.value(), 10);
        assert_eq!(Rank::Queen.value(), 10);
        assert_eq!(Rank::King.value(), 10);
        assert_eq!(Rank::Ace.value(), 11);
    }

    #[test]
    fn test_rank_symbols() {
        assert_eq!(Rank::Ace.symbol(), "A");
        assert_eq!(Rank::Ten.symbol(), "10");
        assert_eq!(Rank::King.symbol(), "K");
        assert_eq!(Rank::Two.symbol(), "2");
    }

    #[test]
    fn test_rank_all_has_13_variants() {
        assert_eq!(Rank::ALL.len(), 13);
    }

    #[test]
    fn test_suit_names() {
        assert_eq!(Suit::Hearts.name(), "hearts");
        assert_eq!(Suit::Spades.name(), "spades");
    }

    #[test]
    fn test_suit_all_has_4_variants() {
        assert_eq!(Suit::ALL.len(), 4);
    }

    #[test]
    fn test_card_display() {
        let card = Card::new(Rank::Ace, Suit::Spades);
        assert_eq!(card.to_string(), "A♠");

        let card = Card::new(Rank::Ten, Suit::Hearts);
        assert_eq!(card.to_string(), "10♥");
    }

    #[test]
    fn test_game_status_is_terminal() {
        assert!(!GameStatus::Playing.is_terminal());
        assert!(GameStatus::PlayerBust.is_terminal());
        assert!(GameStatus::DealerBust.is_terminal());
        assert!(GameStatus::PlayerWins.is_terminal());
        assert!(GameStatus::DealerWins.is_terminal());
        assert!(GameStatus::Push.is_terminal());
        assert!(GameStatus::Blackjack.is_terminal());
    }

    #[test]
    fn test_game_status_display() {
        assert_eq!(GameStatus::Playing.to_string(), "playing");
        assert_eq!(GameStatus::PlayerBust.to_string(), "player_bust");
        assert_eq!(GameStatus::Blackjack.to_string(), "blackjack");
    }

    #[test]
    fn test_action_display() {
        let card = Card::new(Rank::Five, Suit::Clubs);
        assert_eq!(Action::Deal.to_string(), "Deal");
        assert_eq!(Action::Hit { card }.to_string(), "Hit 5♣");
        assert_eq!(Action::Stand.to_string(), "Stand");
    }
}
