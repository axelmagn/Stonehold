use macroquad::{
    input::{is_key_down, KeyCode},
    math::{vec2, Rect, Vec2},
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
pub struct Character {
    pub position: Vec2,
    input_direction: Vec2,
    facing_direction: FacingDirection,
    sprite_id: u32,
    acceleration: f32,
    braking: f32,
    collider_handle: ColliderHandle,
    body_handle: RigidBodyHandle,
}

impl Character {
    pub fn create<T: CharacterConfigProvider>(
        position: Vec2,
        collider_set: &mut ColliderSet,
        rigid_body_set: &mut RigidBodySet,
    ) -> Self {
        let (collider_handle, body_handle) =
            T::init_physics(position, collider_set, rigid_body_set);
        Self {
            position,
            input_direction: Vec2::ZERO,
            facing_direction: FacingDirection::Left,
            sprite_id: T::get_sprite_id(),
            acceleration: T::get_acceleration(),
            braking: T::get_braking(),
            collider_handle,
            body_handle,
        }
    }

    pub fn create_player(
        position: Vec2,
        collider_set: &mut ColliderSet,
        rigid_body_set: &mut RigidBodySet,
    ) -> Self {
        Self::create::<PlayerConfigProvider>(position, collider_set, rigid_body_set)
    }

    pub fn collect_player_inputs(&mut self) {
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
        let body = &mut physics.bodies[self.body_handle];

        let move_force = self.input_direction * self.acceleration * body.mass();
        let move_force = vector![move_force.x, move_force.y];

        let vel_dir = vec2(body.linvel().x, body.linvel().y).normalize_or_zero();
        let braking_force = (self.input_direction - vel_dir)
            * body.linvel().magnitude()
            * self.braking
            * body.mass();
        let braking_force = vector![braking_force.x, braking_force.y];

        body.reset_forces(true);
        body.add_force(move_force, true);
        body.add_force(braking_force, true);

        // latch facing direction on nonzero input direction
        if self.input_direction.x > 0. {
            self.facing_direction = FacingDirection::Left;
        } else if self.input_direction.x < 0. {
            self.facing_direction = FacingDirection::Right;
        }
    }

    pub fn post_physics(&mut self, physics: &mut Physics) {
        let body = &physics.bodies[self.body_handle];
        // mq -> nalgebra conversion
        self.position.x = body.translation().x - 0.5;
        self.position.y = body.translation().y - 0.5;
    }

    pub fn draw(&self, tile_map: &TiledMap) {
        tile_map.spr(TILESET_MAP_ID, self.sprite_id, self.get_draw_rect());
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
}

pub trait CharacterConfigProvider {
    fn get_sprite_id() -> u32;
    fn get_acceleration() -> f32;
    fn get_braking() -> f32;

    fn init_physics(
        position: Vec2,
        collider_set: &mut ColliderSet,
        rigid_body_set: &mut RigidBodySet,
    ) -> (ColliderHandle, RigidBodyHandle);
}

struct PlayerConfigProvider;
impl CharacterConfigProvider for PlayerConfigProvider {
    fn get_sprite_id() -> u32 {
        PLAYER_SPRITE_ID
    }

    fn init_physics(
        position: Vec2,
        collider_set: &mut ColliderSet,
        rigid_body_set: &mut RigidBodySet,
    ) -> (ColliderHandle, RigidBodyHandle) {
        let body = RigidBodyBuilder::dynamic()
            .translation(vector![position.x + 0.5, position.y + 0.5])
            .lock_rotations()
            .linear_damping(PLAYER_LINEAR_DAMPING) // TODO: make const
            .ccd_enabled(false)
            .can_sleep(true)
            .build();
        let collider = ColliderBuilder::ball(PLAYER_RADIUS)
            .mass(PLAYER_MASS)
            .friction(PLAYER_FRICTION)
            .friction_combine_rule(PLAYER_FRICTION_COMBINE_RULE)
            .restitution(PLAYER_RESTITUTION)
            .build();

        let body_handle = rigid_body_set.insert(body);
        let collider_handle =
            collider_set.insert_with_parent(collider, body_handle, rigid_body_set);
        (collider_handle, body_handle)
    }

    fn get_acceleration() -> f32 {
        PLAYER_ACCELERATION
    }

    fn get_braking() -> f32 {
        PLAYER_BRAKING
    }
}
