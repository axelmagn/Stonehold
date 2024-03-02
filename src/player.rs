use macroquad::math::{Rect, Vec2};
use macroquad_tiled::Map;

use crate::constants::{PLAYER_SPRITE_ID, TILESET_MAP_ID, TILESET_MAP_PATH, TILE_SIZE};

pub struct Player {
    pub position: Vec2,
    pub sprite_id: u32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: Vec2::new(20. * 16., 21. * 16.),
            sprite_id: PLAYER_SPRITE_ID,
        }
    }

    pub fn draw(&self, tile_map: &Map) {
        tile_map.spr(TILESET_MAP_ID, self.sprite_id, self.get_draw_rect());
    }

    pub fn get_draw_rect(&self) -> Rect {
        Rect {
            x: self.position.x,
            y: self.position.y,
            w: TILE_SIZE,
            h: TILE_SIZE,
        }
    }
}
