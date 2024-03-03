use game::Game;

mod camera;
mod constants;
mod game;
mod map;
mod physics;
mod player;

#[macroquad::main("Stonehold")]
async fn main() {
    // TODO(axelmagn): draw splash screen while game is loading
    let mut game = Game::load().await.expect("Could not load game.");
    game.run().await.expect("Error during game execution.");
}
