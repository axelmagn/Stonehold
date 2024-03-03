use crate::{
    camera::{create_screen_camera, create_world_camera},
    constants::{TILESET_MAP_PATH, TILESET_TEXTURE_PATH, TILE_MAP_JSON_PATH},
    physics::Physics,
    player::Player,
};
use anyhow::Result;
use futures::try_join;
use macroquad::{
    camera::{set_camera, Camera2D},
    color::{DARKGRAY, WHITE},
    file::load_string,
    math::{vec2, Rect},
    texture::{draw_texture_ex, load_texture, DrawTextureParams, FilterMode},
    window::{clear_background, next_frame},
};
use macroquad_tiled::{load_map, Map};

pub struct Game {
    tile_map: Map,
    player: Player,
    physics: Physics,

    /// Worldspace camera (tile units, render_target)
    world_camera: Camera2D,
    /// Screenspace camera (screen pixel units)
    screen_camera: Camera2D,
}

impl Game {
    pub fn new(tile_map: Map) -> Self {
        Self {
            tile_map,
            player: Player::new(),
            physics: Physics::default(),
            world_camera: create_world_camera(),
            screen_camera: create_screen_camera(),
        }
    }

    pub async fn load() -> Result<Self> {
        // load assets concurrently for faster load times
        let (tile_texture, tile_map_json) = try_join!(
            load_texture(TILESET_TEXTURE_PATH),
            load_string(TILE_MAP_JSON_PATH)
        )?;

        // we want tiles to have crisp pixels
        tile_texture.set_filter(FilterMode::Nearest);

        // construct tile map from loaded assets
        let tile_map = load_map(&tile_map_json, &[(TILESET_MAP_PATH, tile_texture)], &[])?;

        Ok(Self::new(tile_map))
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

        // update world camera to follow player
        self.world_camera.target = self.player.position;
    }

    fn draw(&self) {
        clear_background(DARKGRAY);

        set_camera(&self.world_camera);

        // draw map
        {
            let width = self.tile_map.layers["terrain"].width as f32;
            let height = self.tile_map.layers["terrain"].height as f32;
            self.tile_map.draw_tiles(
                "terrain",
                // TODO(axelmagn): get from function
                Rect::new(0., 0., width, height),
                None,
            );
        }

        // draw player
        self.player.draw(&self.tile_map);

        // draw full screen quad with previously rendered screen
        set_camera(&self.screen_camera);

        draw_texture_ex(
            &self
                .world_camera
                .render_target
                .as_ref()
                .expect("world camera missing render target")
                .texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(1., 1.)),
                ..Default::default()
            },
        )
    }
}
