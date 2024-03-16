use macroquad::{
    input::{is_key_down, KeyCode},
    logging::info,
    math::{vec2, Rect, Vec2},
    time::{get_frame_time, get_time},
};
use macroquad_tiled::Map as TiledMap;
use nalgebra::vector;
use rapier2d::{
    dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodySet},
    geometry::{ColliderBuilder, ColliderHandle, ColliderSet},
    pipeline::ActiveEvents,
};

use crate::{
    constants::{
        ALERTED_INDICATOR_COOLDOWN, DAMAGE_COOLDOWN, GRAVE_TILE_ID, GUARD_ACCELERATION,
        GUARD_ALERT_DISTANCE, GUARD_BRAKING, GUARD_FRICTION, GUARD_FRICTION_COMBINE_RULE,
        GUARD_LINEAR_DAMPING, GUARD_MASS, GUARD_MAX_HEALTH, GUARD_RADIUS, GUARD_RESTITUTION,
        GUARD_SPRITE_ID, HEART_TILE_ID, KNOCKBACK_COOLDOWN, PLAYER_ACCELERATION, PLAYER_BRAKING,
        PLAYER_FRICTION, PLAYER_FRICTION_COMBINE_RULE, PLAYER_GUARD_KNOCKBACK,
        PLAYER_LINEAR_DAMPING, PLAYER_MASS, PLAYER_MAX_HEALTH, PLAYER_RADIUS, PLAYER_RESTITUTION,
        PLAYER_SPRITE_ID, QUESTION_MARK_TILE_ID, SIMULATED_TILE_PX, TILESET_MAP_ID,
    },
    physics::Physics,
};

#[derive(Debug)]
pub enum FacingDirection {
    Left,
    Right,
}

#[derive(Debug)]
pub struct Character {
    pub position: Vec2,
    input_direction: Vec2,
    facing_direction: FacingDirection,
    sprite_id: u32,
    acceleration: f32,
    braking: f32,
    pub collider_handle: ColliderHandle,
    body_handle: RigidBodyHandle,
    health: u32,
    _max_health: u32,
    accumulated_knockback: Vec2,
    is_alerted: bool,
    last_damage_time: f64,
    last_knockback_time: f64,
    last_alerted: f64,
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
            health: T::get_max_health(),
            _max_health: T::get_max_health(),
            accumulated_knockback: Vec2::ZERO,
            is_alerted: false,
            last_damage_time: 0.,
            last_knockback_time: 0.,
            last_alerted: 0.,
        }
    }

    pub fn create_player(
        position: Vec2,
        collider_set: &mut ColliderSet,
        rigid_body_set: &mut RigidBodySet,
    ) -> Self {
        Self::create::<PlayerConfigProvider>(position, collider_set, rigid_body_set)
    }

    pub fn create_guard(
        position: Vec2,
        collider_set: &mut ColliderSet,
        rigid_body_set: &mut RigidBodySet,
    ) -> Self {
        Self::create::<GuardConfigProvider>(position, collider_set, rigid_body_set)
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

    pub fn collect_guard_inputs(&mut self, player: &Character) {
        if !self.is_alerted || !player.is_alive() {
            self.input_direction = Vec2::ZERO;
            return;
        }

        self.input_direction = (player.position - self.position).normalize_or_zero();
    }

    pub fn update(&mut self, physics: &mut Physics) {
        if !self.is_alive() {
            return;
        }

        // propagate input to physics object
        let body = &mut physics.bodies[self.body_handle];

        let move_acc = self.input_direction * self.acceleration;
        let move_acc = vector![move_acc.x, move_acc.y];

        let vel_dir = vec2(body.linvel().x, body.linvel().y).normalize_or_zero();
        let braking_acc =
            (self.input_direction - vel_dir) * body.linvel().magnitude() * self.braking;
        let braking_acc = vector![braking_acc.x, braking_acc.y];

        let knockback = vector![self.accumulated_knockback.x, self.accumulated_knockback.y];
        self.accumulated_knockback = Vec2::ZERO;

        // body.reset_forces(true);
        // body.add_force(move_force, true);
        // body.add_force(braking_force, true);
        let dt = get_frame_time();
        let new_linvel = body.linvel() + move_acc * dt + braking_acc * dt + knockback;
        body.set_linvel(new_linvel, true);

        // latch facing direction on nonzero input direction
        if self.input_direction.x > 0. {
            self.facing_direction = FacingDirection::Left;
        } else if self.input_direction.x < 0. {
            self.facing_direction = FacingDirection::Right;
        }
    }

    pub fn post_physics(&mut self, physics: &mut Physics) {
        let body = &physics.bodies[self.body_handle];
        // TODO(axelmagn): snap to simulated pixel
        // mq -> nalgebra conversion
        self.position.x = body.translation().x - 0.5;
        self.position.y = body.translation().y - 0.5;
    }

    pub fn draw(&self, tile_map: &TiledMap) {
        let mut draw_rect = self.get_draw_rect();
        let sprite_id = if self.is_alive() {
            self.sprite_id
        } else {
            GRAVE_TILE_ID
        };
        tile_map.spr(TILESET_MAP_ID, sprite_id, draw_rect);
        if self.is_alerted && get_time() < self.last_alerted + ALERTED_INDICATOR_COOLDOWN {
            draw_rect.y -= 1.;
            tile_map.spr(TILESET_MAP_ID, QUESTION_MARK_TILE_ID, draw_rect);
        }
    }

    pub fn draw_ui(&self, tile_map: &TiledMap) {
        let origin = vec2(10., 10.);
        for i in 0..self.health {
            let padding = -1.;
            let offset_x = (SIMULATED_TILE_PX * 2. + padding) * i as f32;
            let draw_rect = Rect::new(
                origin.x + offset_x,
                origin.y,
                SIMULATED_TILE_PX * 2.,
                SIMULATED_TILE_PX * 2.,
            );
            tile_map.spr(TILESET_MAP_ID, HEART_TILE_ID, draw_rect);
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

    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn can_damage(&self) -> bool {
        get_time() > self.last_damage_time + DAMAGE_COOLDOWN
    }

    pub fn can_knockback(&self) -> bool {
        get_time() > self.last_knockback_time + KNOCKBACK_COOLDOWN
    }

    pub fn handle_player_guard_collision(&mut self, guard: &Character) {
        info!("PLAYER HIT");
        self.deal_damage(1);

        let guard_dir = (guard.position - self.position).normalize_or_zero();
        let knockback = -1. * guard_dir * PLAYER_GUARD_KNOCKBACK;
        self.apply_knockback(knockback);
    }

    pub fn deal_damage(&mut self, amount: u32) {
        if !self.can_damage() {
            return;
        }
        self.health -= amount.min(self.health);
        self.last_damage_time = get_time();
    }

    pub fn apply_knockback(&mut self, delta_velocity: Vec2) {
        if !self.can_knockback() {
            return;
        }

        self.accumulated_knockback += delta_velocity;
        self.last_knockback_time = get_time();
    }

    pub fn check_guard_distance(&mut self, player: &Character) {
        if self.position.distance_squared(player.position)
            < GUARD_ALERT_DISTANCE * GUARD_ALERT_DISTANCE
        {
            self.alert_guard();
        }
    }

    pub fn alert_guard(&mut self) {
        if self.is_alerted {
            return;
        }
        self.is_alerted = true;
        self.last_alerted = get_time();
    }
}

pub trait CharacterConfigProvider {
    fn get_sprite_id() -> u32;
    fn get_acceleration() -> f32;
    fn get_braking() -> f32;
    fn get_max_health() -> u32;

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
        // character body
        let body = RigidBodyBuilder::dynamic()
            .translation(vector![position.x + 0.5, position.y + 0.5])
            .lock_rotations()
            .linear_damping(PLAYER_LINEAR_DAMPING) // TODO: make const
            .ccd_enabled(true)
            .build();

        let collider = ColliderBuilder::ball(PLAYER_RADIUS)
            .mass(PLAYER_MASS)
            .friction(PLAYER_FRICTION)
            .friction_combine_rule(PLAYER_FRICTION_COMBINE_RULE)
            .restitution(PLAYER_RESTITUTION)
            .active_events(ActiveEvents::COLLISION_EVENTS | ActiveEvents::CONTACT_FORCE_EVENTS)
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

    fn get_max_health() -> u32 {
        PLAYER_MAX_HEALTH
    }
}

struct GuardConfigProvider;
impl CharacterConfigProvider for GuardConfigProvider {
    fn get_sprite_id() -> u32 {
        GUARD_SPRITE_ID
    }

    fn get_acceleration() -> f32 {
        GUARD_ACCELERATION
    }

    fn get_braking() -> f32 {
        GUARD_BRAKING
    }

    fn init_physics(
        position: Vec2,
        collider_set: &mut ColliderSet,
        rigid_body_set: &mut RigidBodySet,
    ) -> (ColliderHandle, RigidBodyHandle) {
        let body = RigidBodyBuilder::dynamic()
            .translation(vector![position.x + 0.5, position.y + 0.5])
            .lock_rotations()
            .linear_damping(GUARD_LINEAR_DAMPING) // TODO: make const
            .ccd_enabled(true)
            .build();
        let collider = ColliderBuilder::ball(GUARD_RADIUS)
            .mass(GUARD_MASS)
            .friction(GUARD_FRICTION)
            .friction_combine_rule(GUARD_FRICTION_COMBINE_RULE)
            .restitution(GUARD_RESTITUTION)
            .active_events(ActiveEvents::COLLISION_EVENTS | ActiveEvents::CONTACT_FORCE_EVENTS)
            .build();

        let body_handle = rigid_body_set.insert(body);
        let collider_handle =
            collider_set.insert_with_parent(collider, body_handle, rigid_body_set);
        (collider_handle, body_handle)
    }

    fn get_max_health() -> u32 {
        GUARD_MAX_HEALTH
    }
}
