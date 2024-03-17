use macroquad::math::UVec2;
use macroquad_tiled::{Layer, Tile};
use nalgebra::vector;
use rapier2d::geometry::{ColliderBuilder, ColliderHandle, ColliderSet};

use crate::{
    constants::{DOOR_LEFT_CLOSED_TILE_ID, DOOR_RIGHT_CLOSED_TILE_ID, TILESET_MAP_ID},
    map::mapgen::xytoi,
};

pub struct GuardDoor {
    position: UVec2,
    pub is_open: bool,
    pub collider_handle: ColliderHandle,
}

impl GuardDoor {
    pub fn create(position: UVec2, collider_set: &mut ColliderSet) -> Self {
        // set up collider
        let collider = ColliderBuilder::cuboid(1.0, 0.5)
            .translation(vector![position.x as f32 + 1.0, position.y as f32 + 0.5])
            .sensor(true)
            .build();
        let collider_handle = collider_set.insert(collider);

        Self {
            position,
            is_open: true,
            collider_handle,
        }
    }

    pub fn close_door(&mut self, layer: &mut Layer) {
        self.is_open = false;
        let i = xytoi(self.position.x, self.position.y, layer);
        layer.data[i + 1] = Some(Tile {
            id: DOOR_LEFT_CLOSED_TILE_ID,
            tileset: TILESET_MAP_ID.into(),
            attrs: "".into(),
        });
        layer.data[i + 2] = Some(Tile {
            id: DOOR_RIGHT_CLOSED_TILE_ID,
            tileset: TILESET_MAP_ID.into(),
            attrs: "".into(),
        });
    }
}
