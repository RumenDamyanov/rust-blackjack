//! Blackjack game state machine.
//!
//! The [`Game`] struct owns all mutable state for a single round of
//! Blackjack: the deck, player hand, dealer hand, and status.
//! It enforces valid state transitions — calling [`Game::hit`] on a
//! finished game returns an error.

use uuid::Uuid;

use super::deck::Deck;
use super::hand::Hand;
use super::types::{Action, Card, GameError, GameStatus, HandValue};

/// A single Blackjack game (one round, player vs dealer).
#[derive(Clone, Debug)]
pub struct Game {
    /// Unique identifier for this game.
    id: String,
    /// The deck of cards.
    deck: Deck,
    /// Player's hand.
    player_hand: Hand,
    /// Dealer's hand.
    dealer_hand: Hand,
    /// Current game status.
    status: GameStatus,
    /// Ordered log of actions taken during this game.
    history: Vec<Action>,
}

impl Game {
    /// Create a new game with a shuffled deck and deal the initial hands.
    ///
    /// The deal follows standard Blackjack order:
    /// 1. Player card, 2. Dealer card, 3. Player card, 4. Dealer card.
    ///
    /// If the player has a natural Blackjack (21 on first two cards),
    /// the game status is set to `Blackjack` immediately.
    pub fn new() -> Result<Self, GameError> {
        let mut game = Self {
            id: Uuid::new_v4().to_string(),
            deck: Deck::new_shuffled(),
            player_hand: Hand::new(),
            dealer_hand: Hand::new(),
            status: GameStatus::Playing,
            history: Vec::new(),
        };

        // Deal: player, dealer, player, dealer.
        game.player_hand.add(game.deck.draw()?);
        game.dealer_hand.add(game.deck.draw()?);
        game.player_hand.add(game.deck.draw()?);
        game.dealer_hand.add(game.deck.draw()?);
        game.history.push(Action::Deal);

        // Check for natural blackjack.
        if game.player_hand.is_blackjack() {
            game.status = GameStatus::Blackjack;
        }

        Ok(game)
    }

    // -- Accessors ----------------------------------------------------------

    /// The game's unique identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Current game status.
    pub fn status(&self) -> GameStatus {
        self.status
    }

    /// The player's hand.
    pub fn player_hand(&self) -> &Hand {
        &self.player_hand
    }

    /// The dealer's hand.
    pub fn dealer_hand(&self) -> &Hand {
        &self.dealer_hand
    }

    /// Number of cards remaining in the deck.
    pub fn deck_remaining(&self) -> usize {
        self.deck.remaining()
    }

    /// The action history for this game.
    pub fn history(&self) -> &[Action] {
        &self.history
    }

    /// Available actions the player can take in the current state.
    pub fn available_actions(&self) -> Vec<String> {
        if self.status != GameStatus::Playing {
            return vec![];
        }

        let mut actions = vec!["hit".to_string(), "stand".to_string()];

        // Double down: only allowed on first two cards.
        if self.player_hand.len() == 2 {
            actions.push("double".to_string());
        }

        // Split: only allowed with a pair on first two cards.
        if self.player_hand.is_pair() {
            actions.push("split".to_string());
        }

        actions
    }

    /// The dealer's visible hand value (only the face-up card when game is active).
    pub fn dealer_visible_value(&self) -> u8 {
        if self.status == GameStatus::Playing {
            // Only the first card is visible during player's turn.
            let cards = self.dealer_hand.cards();
            if cards.is_empty() {
                0
            } else {
                cards[0].rank.value()
            }
        } else {
            self.dealer_hand.value().value
        }
    }

    /// The dealer's visible cards (hides the hole card during player's turn).
    pub fn dealer_visible_cards(&self) -> Vec<Option<Card>> {
        let cards = self.dealer_hand.cards();
        if self.status == GameStatus::Playing && cards.len() >= 2 {
            // Show first card, hide the rest.
            let mut visible = vec![Some(cards[0])];
            for _ in 1..cards.len() {
                visible.push(None); // hidden (hole card)
            }
            visible
        } else {
            // Game is over — show all cards.
            cards.iter().copied().map(Some).collect()
        }
    }

    // -- Player Actions -----------------------------------------------------

    /// Player hits — draws one card.
    ///
    /// If the new hand busts (> 21), the game ends with `PlayerBust`.
    pub fn hit(&mut self) -> Result<Card, GameError> {
        self.require_playing()?;

        let card = self.deck.draw()?;
        self.player_hand.add(card);
        self.history.push(Action::Hit { card });

        if self.player_hand.is_bust() {
            self.status = GameStatus::PlayerBust;
        }

        Ok(card)
    }

    /// Player stands — ends player's turn and triggers dealer play.
    ///
    /// The dealer draws until reaching ≥ 17, then the game is resolved.
    pub fn stand(&mut self) -> Result<GameStatus, GameError> {
        self.require_playing()?;

        self.history.push(Action::Stand);
        self.status = GameStatus::DealerTurn;
        self.dealer_play()?;
        self.resolve();

        Ok(self.status)
    }

    /// Player doubles down — doubles the bet, draws exactly one card,
    /// then stands immediately (dealer plays).
    ///
    /// Only allowed on the first two cards.
    pub fn double_down(&mut self) -> Result<Card, GameError> {
        self.require_playing()?;

        if self.player_hand.len() != 2 {
            return Err(GameError::InvalidAction(
                "double down is only allowed on first two cards".to_string(),
            ));
        }

        let card = self.deck.draw()?;
        self.player_hand.add(card);
        self.history.push(Action::Double { card });

        if self.player_hand.is_bust() {
            self.status = GameStatus::PlayerBust;
        } else {
            // Player must stand after doubling.
            self.status = GameStatus::DealerTurn;
            self.dealer_play()?;
            self.resolve();
        }

        Ok(card)
    }

    /// Player splits a pair into two hands.
    ///
    /// **Note**: In this simplified v1, split creates a second hand but
    /// only the first hand is played (the second is auto-stood).
    /// Full multi-hand split is a future enhancement.
    pub fn split(&mut self) -> Result<(), GameError> {
        self.require_playing()?;

        if !self.player_hand.is_pair() {
            return Err(GameError::CannotSplit);
        }

        // For v1, we just reject split with a descriptive message.
        // Full split requires multi-hand state management.
        Err(GameError::InvalidAction(
            "split is not yet implemented in v1".to_string(),
        ))
    }

    /// The player's hand value.
    pub fn player_value(&self) -> HandValue {
        self.player_hand.value()
    }

    /// The dealer's hand value (full, regardless of visibility).
    pub fn dealer_value(&self) -> HandValue {
        self.dealer_hand.value()
    }

    // -- Internal -----------------------------------------------------------

    /// Verify the game is in `Playing` status; error otherwise.
    fn require_playing(&self) -> Result<(), GameError> {
        if self.status != GameStatus::Playing {
            return Err(GameError::InvalidAction(format!(
                "game is already over (status: {})",
                self.status
            )));
        }
        Ok(())
    }

    /// Dealer draws cards until reaching a hand value of ≥ 17.
    fn dealer_play(&mut self) -> Result<(), GameError> {
        while self.dealer_hand.value().value < 17 {
            let card = self.deck.draw()?;
            self.dealer_hand.add(card);
            self.history.push(Action::DealerHit { card });
        }
        self.history.push(Action::DealerStand);
        Ok(())
    }

    /// Compare hands and set the final game status.
    fn resolve(&mut self) {
        let dealer_val = self.dealer_hand.value().value;
        let player_val = self.player_hand.value().value;

        if self.dealer_hand.is_bust() {
            self.status = GameStatus::DealerBust;
        } else if player_val > dealer_val {
            self.status = GameStatus::PlayerWins;
        } else if dealer_val > player_val {
            self.status = GameStatus::DealerWins;
        } else {
            self.status = GameStatus::Push;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_deals_four_cards() {
        let game = Game::new().expect("should create game");
        assert_eq!(game.player_hand().len(), 2, "player should have 2 cards");
        assert_eq!(game.dealer_hand().len(), 2, "dealer should have 2 cards");
        assert_eq!(game.deck_remaining(), 48, "deck should have 48 remaining");
    }

    #[test]
    fn test_new_game_has_deal_in_history() {
        let game = Game::new().expect("should create game");
        assert_eq!(game.history().len(), 1);
        assert_eq!(game.history()[0], Action::Deal);
    }

    #[test]
    fn test_new_game_generates_unique_ids() {
        let game1 = Game::new().expect("game 1");
        let game2 = Game::new().expect("game 2");
        assert_ne!(game1.id(), game2.id());
    }

    #[test]
    fn test_hit_draws_card() {
        let mut game = Game::new().expect("should create game");
        // Skip if this happened to be a blackjack.
        if game.status() == GameStatus::Blackjack {
            return;
        }
        let card = game.hit().expect("should hit");
        assert_eq!(game.player_hand().len(), 3);
        assert_eq!(game.deck_remaining(), 47);
        // The card drawn should be the last card in the hand.
        assert_eq!(*game.player_hand().cards().last().unwrap(), card);
    }

    #[test]
    fn test_hit_records_action() {
        let mut game = Game::new().expect("should create game");
        if game.status() == GameStatus::Blackjack {
            return;
        }
        let card = game.hit().expect("should hit");
        assert!(game.history().contains(&Action::Hit { card }));
    }

    #[test]
    fn test_stand_triggers_dealer_and_resolves() {
        let mut game = Game::new().expect("should create game");
        if game.status() == GameStatus::Blackjack {
            return;
        }
        let status = game.stand().expect("should stand");
        assert!(status.is_terminal(), "game should be over after stand");
        assert!(game.history().contains(&Action::Stand));
        assert!(game.history().contains(&Action::DealerStand));
    }

    #[test]
    fn test_cannot_hit_after_stand() {
        let mut game = Game::new().expect("should create game");
        if game.status() == GameStatus::Blackjack {
            return;
        }
        game.stand().expect("should stand");
        assert!(game.hit().is_err(), "cannot hit after standing");
    }

    #[test]
    fn test_cannot_hit_after_blackjack() {
        // We can't guarantee a blackjack, but we can test the status check.
        let mut game = Game::new().expect("should create game");
        if game.status() == GameStatus::Blackjack {
            assert!(game.hit().is_err(), "cannot hit after natural blackjack");
        }
    }

    #[test]
    fn test_double_down_on_first_two_cards() {
        let mut game = Game::new().expect("should create game");
        if game.status() == GameStatus::Blackjack {
            return;
        }
        let card = game.double_down().expect("should double down");
        assert_eq!(game.player_hand().len(), 3);
        // Game should be terminal (either bust or resolved).
        assert!(
            game.status().is_terminal(),
            "game should be over after double down"
        );
        assert!(
            game.history()
                .iter()
                .any(|a| matches!(a, Action::Double { card: c } if *c == card))
        );
    }

    #[test]
    fn test_double_down_not_allowed_after_hit() {
        let mut game = Game::new().expect("should create game");
        if game.status() == GameStatus::Blackjack {
            return;
        }
        game.hit().expect("should hit");
        if game.status().is_terminal() {
            return; // busted
        }
        assert!(
            game.double_down().is_err(),
            "double down not allowed after hit"
        );
    }

    #[test]
    fn test_available_actions_during_play() {
        let game = Game::new().expect("should create game");
        if game.status() == GameStatus::Blackjack {
            return;
        }
        let actions = game.available_actions();
        assert!(actions.contains(&"hit".to_string()));
        assert!(actions.contains(&"stand".to_string()));
        assert!(actions.contains(&"double".to_string()));
    }

    #[test]
    fn test_no_actions_when_terminal() {
        let mut game = Game::new().expect("should create game");
        if game.status() == GameStatus::Blackjack {
            assert!(game.available_actions().is_empty());
            return;
        }
        game.stand().expect("should stand");
        assert!(game.available_actions().is_empty());
    }

    #[test]
    fn test_dealer_visible_cards_during_play() {
        let game = Game::new().expect("should create game");
        if game.status() == GameStatus::Blackjack {
            return;
        }
        let visible = game.dealer_visible_cards();
        assert_eq!(visible.len(), 2);
        assert!(visible[0].is_some(), "first card should be visible");
        assert!(visible[1].is_none(), "hole card should be hidden");
    }

    #[test]
    fn test_dealer_all_cards_visible_after_stand() {
        let mut game = Game::new().expect("should create game");
        if game.status() == GameStatus::Blackjack {
            return;
        }
        game.stand().expect("should stand");
        let visible = game.dealer_visible_cards();
        assert!(
            visible.iter().all(|c| c.is_some()),
            "all dealer cards should be visible after stand"
        );
    }

    #[test]
    fn test_play_full_game_hit_then_stand() {
        // Play a complete game: hit once, then stand.
        let mut game = Game::new().expect("should create game");
        if game.status() == GameStatus::Blackjack {
            return;
        }

        game.hit().expect("should hit");
        if game.status().is_terminal() {
            return; // busted on hit
        }

        let status = game.stand().expect("should stand");
        assert!(status.is_terminal());

        // Verify the status is one of the valid terminal states.
        assert!(matches!(
            status,
            GameStatus::PlayerWins
                | GameStatus::DealerWins
                | GameStatus::Push
                | GameStatus::DealerBust
        ));
    }

    #[test]
    fn test_split_not_implemented_returns_error() {
        let mut game = Game::new().expect("should create game");
        if game.status() == GameStatus::Blackjack {
            return;
        }
        // Regardless of hand, split returns an error in v1.
        let result = game.split();
        assert!(result.is_err());
    }
}
