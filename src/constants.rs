use macroquad::math::UVec2;

/// Resolution of the simulated screen
pub const SIMULATED_RESOLUTION: UVec2 = UVec2::new(320, 240);

/// load path for the tile map texture
pub const TILESET_TEXTURE_PATH: &'static str =
    "assets/kenney_tiny-dungeon/Tilemap/tilemap_packed.png";

/// key that the tile map refers to its texture with (set by Tiled)
pub const TILESET_MAP_KEY: &'static str = "../../kenney_tiny-dungeon/Tilemap/tilemap_packed.png";

/// load path for the tile map data
pub const TILE_MAP_JSON_PATH: &'static str = "assets/tiled/export/sandbox01.tmj";
