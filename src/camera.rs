use macroquad::{
    camera::Camera2D,
    color::WHITE,
    math::{vec2, Vec2},
    texture::{draw_texture_ex, render_target, DrawTextureParams, FilterMode},
    window::{screen_height, screen_width},
};

use crate::constants::{SIMULATED_RESOLUTION, SIMULATED_TILE_PX};

pub struct Cameras {
    /// Worldspace camera (tile units, render_target)
    pub world_camera: Camera2D,

    /// Worldspace camera (simulated pixel, render_target)
    pub ui_camera: Camera2D,

    /// Screenspace camera (screen pixel units)
    pub screen_camera: Camera2D,
}

impl Cameras {
    pub fn new() -> Self {
        Self {
            world_camera: create_world_camera(),
            ui_camera: create_ui_camera(),
            screen_camera: create_screen_camera(),
        }
    }

    pub fn update(&mut self, player_pos: Vec2) {
        // update world camera to follow player
        self.world_camera.target = player_pos;

        // update screen camera to compensate for resolution changes.
        // creating a new one is cheap so we just do that
        self.screen_camera = create_screen_camera();
    }

    pub fn draw_world_render_to_screen(&self) {
        draw_texture_ex(
            &self
                .world_camera
                .render_target
                .as_ref()
                .expect("world camera missing render target")
                .texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(1., 1.)),
                ..Default::default()
            },
        )
    }

    pub fn draw_ui_render_to_screen(&self) {
        draw_texture_ex(
            &self
                .ui_camera
                .render_target
                .as_ref()
                .expect("world camera missing render target")
                .texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(1., 1.)),
                ..Default::default()
            },
        )
    }
}

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

/// Create a UI camera, zoomed to simulated resolution
pub fn create_ui_camera() -> Camera2D {
    let render_target = render_target(SIMULATED_RESOLUTION.x, SIMULATED_RESOLUTION.y);
    render_target.texture.set_filter(FilterMode::Nearest);
    let width = SIMULATED_RESOLUTION.x as f32;
    let height = SIMULATED_RESOLUTION.y as f32;
    Camera2D {
        target: vec2(width / 2., height / 2.),
        zoom: vec2(2. / width, 2. / height),
        render_target: Some(render_target),
        ..Default::default()
    }
}

/// Create a screen camera, which scales up and letterboxes the world camera.
pub fn create_screen_camera() -> Camera2D {
    let world_aspect = SIMULATED_RESOLUTION.x as f32 / SIMULATED_RESOLUTION.y as f32;
    let screen_aspect = screen_width() / screen_height();

    // a [0.5 0.5] target with [2. 2.] zoom renders the rect [0. 0.][1. 1.]
    let target = vec2(0.5, 0.5);
    let mut zoom = vec2(2., 2.);
    if screen_aspect > world_aspect {
        // screen is wider than world aspect. decrease x zoom to compensate
        zoom.x *= world_aspect / screen_aspect;
    } else {
        // screen is taller than world aspect. decrease y zoom to compensate
        zoom.y *= screen_aspect / world_aspect;
    }

    Camera2D {
        target,
        zoom,
        ..Default::default()
    }
}
