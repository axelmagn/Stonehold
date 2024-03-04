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

/// Sprite ID for the player
pub const PLAYER_SPRITE_ID: u32 = 85;

// TODO(axelmagn): fill this out
/// Tile ID ranges which should be treated as solid
pub const SOLID_TILES: [Range<u32>; 1] = [Range { start: 0, end: 8 }];

// temporary
pub const PLAYER_START_POS: Vec2 = vec2(20., 20.);

pub const PLAYER_RADIUS: f32 = 0.5;
pub const PLAYER_LINEAR_DAMPING: f32 = 2.;
pub const PLAYER_BRAKING: f32 = 10.;
pub const PLAYER_ACCELERATION: f32 = 48.;
pub const PLAYER_FRICTION: f32 = 0.;
pub const PLAYER_FRICTION_COMBINE_RULE: CoefficientCombineRule = CoefficientCombineRule::Min;
pub const PLAYER_RESTITUTION: f32 = 0.5;
pub const PLAYER_MASS: f32 = 100.;
