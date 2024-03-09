use std::ops::Range;

use macroquad::math::{uvec2, vec2, UVec2, Vec2};
use rapier2d::dynamics::CoefficientCombineRule;

use crate::map::MapGenerator;

/// Resolution of the simulated screen
// pub const SIMULATED_RESOLUTION: UVec2 = UVec2::new(320, 240);
pub const SIMULATED_RESOLUTION: UVec2 = UVec2::new(640, 480);

pub const SIMULATED_TILE_PX: f32 = 16.;

/// load path for the tile map texture
pub const TILESET_TEXTURE_PATH: &str = "assets/kenney_tiny-dungeon/Tilemap/tilemap_packed.png";

/// path that the map uses to find its tileset texture
pub const TILESET_MAP_PATH: &str = "../../kenney_tiny-dungeon/Tilemap/tilemap_packed.png";

/// ID that the map uses to refer to its tileset
pub const TILESET_MAP_ID: &str = "tiny_dungeon";

/// ID that the map uses to refer to its tileset
pub const TERRAIN_MAP_ID: &str = "terrain";
// pub const TERRAIN_MAP_ID: &str = "generated";

/// load path for the tile map data
// pub const TILE_MAP_JSON_PATH: &str = "assets/tiled/export/sandbox01.tmj";
pub const TILE_MAP_JSON_PATH: &str = "assets/tiled/export/sandbox03.tmj";

// TODO(axelmagn): fill this out
/// Tile ID ranges which should be treated as solid
pub const SOLID_TILES: [Range<u32>; 3] = [
    Range { start: 0, end: 6 },
    Range { start: 12, end: 18 },
    Range { start: 24, end: 28 },
];

pub const MIN_ROOM_SIZE: UVec2 = uvec2(10, 10);
pub const MAX_ROOM_SIZE: UVec2 = uvec2(20, 20);
pub const MAX_ROOM_COUNT: u32 = 20;
pub const CORRIDOR_PADDING: Option<u32> = Some(2);

pub const WALL_01_TILE_ID: u32 = 0;
pub const WALL_UP_TILE_ID: u32 = 2;
pub const WALL_DOWN_TILE_ID: u32 = 26;
pub const WALL_LEFT_TILE_ID: u32 = 13;
pub const WALL_RIGHT_TILE_ID: u32 = 15;
pub const WALL_INNER_UL_ID: u32 = 1;
pub const WALL_INNER_UR_ID: u32 = 3;
pub const WALL_INNER_DL_ID: u32 = 25;
pub const WALL_INNER_DR_ID: u32 = 27;
pub const WALL_OUTER_UL_ID: u32 = 4;
pub const WALL_OUTER_UR_ID: u32 = 5;
pub const WALL_OUTER_DL_ID: u32 = 16;
pub const WALL_OUTER_DR_ID: u32 = 17;
pub const WALL_TILE_IDS: &[u32] = &[
    WALL_01_TILE_ID,
    WALL_UP_TILE_ID,
    WALL_DOWN_TILE_ID,
    WALL_LEFT_TILE_ID,
    WALL_RIGHT_TILE_ID,
    WALL_INNER_UL_ID,
    WALL_INNER_UR_ID,
    WALL_INNER_DL_ID,
    WALL_INNER_DR_ID,
    WALL_OUTER_UL_ID,
    WALL_OUTER_UR_ID,
    WALL_OUTER_DL_ID,
    WALL_OUTER_DR_ID,
];

pub const FACADE_CENTER_TILE_ID: u32 = 40;
pub const FACADE_LEFT_TILE_ID: u32 = 57;
pub const FACADE_RIGHT_TILE_ID: u32 = 59;

pub const GROUND_01_TILE_ID: u32 = 48;

pub const PLAYER_ACCELERATION: f32 = 48.;
pub const PLAYER_BRAKING: f32 = 10.;
pub const PLAYER_FRICTION: f32 = 0.;
pub const PLAYER_FRICTION_COMBINE_RULE: CoefficientCombineRule = CoefficientCombineRule::Min;
pub const PLAYER_LINEAR_DAMPING: f32 = 2.;
pub const PLAYER_MASS: f32 = 100.;
pub const PLAYER_RADIUS: f32 = 0.5;
pub const PLAYER_RESTITUTION: f32 = 0.5;
pub const PLAYER_SPRITE_ID: u32 = 85;

pub const GUARD_ACCELERATION: f32 = 32.;
pub const GUARD_BRAKING: f32 = 10.;
pub const GUARD_FRICTION: f32 = 0.;
pub const GUARD_FRICTION_COMBINE_RULE: CoefficientCombineRule = CoefficientCombineRule::Min;
pub const GUARD_LINEAR_DAMPING: f32 = 2.5;
pub const GUARD_MASS: f32 = 100.;
pub const GUARD_RADIUS: f32 = 0.5;
pub const GUARD_RESTITUTION: f32 = 0.5;
pub const GUARD_SPRITE_ID: u32 = 96;
