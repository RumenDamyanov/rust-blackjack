//! Deck management for Blackjack.
//!
//! A standard 52-card deck that can be shuffled and drawn from.
//! Uses `rand` for shuffling — the only external dependency in the engine.

use rand::rng;
use rand::seq::SliceRandom;

use super::types::{Card, GameError, Rank, Suit};

/// A standard 52-card deck.
#[derive(Clone, Debug)]
pub struct Deck {
    /// Cards remaining in the deck. Top of deck = last element.
    cards: Vec<Card>,
}

impl Deck {
    /// Create a new, unshuffled 52-card deck.
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(52);
        for &suit in &Suit::ALL {
            for &rank in &Rank::ALL {
                cards.push(Card::new(rank, suit));
            }
        }
        Self { cards }
    }

    /// Create a new deck and shuffle it.
    pub fn new_shuffled() -> Self {
        let mut deck = Self::new();
        deck.shuffle();
        deck
    }

    /// Shuffle the remaining cards in place.
    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut rng());
    }

    /// Draw the top card from the deck.
    ///
    /// Returns `Err(GameError::DeckEmpty)` if no cards remain.
    pub fn draw(&mut self) -> Result<Card, GameError> {
        self.cards.pop().ok_or(GameError::DeckEmpty)
    }

    /// Number of cards remaining in the deck.
    pub fn remaining(&self) -> usize {
        self.cards.len()
    }

    /// Whether the deck has no cards left.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_new_deck_has_52_cards() {
        let deck = Deck::new();
        assert_eq!(deck.remaining(), 52);
    }

    #[test]
    fn test_new_deck_has_unique_cards() {
        let deck = Deck::new();
        let unique: HashSet<Card> = deck.cards.iter().copied().collect();
        assert_eq!(unique.len(), 52, "all 52 cards should be unique");
    }

    #[test]
    fn test_draw_reduces_count() {
        let mut deck = Deck::new_shuffled();
        let _ = deck.draw().expect("should draw a card");
        assert_eq!(deck.remaining(), 51);
    }

    #[test]
    fn test_draw_returns_card() {
        let mut deck = Deck::new_shuffled();
        let card = deck.draw().expect("should draw a card");
        // Card should have valid rank and suit (if it compiles, the types are correct)
        assert!(Rank::ALL.contains(&card.rank));
        assert!(Suit::ALL.contains(&card.suit));
    }

    #[test]
    fn test_draw_all_cards_then_empty() {
        let mut deck = Deck::new_shuffled();
        for _ in 0..52 {
            deck.draw().expect("should have cards");
        }
        assert!(deck.is_empty());
        assert!(deck.draw().is_err(), "drawing from empty deck should error");
    }

    #[test]
    fn test_shuffled_deck_has_52_unique_cards() {
        let deck = Deck::new_shuffled();
        assert_eq!(deck.remaining(), 52);
        let unique: HashSet<Card> = deck.cards.iter().copied().collect();
        assert_eq!(unique.len(), 52);
    }

    #[test]
    fn test_shuffle_changes_order() {
        // This test is probabilistic but the chance of two shuffles
        // producing the same order is astronomically small.
        let deck1 = Deck::new_shuffled();
        let deck2 = Deck::new_shuffled();
        // Compare the card sequences — they should almost certainly differ.
        assert_ne!(
            deck1.cards, deck2.cards,
            "two shuffled decks should (almost certainly) differ"
        );
    }
}
