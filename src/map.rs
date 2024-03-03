use anyhow::Result;
use futures::try_join;
use macroquad::{
    file::load_string,
    math::{Rect, UVec2},
    texture::{load_texture, FilterMode},
};
use macroquad_tiled::load_map;
use macroquad_tiled::Map as TiledMap;
use rapier2d::geometry::ColliderHandle;
use std::collections::HashMap;

use crate::constants::{TILESET_MAP_PATH, TILESET_TEXTURE_PATH, TILE_MAP_JSON_PATH};

pub struct Map {
    pub tile_map: TiledMap,
    pub colliders: HashMap<UVec2, ColliderHandle>,
}

impl Map {
    pub fn new(tile_map: TiledMap) -> Self {
        Self {
            tile_map,
            colliders: HashMap::new(),
        }
    }

    /// Load the map from a constant path
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

    /// draw the map in worldspace
    pub fn draw(&self) {
        let width = self.tile_map.layers["terrain"].width as f32;
        let height = self.tile_map.layers["terrain"].height as f32;
        self.tile_map.draw_tiles(
            "terrain",
            // TODO(axelmagn): get from function
            Rect::new(0., 0., width, height),
            None,
        );
    }
}
