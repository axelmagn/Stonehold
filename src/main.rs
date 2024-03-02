use game::Game;

mod constants;
mod game;

#[macroquad::main("Stonehold")]
async fn main() {
    let mut game = Game::load().await.expect("Could not load game.");
    game.run().await.expect("Error during game execution.");
}
