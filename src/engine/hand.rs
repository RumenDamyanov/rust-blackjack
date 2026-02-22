//! Hand management for Blackjack.
//!
//! A hand holds a player's or dealer's cards and calculates
//! the best possible value following Blackjack rules:
//! - Number cards are face value
//! - Face cards (J, Q, K) are 10
//! - Aces are 11 unless that busts, then 1

use super::types::{Card, HandValue, Rank};

/// A Blackjack hand — a collection of cards with value calculation.
#[derive(Clone, Debug, Default)]
pub struct Hand {
    /// Cards currently in this hand.
    cards: Vec<Card>,
}

impl Hand {
    /// Create an empty hand.
    pub fn new() -> Self {
        Self { cards: Vec::new() }
    }

    /// Add a card to the hand.
    pub fn add(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// The cards in this hand.
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    /// Number of cards in the hand.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Whether the hand has no cards.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Calculate the best value for this hand.
    ///
    /// Aces count as 11 unless that would cause a bust (> 21),
    /// in which case they are downgraded to 1 one at a time.
    ///
    /// Returns a [`HandValue`] with the total and whether the
    /// hand is "soft" (an Ace is counted as 11).
    pub fn value(&self) -> HandValue {
        let mut total: u8 = 0;
        let mut aces: u8 = 0;

        for card in &self.cards {
            // Ace initially counts as 11 via Rank::value().
            total = total.saturating_add(card.rank.value());
            if card.rank == Rank::Ace {
                aces += 1;
            }
        }

        // Downgrade aces from 11 → 1 as needed to avoid bust.
        while total > 21 && aces > 0 {
            total -= 10;
            aces -= 1;
        }

        HandValue {
            value: total,
            soft: aces > 0,
        }
    }

    /// Whether the hand value exceeds 21 (bust).
    pub fn is_bust(&self) -> bool {
        self.value().value > 21
    }

    /// Whether the hand is a natural Blackjack (exactly two cards totaling 21).
    pub fn is_blackjack(&self) -> bool {
        self.cards.len() == 2 && self.value().value == 21
    }

    /// Whether the hand is a pair (exactly two cards of the same rank).
    ///
    /// This is used to determine if the player can split.
    pub fn is_pair(&self) -> bool {
        self.cards.len() == 2 && self.cards[0].rank == self.cards[1].rank
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::Suit;

    /// Helper: create a card.
    fn card(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit)
    }

    #[test]
    fn test_empty_hand() {
        let hand = Hand::new();
        assert!(hand.is_empty());
        assert_eq!(hand.len(), 0);
        assert_eq!(
            hand.value(),
            HandValue {
                value: 0,
                soft: false
            }
        );
    }

    #[test]
    fn test_simple_hard_hand() {
        let mut hand = Hand::new();
        hand.add(card(Rank::Ten, Suit::Hearts));
        hand.add(card(Rank::Seven, Suit::Clubs));
        let v = hand.value();
        assert_eq!(v.value, 17);
        assert!(!v.soft, "no aces means hard hand");
    }

    #[test]
    fn test_soft_hand_with_ace() {
        let mut hand = Hand::new();
        hand.add(card(Rank::Ace, Suit::Spades));
        hand.add(card(Rank::Six, Suit::Hearts));
        let v = hand.value();
        assert_eq!(v.value, 17, "A + 6 = soft 17");
        assert!(v.soft);
    }

    #[test]
    fn test_ace_downgrades_when_bust() {
        let mut hand = Hand::new();
        hand.add(card(Rank::Ace, Suit::Spades));
        hand.add(card(Rank::Eight, Suit::Hearts));
        hand.add(card(Rank::Seven, Suit::Clubs));
        let v = hand.value();
        // A(11)+8+7 = 26 → bust, so A becomes 1: 1+8+7 = 16
        assert_eq!(v.value, 16);
        assert!(!v.soft, "ace was downgraded");
        assert!(!hand.is_bust());
    }

    #[test]
    fn test_two_aces_downgrade() {
        let mut hand = Hand::new();
        hand.add(card(Rank::Ace, Suit::Hearts));
        hand.add(card(Rank::Ace, Suit::Spades));
        let v = hand.value();
        // A(11)+A(11) = 22 → one downgrade: 11+1 = 12
        assert_eq!(v.value, 12);
        assert!(v.soft, "one ace still counts as 11");
    }

    #[test]
    fn test_three_aces() {
        let mut hand = Hand::new();
        hand.add(card(Rank::Ace, Suit::Hearts));
        hand.add(card(Rank::Ace, Suit::Spades));
        hand.add(card(Rank::Ace, Suit::Clubs));
        let v = hand.value();
        // A(11)+A(11)+A(11) = 33 → 23 → 13
        assert_eq!(v.value, 13);
        assert!(v.soft, "one ace still counts as 11");
    }

    #[test]
    fn test_blackjack() {
        let mut hand = Hand::new();
        hand.add(card(Rank::Ace, Suit::Hearts));
        hand.add(card(Rank::King, Suit::Spades));
        assert!(hand.is_blackjack());
        assert_eq!(hand.value().value, 21);
        assert!(hand.value().soft);
    }

    #[test]
    fn test_21_but_not_blackjack() {
        let mut hand = Hand::new();
        hand.add(card(Rank::Seven, Suit::Hearts));
        hand.add(card(Rank::Seven, Suit::Spades));
        hand.add(card(Rank::Seven, Suit::Clubs));
        assert_eq!(hand.value().value, 21);
        assert!(
            !hand.is_blackjack(),
            "three cards is not a natural blackjack"
        );
    }

    #[test]
    fn test_bust() {
        let mut hand = Hand::new();
        hand.add(card(Rank::Ten, Suit::Hearts));
        hand.add(card(Rank::King, Suit::Spades));
        hand.add(card(Rank::Five, Suit::Clubs));
        assert!(hand.is_bust(), "10+10+5 = 25 is a bust");
        assert_eq!(hand.value().value, 25);
    }

    #[test]
    fn test_is_pair() {
        let mut hand = Hand::new();
        hand.add(card(Rank::Eight, Suit::Hearts));
        hand.add(card(Rank::Eight, Suit::Spades));
        assert!(hand.is_pair());
    }

    #[test]
    fn test_is_not_pair_different_ranks() {
        let mut hand = Hand::new();
        hand.add(card(Rank::Eight, Suit::Hearts));
        hand.add(card(Rank::Nine, Suit::Spades));
        assert!(!hand.is_pair());
    }

    #[test]
    fn test_is_not_pair_three_cards() {
        let mut hand = Hand::new();
        hand.add(card(Rank::Eight, Suit::Hearts));
        hand.add(card(Rank::Eight, Suit::Spades));
        hand.add(card(Rank::Eight, Suit::Clubs));
        assert!(!hand.is_pair(), "pair requires exactly 2 cards");
    }

    #[test]
    fn test_face_cards_worth_ten() {
        for rank in [Rank::Jack, Rank::Queen, Rank::King] {
            let mut hand = Hand::new();
            hand.add(card(rank, Suit::Hearts));
            assert_eq!(hand.value().value, 10, "{rank:?} should be worth 10");
        }
    }
}
