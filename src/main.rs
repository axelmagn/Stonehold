use anyhow::Result;
use game::Game;

mod camera;
mod character;
mod constants;
mod door;
mod game;
mod map;
mod menus;
mod physics;

#[macroquad::main("Stonehold")]
async fn main() {
    loop {
        let mut game = Game::load().await.expect("Could not load game.");
        game.run_state()
            .await
            .expect("Error during game execution.");
    }
}
