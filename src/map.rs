use anyhow::Result;
use futures::try_join;
use macroquad::{
    file::load_string,
    math::{Rect, UVec2},
    texture::{load_texture, FilterMode},
};
use macroquad_tiled::Map as TileMap;
use macroquad_tiled::{load_map, TileSet};
use rapier2d::{
    geometry::{ColliderBuilder, ColliderHandle, ColliderSet},
    na::vector,
};
use std::{collections::HashMap, iter, ops::Range};

use crate::constants::{
    SOLID_TILES, TERRAIN_MAP_ID, TILESET_MAP_ID, TILESET_MAP_PATH, TILESET_TEXTURE_PATH,
    TILE_MAP_JSON_PATH,
};

pub mod mapgen;

pub struct Map {
    /// tile map loaded from TilEd
    pub tile_map: TileMap,

    /// physics collider handles
    pub colliders: HashMap<UVec2, ColliderHandle>,

    /// bitmask of which tiles are solid
    pub solid_tile_mask: Vec<bool>,
}

impl Map {
    pub fn new(tile_map: TileMap) -> Self {
        let solid_tile_mask =
            Self::create_solid_tile_mask(&tile_map.tilesets[TILESET_MAP_ID], SOLID_TILES);

        Self {
            tile_map,
            colliders: HashMap::new(),
            solid_tile_mask,
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
        let width = self.tile_map.layers[TERRAIN_MAP_ID].width as f32;
        let height = self.tile_map.layers[TERRAIN_MAP_ID].height as f32;
        self.tile_map.draw_tiles(
            TERRAIN_MAP_ID,
            // TODO(axelmagn): get from function
            Rect::new(0., 0., width, height),
            None,
        );
    }

    pub fn init_colliders(&mut self, collider_set: &mut ColliderSet) {
        for (x, y, tile) in self.tile_map.tiles(TERRAIN_MAP_ID, None) {
            if let Some(tile) = tile {
                if self.is_tile_solid(tile.id) {
                    let coord = UVec2::new(x, y);
                    let collider = ColliderBuilder::cuboid(0.5, 0.5)
                        .translation(vector![x as f32 + 0.5, y as f32 + 0.5])
                        .build();
                    self.colliders.insert(coord, collider_set.insert(collider));
                }
            }
        }
    }

    /// Calculate which tiles are solid
    fn create_solid_tile_mask(tileset: &TileSet, solid_tile_ranges: &[Range<u32>]) -> Vec<bool> {
        // ugly calculation because the library authors couldn't bother to  store the tilecount field
        let tile_count: i32 = (tileset.texture.height() as i32 + tileset.spacing
            - 2 * tileset.margin)
            / (tileset.tileheight + tileset.spacing)
            * tileset.columns as i32;
        let mut out: Vec<bool> = iter::repeat(false).take(tile_count as usize).collect();
        for range in solid_tile_ranges {
            for i in range.clone() {
                out[i as usize] = true;
            }
        }
        out
    }

    fn is_tile_solid(&self, tile_id: u32) -> bool {
        self.solid_tile_mask[tile_id as usize]
    }
}
