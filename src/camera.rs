use macroquad::{
    camera::Camera2D,
    math::vec2,
    texture::{render_target, FilterMode},
};

use crate::constants::{SIMULATED_RESOLUTION, SIMULATED_TILE_PX};

/// Create a world camera, zoomed to a world space where 1 unit = 1 tile.
pub fn create_world_camera() -> Camera2D {
    let render_target = render_target(SIMULATED_RESOLUTION.x, SIMULATED_RESOLUTION.y);
    render_target.texture.set_filter(FilterMode::Nearest);
    let width = SIMULATED_RESOLUTION.x as f32 / SIMULATED_TILE_PX;
    let height = SIMULATED_RESOLUTION.y as f32 / SIMULATED_TILE_PX;
    Camera2D {
        target: vec2(width / 2., height / 2.),
        zoom: vec2(2. / width, 2. / height),
        render_target: Some(render_target),
        ..Default::default()
    }
}

pub fn create_screen_camera() -> Camera2D {
    Camera2D {
        target: vec2(0.5, 0.5),
        zoom: vec2(1., 1.),
        ..Default::default()
    }
}
