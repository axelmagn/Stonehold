use macroquad::{
    math::{uvec2, Rect, UVec2},
    rand::gen_range,
};
use macroquad_tiled::Layer;
use macroquad_tiled::Tile;

use crate::constants::{
    CORRIDOR_PADDING, DOOR_CLEARANCE, DOOR_LEFT_CLOSED_TILE_ID, DOOR_LEFT_OPEN_TILE_ID,
    DOOR_RIGHT_CLOSED_TILE_ID, DOOR_RIGHT_OPEN_TILE_ID, FACADE_CENTER_02_TILE_ID,
    FACADE_CENTER_TILE_ID, FACADE_LEFT_TILE_ID, FACADE_RIGHT_TILE_ID, GROUND_01_TILE_ID,
    GROUND_02_TILE_ID, GROUND_03_TILE_ID, MAX_ROOM_COUNT, MAX_ROOM_SIZE, MIN_ROOM_SIZE,
    MONSTER_PIPE_CLOSED_TILE_ID, POOL_EMPTY_TILE_ID, STAIRS_LEFT_TILE_ID, STAIRS_RIGHT_TILE_ID,
    TILESET_MAP_ID, TILE_FILLER_PROB, WALL_01_TILE_ID, WALL_02_TILE_ID, WALL_03_TILE_ID,
    WALL_DOWN_TILE_ID, WALL_INNER_DL_ID, WALL_INNER_DR_ID, WALL_INNER_UL_ID, WALL_INNER_UR_ID,
    WALL_LEFT_TILE_ID, WALL_OUTER_DL_ID, WALL_OUTER_DR_ID, WALL_OUTER_UL_ID, WALL_OUTER_UR_ID,
    WALL_RIGHT_TILE_ID, WALL_TILE_IDS, WALL_UP_TILE_ID,
};

pub struct MapGenerator {
    pub ground_tile_id: u32,
    pub wall_tile_id: u32,
    pub tileset_id: String,

    pub size: UVec2,
    pub min_room_size: UVec2,
    pub max_room_size: UVec2,
    pub max_room_count: u32,
    pub corridor_padding: Option<u32>,
    pub door_clearance: u32,
}

pub struct MapGenResult {
    pub layer: Layer,
    pub rooms: Vec<Rect>,
    pub guard_doors: Vec<UVec2>,
    pub exit_door: UVec2,
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
            door_clearance: DOOR_CLEARANCE,
        }
    }

    pub fn generate_layer(&self) -> MapGenResult {
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

        // TODO: generate guard counts & locations
        let num_doors = rooms.len();

        // generate guard doors
        let mut guard_doors = Vec::new();
        for _ in 0..10 {
            guard_doors = self.generate_guard_doors(num_doors, &mut layer);
            if num_doors == guard_doors.len() {
                break;
            }
        }
        assert_eq!(num_doors, guard_doors.len());

        // generate exit door
        let exit_door = guard_doors.remove(gen_range(0, num_doors));
        self.rewrite_exit_door(exit_door, &mut layer);

        // add fillers
        self.rewrite_random_filler(
            WALL_01_TILE_ID,
            WALL_02_TILE_ID,
            TILE_FILLER_PROB,
            &mut layer,
        );
        self.rewrite_random_filler(
            WALL_01_TILE_ID,
            WALL_03_TILE_ID,
            TILE_FILLER_PROB,
            &mut layer,
        );
        self.rewrite_random_filler(
            GROUND_01_TILE_ID,
            GROUND_02_TILE_ID,
            TILE_FILLER_PROB,
            &mut layer,
        );
        self.rewrite_random_filler(
            GROUND_01_TILE_ID,
            GROUND_03_TILE_ID,
            TILE_FILLER_PROB,
            &mut layer,
        );
        self.rewrite_random_filler(
            FACADE_CENTER_TILE_ID,
            FACADE_CENTER_02_TILE_ID,
            TILE_FILLER_PROB * 10.,
            &mut layer,
        );

        MapGenResult {
            layer,
            rooms,
            guard_doors,
            exit_door,
        }
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
                    // TODO: one of these isn't working correctly. looks like maybe vertical one
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

    fn try_rewrite_thin_horizontal_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_thin_vertical_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_double_corner_horizontal(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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
                && !(tile00.id == GROUND_01_TILE_ID
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
    fn try_rewrite_double_corner_vertical(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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
                && !(tile00.id == GROUND_01_TILE_ID
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

    fn try_rewrite_top_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_bottom_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_left_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_right_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_inner_ul_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_inner_ur_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_inner_dl_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_inner_dr_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_outer_ul_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_outer_ur_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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
    fn try_rewrite_outer_dl_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_outer_dr_wall(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_center_facades(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_left_facades(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn try_rewrite_right_facades(&self, x: u32, y: u32, layer: &mut Layer) -> bool {
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

    fn rewrite_exit_door(&self, pos: UVec2, layer: &mut Layer) {
        // rewrite doors to closed
        let i = xytoi(pos.x, pos.y, layer);
        layer.data[i] = Some(Tile {
            id: MONSTER_PIPE_CLOSED_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        let i = xytoi(pos.x + 1, pos.y, layer);
        layer.data[i] = Some(Tile {
            id: DOOR_LEFT_CLOSED_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        let i = xytoi(pos.x + 2, pos.y, layer);
        layer.data[i] = Some(Tile {
            id: DOOR_RIGHT_CLOSED_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        let i = xytoi(pos.x + 3, pos.y, layer);
        layer.data[i] = Some(Tile {
            id: MONSTER_PIPE_CLOSED_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });

        // put some stairs under them
        let i = xytoi(pos.x, pos.y + 1, layer);
        layer.data[i] = Some(Tile {
            id: POOL_EMPTY_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        let i = xytoi(pos.x + 1, pos.y + 1, layer);
        layer.data[i] = Some(Tile {
            id: STAIRS_LEFT_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        let i = xytoi(pos.x + 2, pos.y + 1, layer);
        layer.data[i] = Some(Tile {
            id: STAIRS_RIGHT_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
        let i = xytoi(pos.x + 3, pos.y + 1, layer);
        layer.data[i] = Some(Tile {
            id: POOL_EMPTY_TILE_ID,
            tileset: self.tileset_id.clone(),
            attrs: String::new(),
        });
    }

    fn generate_guard_doors(&self, max_doors: usize, layer: &mut Layer) -> Vec<UVec2> {
        let mut candidates: Vec<UVec2> = Vec::new();
        for x in 0..layer.width {
            for y in 0..layer.height {
                if self.check_door_candidate(x, y, layer) {
                    candidates.push(uvec2(x, y));
                }
            }
        }

        let mut doors: Vec<UVec2> = Vec::new();
        while doors.len() < max_doors && candidates.len() > 0 {
            let pos = candidates.remove(gen_range(0, candidates.len()));

            // we have to check again, since doors are 2-wide their placements can interfere
            if !self.check_door_candidate(pos.x, pos.y, layer) {
                continue;
            }

            let i = xytoi(pos.x + 1, pos.y, layer);
            layer.data[i] = Some(Tile {
                id: DOOR_LEFT_OPEN_TILE_ID,
                tileset: self.tileset_id.clone(),
                attrs: String::new(),
            });
            let i = xytoi(pos.x + 2, pos.y, layer);
            layer.data[i] = Some(Tile {
                id: DOOR_RIGHT_OPEN_TILE_ID,
                tileset: self.tileset_id.clone(),
                attrs: String::new(),
            });

            doors.push(pos);
        }

        doors
    }

    /// Check if a location is a candidate for door placement
    fn check_door_candidate(&self, x: u32, y: u32, layer: &Layer) -> bool {
        if x + 4 > layer.width || y + self.door_clearance > layer.height {
            return false;
        }
        for x in x..(x + 4) {
            // check if we can place door on a facade
            let i = xytoi(x, y, layer);
            if let &Some(tile) = &layer.data[i].as_ref() {
                if tile.id != FACADE_CENTER_TILE_ID {
                    return false;
                }
            }

            // check if there is clearance beneath the door
            for y in (y + 1)..(y + self.door_clearance) {
                let i = xytoi(x, y, layer);
                if let &Some(tile) = &layer.data[i].as_ref() {
                    if tile.id != GROUND_01_TILE_ID {
                        return false;
                    }
                }
            }
        }

        return true;
    }

    fn rewrite_random_filler(&self, src: u32, dst: u32, prob: f32, layer: &mut Layer) -> u32 {
        let mut count = 0;

        for x in 1..(layer.width - 1) {
            for y in 1..(layer.height - 1) {
                let i = xytoi(x, y, layer);

                // match src tile
                if let &Some(tile) = &layer.data[i].as_ref() {
                    if tile.id != src {
                        continue;
                    }
                }

                // roll the dice
                let sample = gen_range(0., 1.);
                if sample >= prob {
                    continue;
                }

                // rewrite to dst
                layer.data[i] = Some(Tile {
                    id: dst,
                    tileset: self.tileset_id.clone(),
                    attrs: String::new(),
                });

                count += 1;
            }
        }

        count
    }
}

fn xytoi(x: u32, y: u32, layer: &Layer) -> usize {
    (y * layer.width + x) as usize
}

// unused inverse of xytoi (saved for a rainy;d)
fn _itoxy(i: usize, layer: &Layer) -> UVec2 {
    // overflows be damned
    let i = i as u32;
    uvec2(i % layer.width, i / layer.width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapgen_corner_tile_horizontal_performs_rewrite() {
        // create test layer
        let (width, height) = (3, 2);
        let mut layer = Layer {
            width,
            height,
            ..Default::default()
        };

        // fill with walls
        for _i in 0..(width * height) {
            layer.data.push(Some(Tile {
                id: WALL_01_TILE_ID,
                tileset: "".into(),
                attrs: "".into(),
            }));
        }

        // create mapgen
        let mapgen = MapGenerator::new(uvec2(width, height));

        // check that the rewrite does not trigger on the base case
        let did_rewrite = mapgen.try_rewrite_double_corner_horizontal(0, 0, &mut layer);
        assert_eq!(did_rewrite, false);
        for i in 0..(width * height) {
            if let &Some(tile) = &layer.data[i as usize].as_ref() {
                assert_eq!(tile.id, WALL_01_TILE_ID);
                continue;
            }
            panic!("None tile found");
        }

        // create double corner pattern (variant 1)
        let i = xytoi(0, 0, &layer);
        layer.data[i] = Some(Tile {
            id: GROUND_01_TILE_ID,
            tileset: "".into(),
            attrs: "".into(),
        });
        let i = xytoi(2, 1, &layer);
        layer.data[i] = Some(Tile {
            id: GROUND_01_TILE_ID,
            tileset: "".into(),
            attrs: "".into(),
        });

        // check that double corner variant 1 rewrites correctly
        let did_rewrite = mapgen.try_rewrite_double_corner_horizontal(0, 0, &mut layer);
        assert_eq!(did_rewrite, true);
        for i in 0..(width * height) {
            if let &Some(tile) = &layer.data[i as usize].as_ref() {
                assert_eq!(tile.id, WALL_01_TILE_ID);
                continue;
            }
            panic!("None tile found");
        }

        // create double corner pattern (variant 2)
        let i = xytoi(0, 1, &layer);
        layer.data[i] = Some(Tile {
            id: GROUND_01_TILE_ID,
            tileset: "".into(),
            attrs: "".into(),
        });
        let i = xytoi(2, 0, &layer);
        layer.data[i] = Some(Tile {
            id: GROUND_01_TILE_ID,
            tileset: "".into(),
            attrs: "".into(),
        });

        // check that double corner variant 2 rewrites correctly
        let did_rewrite = mapgen.try_rewrite_double_corner_horizontal(0, 0, &mut layer);
        assert_eq!(did_rewrite, true);
        for i in 0..(width * height) {
            if let &Some(tile) = &layer.data[i as usize].as_ref() {
                assert_eq!(tile.id, WALL_01_TILE_ID);
                continue;
            }
            panic!("None tile found");
        }
    }

    #[test]
    fn test_mapgen_corner_tile_vertical_performs_rewrite() {
        // create test layer
        let (width, height) = (2, 3);
        let mut layer = Layer {
            width,
            height,
            ..Default::default()
        };

        // fill with walls
        for _i in 0..(width * height) {
            layer.data.push(Some(Tile {
                id: WALL_01_TILE_ID,
                tileset: "".into(),
                attrs: "".into(),
            }));
        }

        // create mapgen
        let mapgen = MapGenerator::new(uvec2(width, height));

        // check that the rewrite does not trigger on the base case
        let did_rewrite = mapgen.try_rewrite_double_corner_vertical(0, 0, &mut layer);
        assert_eq!(did_rewrite, false);
        for i in 0..(width * height) {
            if let &Some(tile) = &layer.data[i as usize].as_ref() {
                assert_eq!(tile.id, WALL_01_TILE_ID);
                continue;
            }
            panic!("None tile found");
        }

        // create double corner pattern (variant 1)
        let i = xytoi(0, 0, &layer);
        layer.data[i] = Some(Tile {
            id: GROUND_01_TILE_ID,
            tileset: "".into(),
            attrs: "".into(),
        });
        let i = xytoi(1, 2, &layer);
        layer.data[i] = Some(Tile {
            id: GROUND_01_TILE_ID,
            tileset: "".into(),
            attrs: "".into(),
        });

        // check that double corner variant 1 rewrites correctly
        let did_rewrite = mapgen.try_rewrite_double_corner_vertical(0, 0, &mut layer);
        assert_eq!(did_rewrite, true);
        for i in 0..(width * height) {
            if let &Some(tile) = &layer.data[i as usize].as_ref() {
                assert_eq!(tile.id, WALL_01_TILE_ID);
                continue;
            }
            panic!("None tile found");
        }

        // create double corner pattern (variant 2)
        let i = xytoi(1, 0, &layer);
        layer.data[i] = Some(Tile {
            id: GROUND_01_TILE_ID,
            tileset: "".into(),
            attrs: "".into(),
        });
        let i = xytoi(0, 2, &layer);
        layer.data[i] = Some(Tile {
            id: GROUND_01_TILE_ID,
            tileset: "".into(),
            attrs: "".into(),
        });

        // check that double corner variant 2 rewrites correctly
        let did_rewrite = mapgen.try_rewrite_double_corner_vertical(0, 0, &mut layer);
        assert_eq!(did_rewrite, true);
        for i in 0..(width * height) {
            if let &Some(tile) = &layer.data[i as usize].as_ref() {
                assert_eq!(tile.id, WALL_01_TILE_ID);
                continue;
            }
            panic!("None tile found");
        }
    }
}
