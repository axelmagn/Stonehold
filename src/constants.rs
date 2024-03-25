use std::ops::Range;

use macroquad::math::{uvec2, UVec2};
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
// pub const TERRAIN_MAP_ID: &str = "generated";

/// load path for the tile map data
// pub const TILE_MAP_JSON_PATH: &str = "assets/tiled/export/sandbox01.tmj";
pub const TILE_MAP_JSON_PATH: &str = "assets/tiled/export/sandbox03.tmj";

pub const CLICK_SOUND_PATH: &str = "assets/kenney_interface-sounds/Audio/click_004.ogg";
pub const ATTACK_SOUND_PATH: &str = "assets/kenney_impact-sounds/Audio/impactPunch_heavy_001.ogg";
pub const KNOCKBACK_SOUND_PATH: &str = "assets/kenney_impact-sounds/Audio/impactBell_heavy_002.ogg";
pub const ALERT_SOUND_PATH: &str =
    "assets/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI00.ogg";
pub const DOOR_CLOSE_SOUND_PATH: &str = "assets/kenney_rpg-audio/Audio/doorClose_1.ogg";
pub const VICTORY_SOUND_PATH: &str =
    "assets/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI10.ogg";
pub const DEFEAT_SOUND_PATH: &str =
    "assets/kenney_music-jingles/Audio/Pizzicato jingles/jingles_PIZZI07.ogg";

// TODO(axelmagn): fill this out
/// Tile ID ranges which should be treated as solid
pub const SOLID_TILES: &[Range<u32>] = &[
    Range { start: 0, end: 6 },
    Range { start: 12, end: 14 },
    Range { start: 15, end: 18 },
    Range { start: 19, end: 21 },
    Range { start: 24, end: 28 },
];

pub const MIN_ROOM_SIZE: UVec2 = uvec2(10, 10);
pub const MAX_ROOM_SIZE: UVec2 = uvec2(20, 20);
pub const MAX_ROOM_COUNT: u32 = 50;
pub const CORRIDOR_PADDING: Option<u32> = Some(2);
pub const DOOR_CLEARANCE: u32 = 8;
pub const TILE_FILLER_PROB: f32 = 0.003;

pub const WALL_01_TILE_ID: u32 = 0;
pub const WALL_02_TILE_ID: u32 = 12;
pub const WALL_03_TILE_ID: u32 = 24;

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
    WALL_02_TILE_ID,
    WALL_03_TILE_ID,
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
pub const FACADE_CENTER_02_TILE_ID: u32 = 14;

pub const GROUND_01_TILE_ID: u32 = 48;
pub const GROUND_02_TILE_ID: u32 = 49;
pub const GROUND_03_TILE_ID: u32 = 42;

pub const DOOR_LEFT_CLOSED_TILE_ID: u32 = 46;
pub const DOOR_RIGHT_CLOSED_TILE_ID: u32 = 47;
pub const DOOR_LEFT_OPEN_TILE_ID: u32 = 10;
pub const DOOR_RIGHT_OPEN_TILE_ID: u32 = 11;

pub const STAIRS_LEFT_TILE_ID: u32 = 36;
pub const _STAIRS_CENTER_TILE_ID: u32 = 37;
pub const STAIRS_RIGHT_TILE_ID: u32 = 38;

pub const MONSTER_PIPE_CLOSED_TILE_ID: u32 = 19;
pub const _MONSTER_PIPE_OPEN_TILE_ID: u32 = 20;

pub const POOL_EMPTY_TILE_ID: u32 = 31;
pub const _POOL_FULL_TILE_ID: u32 = 32;

pub const PLAYER_ACCELERATION: f32 = 48.;
pub const PLAYER_BRAKING: f32 = 10.;
pub const PLAYER_FRICTION: f32 = 0.;
pub const PLAYER_FRICTION_COMBINE_RULE: CoefficientCombineRule = CoefficientCombineRule::Min;
pub const PLAYER_LINEAR_DAMPING: f32 = 2.;
pub const PLAYER_MASS: f32 = 100.;
pub const PLAYER_RADIUS: f32 = 0.5;
pub const PLAYER_RESTITUTION: f32 = 0.5;
pub const PLAYER_SPRITE_ID: u32 = 85 + 27;
pub const PLAYER_MAX_HEALTH: u32 = 5;

pub const GUARD_ACCELERATION: f32 = 24.;
pub const GUARD_BRAKING: f32 = 10.;
pub const GUARD_FRICTION: f32 = 0.;
pub const GUARD_FRICTION_COMBINE_RULE: CoefficientCombineRule = CoefficientCombineRule::Min;
pub const GUARD_LINEAR_DAMPING: f32 = 2.5;
pub const GUARD_MASS: f32 = 100.;
pub const GUARD_RADIUS: f32 = 0.5;
pub const GUARD_RESTITUTION: f32 = 0.5;
pub const GUARD_SPRITE_ID: u32 = 96;
pub const GUARD_MAX_HEALTH: u32 = 3;

pub const QUESTION_MARK_TILE_ID: u32 = 127;
pub const HEART_TILE_ID: u32 = 128;
pub const GRAVE_TILE_ID: u32 = 64;

pub const DAMAGE_COOLDOWN: f64 = 1.;
pub const KNOCKBACK_COOLDOWN: f64 = 0.2;
pub const ALERTED_INDICATOR_COOLDOWN: f64 = 3.;
pub const ATTACK_COOLDOWN: f64 = 0.4;
pub const ATTACK_DURATION: f64 = 0.1;

pub const PLAYER_GUARD_KNOCKBACK: f32 = 96.;
pub const PLAYER_ATTACK_KNOCKBACK: f32 = 256.;
pub const GUARD_ALERT_DISTANCE: f32 = 10.;
pub const PLAYER_ATTACK_RADIUS: f32 = 1.6;

pub const DEATH_LINGER_TIME: f64 = 1.;
