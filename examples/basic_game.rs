//! Basic programmatic usage of the Blackjack engine.
//!
//! Run with: `cargo run --example basic_game`

use rumenx_blackjack::engine::game::Game;

fn main() {
    println!("=== Blackjack Engine Demo ===\n");

    let mut game = Game::new().expect("should deal a new game");

    println!("Game ID: {}", game.id());
    println!(
        "Player hand: {} (value: {})",
        format_cards(game.player_hand().cards()),
        game.player_hand().value().value,
    );
    println!(
        "Dealer showing: {}",
        game.dealer_hand()
            .cards()
            .first()
            .map_or("?".to_string(), |c| c.to_string()),
    );
    println!("Status: {}\n", game.status());

    if game.status().is_terminal() {
        println!("Natural Blackjack! 🎉");
        return;
    }

    // Hit once.
    println!("--- Hit ---");
    let card = game.hit().expect("should hit");
    println!("Drew: {card}");
    println!(
        "Player hand: {} (value: {})",
        format_cards(game.player_hand().cards()),
        game.player_hand().value().value,
    );
    println!("Status: {}\n", game.status());

    if game.status().is_terminal() {
        println!("Bust! 💥");
        show_result(&game);
        return;
    }

    // Stand.
    println!("--- Stand ---");
    let status = game.stand().expect("should stand");
    println!(
        "Dealer hand: {} (value: {})",
        format_cards(game.dealer_hand().cards()),
        game.dealer_hand().value().value,
    );
    println!("Result: {status}\n");

    show_result(&game);
}

fn format_cards(cards: &[rumenx_blackjack::engine::types::Card]) -> String {
    cards
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn show_result(game: &Game) {
    println!("=== Game Over ===");
    println!(
        "Player: {} ({})",
        format_cards(game.player_hand().cards()),
        game.player_hand().value().value,
    );
    println!(
        "Dealer: {} ({})",
        format_cards(game.dealer_hand().cards()),
        game.dealer_hand().value().value,
    );
    println!("Outcome: {}", game.status());

    println!("\nHistory:");
    for action in game.history() {
        println!("  • {action}");
    }
}
