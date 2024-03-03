use crate::{camera::Cameras, map::Map, physics::Physics, player::Player};
use anyhow::Result;
use macroquad::{
    camera::set_camera,
    color::DARKGRAY,
    window::{clear_background, next_frame},
};

pub struct Game {
    pub map: Map,
    pub player: Player,
    pub physics: Physics,
    pub cameras: Cameras,
}

impl Game {
    pub fn new(map: Map) -> Self {
        Self {
            map,
            player: Player::new(),
            physics: Physics::default(),
            cameras: Cameras::new(),
        }
    }

    pub async fn load() -> Result<Self> {
        let map = Map::load().await?;
        Ok(Self::new(map))
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            self.collect_inputs();
            self.update();
            self.draw();
            next_frame().await
        }
    }

    fn collect_inputs(&mut self) {
        self.player.collect_inputs();
    }

    fn update(&mut self) {
        // update player
        self.player.update();

        // tick physics
        self.physics.step();

        // update cameras (position on player, etc)
        self.cameras.update(self.player.position);
    }

    fn draw(&self) {
        clear_background(DARKGRAY);

        // setup drawing for worldspace
        set_camera(&self.cameras.world_camera);

        // draw map
        self.map.draw();

        // draw player
        self.player.draw(&self.map.tile_map);

        // draw full screen quad with previously rendered screen
        set_camera(&self.cameras.screen_camera);
        self.cameras.draw_world_render_to_screen();
    }
}
