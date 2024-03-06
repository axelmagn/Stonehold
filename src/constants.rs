use std::ops::Range;

use macroquad::math::{vec2, UVec2, Vec2};
use rapier2d::dynamics::CoefficientCombineRule;

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

/// load path for the tile map data
pub const TILE_MAP_JSON_PATH: &str = "assets/tiled/export/sandbox01.tmj";

// TODO(axelmagn): fill this out
/// Tile ID ranges which should be treated as solid
pub const SOLID_TILES: [Range<u32>; 1] = [Range { start: 0, end: 8 }];

pub const PLAYER_ACCELERATION: f32 = 48.;
pub const PLAYER_BRAKING: f32 = 10.;
pub const PLAYER_FRICTION: f32 = 0.;
pub const PLAYER_FRICTION_COMBINE_RULE: CoefficientCombineRule = CoefficientCombineRule::Min;
pub const PLAYER_LINEAR_DAMPING: f32 = 2.;
pub const PLAYER_MASS: f32 = 100.;
pub const PLAYER_RADIUS: f32 = 0.5;
pub const PLAYER_RESTITUTION: f32 = 0.5;
pub const PLAYER_SPRITE_ID: u32 = 85;
pub const PLAYER_START_POSITION: Vec2 = vec2(20., 20.);

pub const GUARD_ACCELERATION: f32 = 32.;
pub const GUARD_BRAKING: f32 = 10.;
pub const GUARD_FRICTION: f32 = 0.;
pub const GUARD_FRICTION_COMBINE_RULE: CoefficientCombineRule = CoefficientCombineRule::Min;
pub const GUARD_LINEAR_DAMPING: f32 = 2.5;
pub const GUARD_MASS: f32 = 100.;
pub const GUARD_RADIUS: f32 = 0.5;
pub const GUARD_RESTITUTION: f32 = 0.5;
pub const GUARD_SPRITE_ID: u32 = 96;
pub const GUARD_START_POSITIONS: [Vec2; 1] = [vec2(23., 23.)];
