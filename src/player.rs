use std::f32::EPSILON;

use macroquad::{
    color::WHITE,
    input::{is_key_down, KeyCode},
    math::{vec2, Rect, Vec2},
    text::draw_text,
};
use macroquad_tiled::Map as TiledMap;
use nalgebra::vector;
use rapier2d::{
    dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodySet},
    geometry::{ColliderBuilder, ColliderHandle, ColliderSet},
};

use crate::{
    constants::{
        PLAYER_ACCELERATION, PLAYER_BRAKING, PLAYER_FRICTION, PLAYER_FRICTION_COMBINE_RULE,
        PLAYER_LINEAR_DAMPING, PLAYER_MASS, PLAYER_RADIUS, PLAYER_RESTITUTION, PLAYER_SPRITE_ID,
        TILESET_MAP_ID,
    },
    physics::Physics,
};

pub enum FacingDirection {
    Left,
    Right,
}

pub struct Player {
    pub input_direction: Vec2,
    pub position: Vec2,
    pub facing_direction: FacingDirection,
    pub sprite_id: u32,
    pub collider: Option<ColliderHandle>,
    pub body: Option<RigidBodyHandle>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            input_direction: Vec2::ZERO,
            position: vec2(19., 19.),
            facing_direction: FacingDirection::Left,
            sprite_id: PLAYER_SPRITE_ID,
            collider: None,
            body: None,
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

    pub fn update(&mut self, physics: &mut Physics) {
        // propagate input to physics object
        if let Some(body_handle) = self.body {
            let body = &mut physics.bodies[body_handle];

            let move_force = self.input_direction * PLAYER_ACCELERATION * body.mass();
            let move_force = vector![move_force.x, move_force.y];

            let vel_dir = vec2(body.linvel().x, body.linvel().y).normalize_or_zero();
            let braking_force =
                (self.input_direction - vel_dir).normalize_or_zero() * PLAYER_BRAKING * body.mass();
            let braking_force = vector![braking_force.x, braking_force.y];

            body.reset_forces(true);
            body.add_force(move_force.into(), true);
            body.add_force(braking_force.into(), true);
        }

        // latch facing direction on nonzero input direction
        if self.input_direction.x > 0. {
            self.facing_direction = FacingDirection::Left;
        } else if self.input_direction.x < 0. {
            self.facing_direction = FacingDirection::Right;
        }
    }

    pub fn post_physics(&mut self, physics: &mut Physics) {
        // temporary until we get ECS working
        if let Some(body_handle) = self.body {
            let body = &physics.bodies[body_handle];
            // mq -> nalgebra conversion
            self.position.x = body.translation().x - 0.5;
            self.position.y = body.translation().y - 0.5;
        }
    }

    pub fn draw(&self, tile_map: &TiledMap) {
        tile_map.spr(TILESET_MAP_ID, self.sprite_id, self.get_draw_rect());
    }

    pub fn draw_ui(&self, physics: &Physics) {
        // draw debug physics variables
        if let Some(body_handle) = self.body {
            let body = &physics.bodies[body_handle];

            draw_text(
                &format!(
                    "POS:   ({:8.2}, {:8.2})",
                    body.translation().x,
                    body.translation().y
                ),
                10.,
                40.,
                20.,
                WHITE,
            );

            draw_text(
                &format!("VEL:   ({:8.2}, {:8.2})", body.linvel().x, body.linvel().y),
                10.,
                60.,
                20.,
                WHITE,
            );

            draw_text(
                &format!(
                    "INPUT: ({:8.2}, {:8.2})",
                    self.input_direction.x, self.input_direction.y
                ),
                10.,
                80.,
                20.,
                WHITE,
            );

            draw_text(
                &format!(
                    "FORCE: ({:8.2}, {:8.2})",
                    body.user_force().x,
                    body.user_force().y,
                ),
                10.,
                100.,
                20.,
                WHITE,
            );
        }
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

    pub fn init_physics(
        &mut self,
        start_pos: Vec2,
        collider_set: &mut ColliderSet,
        rigid_body_set: &mut RigidBodySet,
    ) {
        let body = RigidBodyBuilder::dynamic()
            .translation(vector![start_pos.x + 0.5, start_pos.y + 0.5])
            .lock_rotations()
            .linear_damping(PLAYER_LINEAR_DAMPING) // TODO: make const
            .ccd_enabled(true)
            .can_sleep(false)
            .build();
        let collider = ColliderBuilder::ball(PLAYER_RADIUS)
            .mass(PLAYER_MASS)
            .friction(PLAYER_FRICTION)
            .friction_combine_rule(PLAYER_FRICTION_COMBINE_RULE)
            .restitution(PLAYER_RESTITUTION)
            .build();

        let body_handle = rigid_body_set.insert(body);

        self.body = Some(body_handle);
        self.collider =
            Some(collider_set.insert_with_parent(collider, body_handle, rigid_body_set));
    }
}
