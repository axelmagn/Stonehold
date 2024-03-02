use crate::{
    constants::{PLAYER_SPRITE_ID, TILESET_MAP_PATH, TILESET_TEXTURE_PATH, TILE_MAP_JSON_PATH},
    player::Player,
};
use anyhow::Result;
use futures::try_join;
use macroquad::{
    camera::set_default_camera,
    color::DARKGRAY,
    file::load_string,
    math::Rect,
    texture::{load_texture, FilterMode},
    window::{clear_background, next_frame, screen_height, screen_width},
};
use macroquad_tiled::{load_map, Map};

pub struct Game {
    /// map of dungeon tiles
    tile_map: Map,
    player: Player,
}

impl Game {
    pub fn new(tile_map: Map) -> Self {
        Self {
            tile_map,
            player: Player::new(),
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
            self.handle_inputs();
            self.draw();
            next_frame().await
        }
    }

    fn handle_inputs(&mut self) {}

    fn draw(&self) {
        clear_background(DARKGRAY);
        // TODO(axelmagn): custom camera
        set_default_camera();

        // draw map
        self.tile_map.draw_tiles(
            "terrain",
            // TODO(axelmagn): get from function
            Rect::new(0., 0., screen_width(), screen_height()),
            None,
        );

        // draw player
        self.player.draw(&self.tile_map);
    }
}
