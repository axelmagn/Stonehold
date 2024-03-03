use macroquad::{
    input::{is_key_down, KeyCode},
    math::{vec2, Rect, Vec2},
    time::get_frame_time,
};
use macroquad_tiled::Map as TiledMap;

use crate::constants::{PLAYER_SPRITE_ID, TILESET_MAP_ID};

pub enum FacingDirection {
    Left,
    Right,
}

pub struct Player {
    pub input_direction: Vec2,
    pub position: Vec2,
    pub facing_direction: FacingDirection,
    pub sprite_id: u32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            input_direction: Vec2::ZERO,
            position: vec2(19., 19.),
            facing_direction: FacingDirection::Left,
            sprite_id: PLAYER_SPRITE_ID,
        }
    }

    pub fn collect_inputs(&mut self) {
        self.input_direction = Vec2::ZERO;
        if is_key_down(KeyCode::W) {
            self.input_direction += vec2(0., -1.);
        }
        if is_key_down(KeyCode::S) {
            self.input_direction += vec2(0., 1.);
        }
        if is_key_down(KeyCode::A) {
            self.input_direction += vec2(-1., 0.);
        }
        if is_key_down(KeyCode::D) {
            self.input_direction += vec2(1., 0.);
        }
        self.input_direction = self.input_direction.normalize_or_zero();
    }

    pub fn update(&mut self) {
        let dt = get_frame_time();
        // temporary until we get kinematics working
        self.position += self.input_direction * dt * 5.;

        // latch facing direction on nonzero input direction
        if self.input_direction.x > 0. {
            self.facing_direction = FacingDirection::Left;
        } else if self.input_direction.x < 0. {
            self.facing_direction = FacingDirection::Right;
        }
    }

    pub fn draw(&self, tile_map: &TiledMap) {
        tile_map.spr(TILESET_MAP_ID, self.sprite_id, self.get_draw_rect());
    }

    pub fn get_draw_rect(&self) -> Rect {
        match self.facing_direction {
            FacingDirection::Left => Rect {
                x: self.position.x,
                y: self.position.y,
                w: 1.,
                h: 1.,
            },
            FacingDirection::Right => Rect {
                x: self.position.x + 1.,
                y: self.position.y,
                w: -1.,
                h: 1.,
            },
        }
    }
}
