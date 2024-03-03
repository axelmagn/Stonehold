use crate::{
    camera::Cameras, constants::PLAYER_START_POS, map::Map, physics::Physics, player::Player,
};
use anyhow::Result;
use macroquad::{
    camera::set_camera,
    color::{Color, DARKGRAY, WHITE},
    text::draw_text,
    time::get_fps,
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

    pub fn setup(&mut self) {
        self.map.init_colliders(&mut self.physics.colliders);

        self.player.init_physics(
            PLAYER_START_POS,
            &mut self.physics.colliders,
            &mut self.physics.bodies,
        );
    }

    pub async fn run(&mut self) -> Result<()> {
        self.setup();
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
        self.player.update(&mut self.physics);

        // tick physics
        self.physics.step();
        self.player.post_physics(&mut self.physics);

        // update cameras (position on player, etc)
        self.cameras.update(self.player.position);
    }

    fn draw(&self) {
        clear_background(DARKGRAY);
        self.draw_world();
        self.draw_ui();
        self.draw_screen();
    }

    fn draw_world(&self) {
        // setup drawing for worldspace
        set_camera(&self.cameras.world_camera);

        // draw map
        self.map.draw();

        // draw player
        self.player.draw(&self.map.tile_map);
    }

    fn draw_ui(&self) {
        // setup drawing for UI space
        set_camera(&self.cameras.ui_camera);
        clear_background(Color::new(0., 0., 0., 0.));
        draw_text(&format!("FPS: {:4.0}", get_fps()), 10., 20., 20., WHITE);
        self.player.draw_ui(&self.physics);
    }

    fn draw_screen(&self) {
        // draw full screen quad with previously rendered screen
        set_camera(&self.cameras.screen_camera);
        self.cameras.draw_world_render_to_screen();
        self.cameras.draw_ui_render_to_screen();
    }
}
