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
    CORRIDOR_PADDING, FACADE_CENTER_TILE_ID, FACADE_LEFT_TILE_ID, FACADE_RIGHT_TILE_ID,
    GROUND_01_TILE_ID, MAX_ROOM_COUNT, MAX_ROOM_SIZE, MIN_ROOM_SIZE, SOLID_TILES, TERRAIN_MAP_ID,
    TILESET_MAP_ID, TILESET_MAP_PATH, TILESET_TEXTURE_PATH, TILE_MAP_JSON_PATH, WALL_01_TILE_ID,
    WALL_DOWN_TILE_ID, WALL_INNER_DL_ID, WALL_INNER_DR_ID, WALL_INNER_UL_ID, WALL_INNER_UR_ID,
    WALL_LEFT_TILE_ID, WALL_OUTER_DL_ID, WALL_OUTER_DR_ID, WALL_OUTER_UL_ID, WALL_OUTER_UR_ID,
    WALL_RIGHT_TILE_ID, WALL_TILE_IDS, WALL_UP_TILE_ID,
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
    pub corridor_padding: Option<u32>,
}

impl MapGenerator {
    pub fn new(size: UVec2) -> Self {
        MapGenerator {
            ground_tile_id: GROUND_01_TILE_ID,
            wall_tile_id: WALL_01_TILE_ID,
            tileset_id: TILESET_MAP_ID.into(),
            size,
            min_room_size: MIN_ROOM_SIZE,
            max_room_size: MAX_ROOM_SIZE,
            max_room_count: MAX_ROOM_COUNT,
            corridor_padding: CORRIDOR_PADDING,
        }
    }

    pub fn generate_layer(&self) -> (Layer, Vec<Rect>) {
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
        let mut rooms: Vec<Rect> = Vec::new();
        for _ in 0..self.max_room_count {
            let width =
                gen_range(self.min_room_size.x, self.max_room_size.x + 1).min(layer.width - 1);
            let height =
                gen_range(self.min_room_size.y, self.max_room_size.y + 1).min(layer.height - 1);

            let max_x = layer.width - width - 1;
            let max_y = layer.height - height - 1;

            let x = gen_range(1, max_x);
            let y = gen_range(1, max_y);

            let room = Rect::new(x as f32, y as f32, width as f32, height as f32);
            // check for collisions
            let overlap_found = rooms.iter().any(|prior| room.overlaps(prior));
            if overlap_found {
                continue;
            }

            self.generate_room(&mut layer, uvec2(x, y), uvec2(width, height));

            // draw corridor from last room
            if let Some(last_room) = rooms.last() {
                // let horizontal_first = gen_range(0, 2) > 0;
                let horizontal_first = true;

                let last_x = last_room.center().x as u32;
                let last_y = last_room.center().y as u32;
                let room_x = room.center().x as u32;
                let room_y = room.center().y as u32;

                if horizontal_first {
                    self.generate_corridor_horizontal(
                        &mut layer,
                        last_x,
                        room_x,
                        last_y,
                        self.corridor_padding,
                    );
                    self.generate_corridor_vertical(
                        &mut layer,
                        room_x,
                        last_y,
                        room_y,
                        self.corridor_padding,
                    );
                } else {
                    self.generate_corridor_vertical(
                        &mut layer,
                        last_x,
                        last_y,
                        room_y,
                        self.corridor_padding,
                    );
                    self.generate_corridor_horizontal(
                        &mut layer,
                        last_x,
                        room_x,
                        room_y,
                        self.corridor_padding,
                    );
                }
            }

            rooms.push(room);
        }

        self.rewrite_wall_details(&mut layer);

        (layer, rooms)
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

    pub fn generate_corridor_horizontal(
        &self,
        layer: &mut Layer,
        src_x: u32,
        dest_x: u32,
        y: u32,
        padding: Option<u32>,
    ) {
        let padding = padding.unwrap_or(0);
        let (src_x, dest_x) = (src_x.min(dest_x), src_x.max(dest_x));

        for y in (y - padding)..=(y + padding) {
            for x in (src_x - padding)..=(dest_x + padding) {
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

    pub fn generate_corridor_vertical(
        &self,
        layer: &mut Layer,
        x: u32,
        src_y: u32,
        dest_y: u32,
        padding: Option<u32>,
    ) {
        let padding = padding.unwrap_or(1);
        let (src_y, dest_y) = (src_y.min(dest_y), src_y.max(dest_y));

        for x in (x - padding)..=(x + padding) {
            for y in (src_y - padding)..=(dest_y + padding) {
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

    pub fn rewrite_wall_details(&self, layer: &mut Layer) {
        // rewrite wall patterns that we don't have detail tiles for
        let mut needs_scan = true;
        while needs_scan {
            for x in 0..layer.width {
                for y in 0..layer.height {
                    needs_scan = false;
                    needs_scan = self.try_rewrite_thin_horizontal_wall(x, y, layer) || needs_scan;
                    needs_scan = self.try_rewrite_thin_vertical_wall(x, y, layer) || needs_scan;
                    needs_scan =
                        self.try_rewrite_double_corner_horizontal(x, y, layer) || needs_scan;
                    needs_scan = self.try_rewrite_double_corner_vertical(x, y, layer) || needs_scan;
                }
            }
        }

        // rewrite walls with detail
        for x in 0..layer.width {
            for y in 0..layer.height {
                self.try_rewrite_inner_ul_wall(x, y, layer);
                self.try_rewrite_inner_ur_wall(x, y, layer);
                self.try_rewrite_inner_dl_wall(x, y, layer);
                self.try_rewrite_inner_dr_wall(x, y, layer);
                self.try_rewrite_outer_ul_wall(x, y, layer);
                self.try_rewrite_outer_ur_wall(x, y, layer);
                self.try_rewrite_outer_dl_wall(x, y, layer);
                self.try_rewrite_outer_dr_wall(x, y, layer);
            }
        }
        for x in 0..layer.width {
            for y in 0..layer.height {
                self.try_rewrite_left_wall(x, y, layer);
                self.try_rewrite_right_wall(x, y, layer);
            }
        }
        for x in 0..layer.width {
            for y in 0..layer.height {
                self.try_rewrite_bottom_wall(x, y, layer);
                self.try_rewrite_top_wall(x, y, layer);
            }
        }

        for x in 0..layer.width {
            for y in 0..layer.height {
                self.try_rewrite_center_facades(x, y, layer);
                self.try_rewrite_left_facades(x, y, layer);
                self.try_rewrite_right_facades(x, y, layer);
            }
        }
    }

    pub fn try_rewrite_thin_horizontal_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width || y >= layer.height - 2 {
            return false;
        }

        let i0 = xytoi(x, y, layer);
        let i1 = xytoi(x, y + 1, layer);
        let i2 = xytoi(x, y + 2, layer);

        if let (&Some(tile0), &Some(tile1), &Some(tile2)) = (
            &layer.data[i0].as_ref(),
            &layer.data[i1].as_ref(),
            &layer.data[i2].as_ref(),
        ) {
            if tile0.id != GROUND_01_TILE_ID
                || tile1.id != WALL_01_TILE_ID
                || tile2.id != GROUND_01_TILE_ID
            {
                return false;
            }
        }

        layer.data[i0] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_thin_vertical_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width - 2 || y >= layer.height {
            return false;
        }

        let i0 = xytoi(x, y, layer);
        let i1 = xytoi(x + 1, y, layer);
        let i2 = xytoi(x + 2, y, layer);

        if let (&Some(tile0), &Some(tile1), &Some(tile2)) = (
            &layer.data[i0].as_ref(),
            &layer.data[i1].as_ref(),
            &layer.data[i2].as_ref(),
        ) {
            if tile0.id != GROUND_01_TILE_ID
                || tile1.id != WALL_01_TILE_ID
                || tile2.id != GROUND_01_TILE_ID
            {
                return false;
            }
        }

        layer.data[i0] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_double_corner_horizontal(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width - 2 || y >= layer.height - 1 {
            return false;
        }

        let i00 = xytoi(x, y, layer);
        let i10 = xytoi(x + 1, y, layer);
        let i20 = xytoi(x + 2, y, layer);
        let i01 = xytoi(x, y + 1, layer);
        let i11 = xytoi(x + 1, y + 1, layer);
        let i21 = xytoi(x + 2, y + 1, layer);

        if let (
            &Some(tile00),
            &Some(tile10),
            &Some(tile20),
            &Some(tile01),
            &Some(tile11),
            &Some(tile21),
        ) = (
            &layer.data[i00].as_ref(),
            &layer.data[i10].as_ref(),
            &layer.data[i20].as_ref(),
            &layer.data[i01].as_ref(),
            &layer.data[i11].as_ref(),
            &layer.data[i21].as_ref(),
        ) {
            if !(tile00.id == WALL_01_TILE_ID
                && tile10.id == WALL_01_TILE_ID
                && tile20.id == GROUND_01_TILE_ID
                && tile01.id == GROUND_01_TILE_ID
                && tile11.id == WALL_01_TILE_ID
                && tile21.id == WALL_01_TILE_ID)
                || (tile00.id == GROUND_01_TILE_ID
                    && tile10.id == WALL_01_TILE_ID
                    && tile20.id == WALL_01_TILE_ID
                    && tile01.id == WALL_01_TILE_ID
                    && tile11.id == WALL_01_TILE_ID
                    && tile21.id == GROUND_01_TILE_ID)
            {
                return false;
            }
        }

        layer.data[i00] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        layer.data[i10] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        layer.data[i20] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        layer.data[i01] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        layer.data[i11] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        layer.data[i21] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }
    pub fn try_rewrite_double_corner_vertical(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width - 1 || y >= layer.height - 2 {
            return false;
        }

        let i00 = xytoi(x, y, layer);
        let i10 = xytoi(x, y + 1, layer);
        let i20 = xytoi(x, y + 2, layer);
        let i01 = xytoi(x + 1, y, layer);
        let i11 = xytoi(x + 1, y + 1, layer);
        let i21 = xytoi(x + 1, y + 2, layer);

        if let (
            &Some(tile00),
            &Some(tile10),
            &Some(tile20),
            &Some(tile01),
            &Some(tile11),
            &Some(tile21),
        ) = (
            &layer.data[i00].as_ref(),
            &layer.data[i10].as_ref(),
            &layer.data[i20].as_ref(),
            &layer.data[i01].as_ref(),
            &layer.data[i11].as_ref(),
            &layer.data[i21].as_ref(),
        ) {
            if !(tile00.id == WALL_01_TILE_ID
                && tile10.id == WALL_01_TILE_ID
                && tile20.id == GROUND_01_TILE_ID
                && tile01.id == GROUND_01_TILE_ID
                && tile11.id == WALL_01_TILE_ID
                && tile21.id == WALL_01_TILE_ID)
                || (tile00.id == GROUND_01_TILE_ID
                    && tile10.id == WALL_01_TILE_ID
                    && tile20.id == WALL_01_TILE_ID
                    && tile01.id == WALL_01_TILE_ID
                    && tile11.id == WALL_01_TILE_ID
                    && tile21.id == GROUND_01_TILE_ID)
            {
                return false;
            }
        }

        layer.data[i00] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        layer.data[i10] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        layer.data[i20] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        layer.data[i01] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        layer.data[i11] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        layer.data[i21] = Some(Tile {
            id: WALL_01_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_top_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width || y >= layer.height - 1 {
            return false;
        }

        let i0 = xytoi(x, y, layer);
        let i1 = xytoi(x, y + 1, &layer);

        if let (&Some(tile0), &Some(tile1)) = (&layer.data[i0].as_ref(), &layer.data[i1].as_ref()) {
            if tile0.id != WALL_01_TILE_ID || tile1.id != GROUND_01_TILE_ID {
                return false;
            }
        }

        layer.data[i0] = Some(Tile {
            id: WALL_UP_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_bottom_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width || y >= layer.height - 1 {
            return false;
        }

        let i0 = xytoi(x, y, layer);
        let i1 = xytoi(x, y + 1, &layer);

        if let (&Some(tile0), &Some(tile1)) = (&layer.data[i0].as_ref(), &layer.data[i1].as_ref()) {
            if tile0.id != GROUND_01_TILE_ID || tile1.id != WALL_01_TILE_ID {
                return false;
            }
        }

        layer.data[i1] = Some(Tile {
            id: WALL_DOWN_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_left_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width - 1 || y >= layer.height {
            return false;
        }

        let i0 = xytoi(x, y, layer);
        let i1 = xytoi(x + 1, y, &layer);

        if let (&Some(tile0), &Some(tile1)) = (&layer.data[i0].as_ref(), &layer.data[i1].as_ref()) {
            if tile0.id != WALL_01_TILE_ID || tile1.id != GROUND_01_TILE_ID {
                return false;
            }
        }

        layer.data[i0] = Some(Tile {
            id: WALL_LEFT_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_right_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width - 1 || y >= layer.height {
            return false;
        }

        let i0 = xytoi(x, y, layer);
        let i1 = xytoi(x + 1, y, &layer);

        if let (&Some(tile0), &Some(tile1)) = (&layer.data[i0].as_ref(), &layer.data[i1].as_ref()) {
            if tile0.id != GROUND_01_TILE_ID || tile1.id != WALL_01_TILE_ID {
                return false;
            }
        }

        layer.data[i1] = Some(Tile {
            id: WALL_RIGHT_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_inner_ul_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width - 1 || y >= layer.height - 1 {
            return false;
        }

        let i00 = xytoi(x, y, layer);
        let i01 = xytoi(x + 1, y, layer);
        let i10 = xytoi(x, y + 1, layer);
        let i11 = xytoi(x + 1, y + 1, layer);

        if let (&Some(tile00), &Some(tile01), &Some(tile10), &Some(tile11)) = (
            &layer.data[i00].as_ref(),
            &layer.data[i01].as_ref(),
            &layer.data[i10].as_ref(),
            &layer.data[i11].as_ref(),
        ) {
            if !WALL_TILE_IDS.contains(&tile00.id)
                || !WALL_TILE_IDS.contains(&tile01.id)
                || !WALL_TILE_IDS.contains(&tile10.id)
                || tile11.id != GROUND_01_TILE_ID
            {
                return false;
            }
        }

        layer.data[i00] = Some(Tile {
            id: WALL_INNER_UL_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_inner_ur_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width - 1 || y >= layer.height - 1 {
            return false;
        }

        let i00 = xytoi(x, y, layer);
        let i01 = xytoi(x + 1, y, layer);
        let i10 = xytoi(x, y + 1, layer);
        let i11 = xytoi(x + 1, y + 1, layer);

        if let (&Some(tile00), &Some(tile01), &Some(tile10), &Some(tile11)) = (
            &layer.data[i00].as_ref(),
            &layer.data[i01].as_ref(),
            &layer.data[i10].as_ref(),
            &layer.data[i11].as_ref(),
        ) {
            if !WALL_TILE_IDS.contains(&tile00.id)
                || !WALL_TILE_IDS.contains(&tile01.id)
                || tile10.id != GROUND_01_TILE_ID
                || !WALL_TILE_IDS.contains(&tile11.id)
            {
                return false;
            }
        }

        layer.data[i01] = Some(Tile {
            id: WALL_INNER_UR_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_inner_dl_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width - 1 || y >= layer.height - 1 {
            return false;
        }

        let i00 = xytoi(x, y, layer);
        let i01 = xytoi(x + 1, y, layer);
        let i10 = xytoi(x, y + 1, layer);
        let i11 = xytoi(x + 1, y + 1, layer);

        if let (&Some(tile00), &Some(tile01), &Some(tile10), &Some(tile11)) = (
            &layer.data[i00].as_ref(),
            &layer.data[i01].as_ref(),
            &layer.data[i10].as_ref(),
            &layer.data[i11].as_ref(),
        ) {
            if !WALL_TILE_IDS.contains(&tile00.id)
                || tile01.id != GROUND_01_TILE_ID
                || !WALL_TILE_IDS.contains(&tile10.id)
                || !WALL_TILE_IDS.contains(&tile11.id)
            {
                return false;
            }
        }

        layer.data[i10] = Some(Tile {
            id: WALL_INNER_DL_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_inner_dr_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width - 1 || y >= layer.height - 1 {
            return false;
        }

        let i00 = xytoi(x, y, layer);
        let i01 = xytoi(x + 1, y, layer);
        let i10 = xytoi(x, y + 1, layer);
        let i11 = xytoi(x + 1, y + 1, layer);

        if let (&Some(tile00), &Some(tile01), &Some(tile10), &Some(tile11)) = (
            &layer.data[i00].as_ref(),
            &layer.data[i01].as_ref(),
            &layer.data[i10].as_ref(),
            &layer.data[i11].as_ref(),
        ) {
            if tile00.id != GROUND_01_TILE_ID
                || !WALL_TILE_IDS.contains(&tile01.id)
                || !WALL_TILE_IDS.contains(&tile10.id)
                || !WALL_TILE_IDS.contains(&tile11.id)
            {
                return false;
            }
        }

        layer.data[i11] = Some(Tile {
            id: WALL_INNER_DR_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_outer_ul_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width - 1 || y >= layer.height - 1 {
            return false;
        }

        let i00 = xytoi(x, y, layer);
        let i01 = xytoi(x + 1, y, layer);
        let i10 = xytoi(x, y + 1, layer);
        let i11 = xytoi(x + 1, y + 1, layer);

        if let (&Some(tile00), &Some(tile01), &Some(tile10), &Some(tile11)) = (
            &layer.data[i00].as_ref(),
            &layer.data[i01].as_ref(),
            &layer.data[i10].as_ref(),
            &layer.data[i11].as_ref(),
        ) {
            if tile00.id != GROUND_01_TILE_ID
                || tile01.id != GROUND_01_TILE_ID
                || tile10.id != GROUND_01_TILE_ID
                || !WALL_TILE_IDS.contains(&tile11.id)
            {
                return false;
            }
        }

        layer.data[i11] = Some(Tile {
            id: WALL_OUTER_UL_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_outer_ur_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width - 1 || y >= layer.height - 1 {
            return false;
        }

        let i00 = xytoi(x, y, layer);
        let i01 = xytoi(x + 1, y, layer);
        let i10 = xytoi(x, y + 1, layer);
        let i11 = xytoi(x + 1, y + 1, layer);

        if let (&Some(tile00), &Some(tile01), &Some(tile10), &Some(tile11)) = (
            &layer.data[i00].as_ref(),
            &layer.data[i01].as_ref(),
            &layer.data[i10].as_ref(),
            &layer.data[i11].as_ref(),
        ) {
            if tile00.id != GROUND_01_TILE_ID
                || tile01.id != GROUND_01_TILE_ID
                || !WALL_TILE_IDS.contains(&tile10.id)
                || tile11.id != GROUND_01_TILE_ID
            {
                return false;
            }
        }

        layer.data[i10] = Some(Tile {
            id: WALL_OUTER_UR_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }
    pub fn try_rewrite_outer_dl_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width - 1 || y >= layer.height - 1 {
            return false;
        }

        let i00 = xytoi(x, y, layer);
        let i01 = xytoi(x + 1, y, layer);
        let i10 = xytoi(x, y + 1, layer);
        let i11 = xytoi(x + 1, y + 1, layer);

        if let (&Some(tile00), &Some(tile01), &Some(tile10), &Some(tile11)) = (
            &layer.data[i00].as_ref(),
            &layer.data[i01].as_ref(),
            &layer.data[i10].as_ref(),
            &layer.data[i11].as_ref(),
        ) {
            if tile00.id != GROUND_01_TILE_ID
                || !WALL_TILE_IDS.contains(&tile01.id)
                || tile10.id != GROUND_01_TILE_ID
                || tile11.id != GROUND_01_TILE_ID
            {
                return false;
            }
        }

        layer.data[i01] = Some(Tile {
            id: WALL_OUTER_DL_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_outer_dr_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width - 1 || y >= layer.height - 1 {
            return false;
        }

        let i00 = xytoi(x, y, layer);
        let i01 = xytoi(x + 1, y, layer);
        let i10 = xytoi(x, y + 1, layer);
        let i11 = xytoi(x + 1, y + 1, layer);

        if let (&Some(tile00), &Some(tile01), &Some(tile10), &Some(tile11)) = (
            &layer.data[i00].as_ref(),
            &layer.data[i01].as_ref(),
            &layer.data[i10].as_ref(),
            &layer.data[i11].as_ref(),
        ) {
            if !WALL_TILE_IDS.contains(&tile00.id)
                || tile01.id != GROUND_01_TILE_ID
                || tile10.id != GROUND_01_TILE_ID
                || tile11.id != GROUND_01_TILE_ID
            {
                return false;
            }
        }

        layer.data[i00] = Some(Tile {
            id: WALL_OUTER_DR_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_center_facades(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width || y >= layer.height - 1 {
            return false;
        }

        let i0 = xytoi(x, y, layer);
        let i1 = xytoi(x, y + 1, &layer);

        if let (&Some(tile0), &Some(tile1)) = (&layer.data[i0].as_ref(), &layer.data[i1].as_ref()) {
            if tile0.id != WALL_UP_TILE_ID || tile1.id != GROUND_01_TILE_ID {
                return false;
            }
        }

        layer.data[i1] = Some(Tile {
            id: FACADE_CENTER_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_left_facades(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width || y >= layer.height - 1 {
            return false;
        }

        let i0 = xytoi(x, y, layer);
        let i1 = xytoi(x, y + 1, &layer);

        if let (&Some(tile0), &Some(tile1)) = (&layer.data[i0].as_ref(), &layer.data[i1].as_ref()) {
            if tile0.id != WALL_OUTER_DL_ID || tile1.id != GROUND_01_TILE_ID {
                return false;
            }
        }

        layer.data[i1] = Some(Tile {
            id: FACADE_LEFT_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }

    pub fn try_rewrite_right_facades(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
        if x >= layer.width || y >= layer.height - 1 {
            return false;
        }

        let i0 = xytoi(x, y, layer);
        let i1 = xytoi(x, y + 1, &layer);

        if let (&Some(tile0), &Some(tile1)) = (&layer.data[i0].as_ref(), &layer.data[i1].as_ref()) {
            if tile0.id != WALL_OUTER_DR_ID || tile1.id != GROUND_01_TILE_ID {
                return false;
            }
        }

        layer.data[i1] = Some(Tile {
            id: FACADE_RIGHT_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        true
    }
}

fn xytoi(x: u32, y: u32, layer: &Layer) -> usize {
    (y * layer.width + x) as usize
}

fn itoxy(i: usize, layer: &Layer) -> UVec2 {
    // overflows be damned
    let i = i as u32;
    uvec2(i % layer.width, i / layer.width)
}
