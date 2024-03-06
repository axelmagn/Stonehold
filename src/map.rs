use anyhow::Result;
use futures::try_join;
use macroquad::{
    file::load_string,
    math::{uvec2, Rect, UVec2},
    rand::gen_range,
    texture::{load_texture, FilterMode},
};
use macroquad_tiled::{load_map, Tile, TileSet};
use macroquad_tiled::{Layer, Map as TileMap};
use rapier2d::{
    geometry::{ColliderBuilder, ColliderHandle, ColliderSet},
    na::vector,
};
use std::{collections::HashMap, iter, ops::Range};

use crate::constants::{
    SOLID_TILES, TERRAIN_MAP_ID, TILESET_MAP_ID, TILESET_MAP_PATH, TILESET_TEXTURE_PATH,
    TILE_MAP_JSON_PATH,
};

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
            Self::create_solid_tile_mask(&tile_map.tilesets[TILESET_MAP_ID], &SOLID_TILES);

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

pub struct MapGenerator {
    pub ground_tile_id: u32,
    pub wall_tile_id: u32,
    pub tileset_id: String,

    pub size: UVec2,
    pub min_room_size: UVec2,
    pub max_room_size: UVec2,
    pub max_room_count: u32,
}

impl MapGenerator {
    pub fn generate_layer(&self) -> Layer {
        let mut layer = Layer {
            width: self.size.x,
            height: self.size.y,
            ..Default::default()
        };

        // fill layer with wall
        for _ in 0..(self.size.x * self.size.y) {
            let wall_tile = Tile {
                id: self.wall_tile_id,
                tileset: self.tileset_id.clone(),
                attrs: String::new(),
            };
            layer.data.push(Some(wall_tile));
        }

        // generate rooms
        for _ in 0..self.max_room_count {
            let width =
                gen_range(self.min_room_size.x, self.max_room_size.x + 1).max(layer.width - 1);
            let height =
                gen_range(self.min_room_size.y, self.max_room_size.y + 1).max(layer.height - 1);

            let max_x = layer.width - width;
            let max_y = layer.height - height;

            let x = gen_range(0, max_x);
            let y = gen_range(0, max_y);

            self.generate_room(&mut layer, uvec2(x, y), uvec2(width, height));
        }

        layer
    }

    pub fn generate_room(&self, layer: &mut Layer, dest: UVec2, size: UVec2) {
        for x in dest.x..(dest.x + size.x) {
            for y in dest.y..(dest.y + size.y) {
                let i = y * layer.width + x;
                let tile = Tile {
                    id: self.ground_tile_id,
                    tileset: self.tileset_id.clone(),
                    attrs: String::new(),
                };
                layer.data[i as usize] = Some(tile);
            }
        }
    }
}
