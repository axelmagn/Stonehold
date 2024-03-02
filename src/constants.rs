use macroquad::math::UVec2;

/// Resolution of the simulated screen
pub const SIMULATED_RESOLUTION: UVec2 = UVec2::new(320, 240);

/// load path for the tile map texture
pub const TILESET_TEXTURE_PATH: &'static str =
    "assets/kenney_tiny-dungeon/Tilemap/tilemap_packed.png";

/// path that the map uses to find its tileset texture
pub const TILESET_MAP_PATH: &'static str = "../../kenney_tiny-dungeon/Tilemap/tilemap_packed.png";

/// ID that the map uses to refer to its tileset
pub const TILESET_MAP_ID: &'static str = "tiny_dungeon";

/// load path for the tile map data
pub const TILE_MAP_JSON_PATH: &'static str = "assets/tiled/export/sandbox01.tmj";

pub const TILE_SIZE: f32 = 16.;

pub const PLAYER_SPRITE_ID: u32 = 85;
