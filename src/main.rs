use game::Game;
use menus::MainMenu;

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
    // TODO(axelmagn): draw splash screen while game is loading
    let main_menu = MainMenu::new();
    main_menu.run().await.expect("Error during main menu");

    let mut game = Game::load().await.expect("Could not load game.");
    game.run().await.expect("Error during game execution.");
}
