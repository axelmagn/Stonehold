use macroquad::{
    audio::{play_sound_once},
    color::WHITE,
    input::{is_key_down, is_mouse_button_down, mouse_position_local, KeyCode, MouseButton},
    logging::info,
    math::{vec2, Rect, Vec2},
    shapes::draw_circle,
    time::{get_frame_time, get_time},
};
use macroquad_tiled::Map as TiledMap;
use nalgebra::{vector, Vector2};
use rapier2d::{
    dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodySet},
    geometry::{ColliderBuilder, ColliderHandle, ColliderSet},
    math::Isometry,
    pipeline::ActiveEvents,
};

use crate::{
    audio::Sounds,
    constants::{
        ALERTED_INDICATOR_COOLDOWN, ATTACK_COOLDOWN, ATTACK_DURATION, DAMAGE_COOLDOWN,
        GRAVE_TILE_ID, GUARD_ACCELERATION, GUARD_ALERT_DISTANCE, GUARD_BRAKING, GUARD_FRICTION,
        GUARD_FRICTION_COMBINE_RULE, GUARD_KNOCKBACK_COOLDOWN, GUARD_LINEAR_DAMPING, GUARD_MASS,
        GUARD_MAX_HEALTH, GUARD_RADIUS, GUARD_RESTITUTION, GUARD_SPRITE_ID, HEART_TILE_ID,
        KNOCKBACK_COOLDOWN, PLAYER_ACCELERATION, PLAYER_ATTACK_KNOCKBACK, PLAYER_ATTACK_RADIUS,
        PLAYER_BRAKING, PLAYER_FRICTION, PLAYER_FRICTION_COMBINE_RULE, PLAYER_GUARD_KNOCKBACK,
        PLAYER_KNOCKBACK_COOLDOWN, PLAYER_LINEAR_DAMPING, PLAYER_MASS, PLAYER_MAX_HEALTH,
        PLAYER_RADIUS, PLAYER_RESTITUTION, PLAYER_SPRITE_ID, QUESTION_MARK_TILE_ID,
        SIMULATED_TILE_PX, TILESET_MAP_ID,
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
    attack_position: Vec2,
    input_direction: Vec2,
    facing_direction: FacingDirection,
    sprite_id: u32,
    acceleration: f32,
    braking: f32,
    pub collider_handle: Option<ColliderHandle>,
    pub attack_collider_handle: Option<ColliderHandle>,
    body_handle: Option<RigidBodyHandle>,
    health: u32,
    _max_health: u32,
    accumulated_knockback: Vec2,
    is_alerted: bool,
    pub is_attacking: bool,
    attack_direction: Vec2,
    last_attack_start: f64,
    last_damage_time: f64,
    last_knockback_time: f64,
    last_alerted: f64,
    pub death_time: f64,
    pub draw_attack: bool,
    pub sounds: Sounds,
    pub knockback_cooldown: f64,
}

impl Character {
    pub fn create<T: CharacterConfigProvider>(
        position: Vec2,
        collider_set: &mut ColliderSet,
        rigid_body_set: &mut RigidBodySet,
        sounds: Sounds,
    ) -> Self {
        let (collider_handle, body_handle, attack_collider_handle) =
            T::init_physics(position, collider_set, rigid_body_set);
        Self {
            position,
            attack_position: position,
            input_direction: Vec2::ZERO,
            facing_direction: FacingDirection::Left,
            sprite_id: T::get_sprite_id(),
            acceleration: T::get_acceleration(),
            braking: T::get_braking(),
            collider_handle: Some(collider_handle),
            attack_collider_handle,
            body_handle: Some(body_handle),
            health: T::get_max_health(),
            _max_health: T::get_max_health(),
            accumulated_knockback: Vec2::ZERO,
            is_alerted: false,
            is_attacking: false,
            attack_direction: Vec2::ZERO,
            last_attack_start: 0.,
            last_damage_time: 0.,
            last_knockback_time: 0.,
            last_alerted: 0.,
            death_time: 0.,
            draw_attack: T::draw_attack(),
            sounds,
            knockback_cooldown: T::knockback_cooldown(),
        }
    }

    pub fn create_player(
        position: Vec2,
        collider_set: &mut ColliderSet,
        rigid_body_set: &mut RigidBodySet,
        sounds: &Sounds,
    ) -> Self {
        Self::create::<PlayerConfigProvider>(position, collider_set, rigid_body_set, sounds.clone())
    }

    pub fn create_guard(
        position: Vec2,
        collider_set: &mut ColliderSet,
        rigid_body_set: &mut RigidBodySet,
        sounds: &Sounds,
    ) -> Self {
        Self::create::<GuardConfigProvider>(position, collider_set, rigid_body_set, sounds.clone())
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

        if is_mouse_button_down(MouseButton::Left)
            && get_time() > self.last_attack_start + ATTACK_COOLDOWN
        {
            if !self.is_attacking {
                play_sound_once(&self.sounds.attack);
            }
            self.is_attacking = true;
            self.last_attack_start = get_time();
        }
        self.attack_direction = mouse_position_local().normalize_or_zero();

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
        if !self.is_alive() && self.body_handle.is_some() {
            self.destroy_physics(physics);
        }

        if !self.is_alive() || self.body_handle.is_none() {
            return;
        }

        // timeout attack
        if self.is_attacking && get_time() > self.last_attack_start + ATTACK_DURATION {
            self.is_attacking = false;
        }

        // set the attack collider position. attack collider is always centered around the player radius in the attack direction.
        if let Some(attack_collider_handle) = self.attack_collider_handle {
            let attack_collider = &mut physics.colliders[attack_collider_handle];
            let attack_direction = vector![self.attack_direction.x, self.attack_direction.y]
                * (PLAYER_ATTACK_RADIUS - PLAYER_RADIUS);
            attack_collider.set_position_wrt_parent(Isometry::translation(
                attack_direction.x,
                attack_direction.y,
            ));
        }

        // move the player
        let body = &mut physics.bodies[self.body_handle.unwrap()];

        let (move_acc, braking_acc) = if self.is_knockback_stunned() {
            (Vector2::zeros(), Vector2::zeros())
        } else {
            let move_acc = self.input_direction * self.acceleration;
            let move_acc = vector![move_acc.x, move_acc.y];

            let vel_dir = vec2(body.linvel().x, body.linvel().y).normalize_or_zero();
            let braking_acc =
                (self.input_direction - vel_dir) * body.linvel().magnitude() * self.braking;
            let braking_acc = vector![braking_acc.x, braking_acc.y];

            (move_acc, braking_acc)
        };

        let knockback = vector![self.accumulated_knockback.x, self.accumulated_knockback.y];
        self.accumulated_knockback = Vec2::ZERO;

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
        if self.body_handle.is_none() {
            return;
        }

        let body = &physics.bodies[self.body_handle.unwrap()];
        // TODO(axelmagn): snap to simulated pixel
        // mq -> nalgebra conversion
        self.position.x = body.translation().x - 0.5;
        self.position.y = body.translation().y - 0.5;

        if let Some(attack_collider_handle) = self.attack_collider_handle {
            let attack_collider = &physics.colliders[attack_collider_handle];
            self.attack_position.x = attack_collider.translation().x;
            self.attack_position.y = attack_collider.translation().y;
        }
    }

    pub fn draw(&self, tile_map: &TiledMap) {
        // draw attack
        if self.draw_attack && self.is_alive() {
            if self.is_attacking {
                draw_circle(
                    self.attack_position.x,
                    self.attack_position.y,
                    PLAYER_ATTACK_RADIUS,
                    WHITE,
                )
            } else {
                let draw_rect = Rect::new(
                    self.attack_position.x - 0.5,
                    self.attack_position.y - 0.5,
                    1.,
                    1.,
                );
                tile_map.spr(
                    TILESET_MAP_ID,
                    60, /* todo: move to constant */
                    draw_rect,
                );
            }
        }

        // draw player
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
        let origin = vec2(16., 16.);
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

    pub fn is_knockback_stunned(&self) -> bool {
        get_time() < self.last_knockback_time + self.knockback_cooldown
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

        let knockback_dir = (self.position - guard.position).normalize_or_zero();
        let knockback = knockback_dir * PLAYER_GUARD_KNOCKBACK;
        self.apply_knockback(knockback);
    }

    pub fn deal_damage(&mut self, amount: u32) {
        if !self.can_damage() || !self.is_alive() {
            return;
        }
        self.health -= amount.min(self.health);
        self.last_damage_time = get_time();

        if !self.is_alive() {
            self.death_time = get_time();
        }
    }

    pub fn apply_knockback(&mut self, delta_velocity: Vec2) {
        if !self.can_knockback() {
            return;
        }

        self.accumulated_knockback += delta_velocity;
        self.last_knockback_time = get_time();
        play_sound_once(&self.sounds.knockback);
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
        play_sound_once(&self.sounds.alert);
    }

    pub fn destroy_physics(&mut self, physics: &mut Physics) {
        if self.body_handle.is_none() {
            return;
        }
        physics.remove_body(&self.body_handle.unwrap(), true);
        self.body_handle = None;
        self.collider_handle = None;
    }

    pub fn handle_attack_collision(&mut self, guard: &mut Character) {
        if !self.is_attacking {
            return;
        }
        info!("ATTACK COLLISION");
        let knockback_dir = self.attack_direction;
        guard.apply_knockback(knockback_dir * PLAYER_ATTACK_KNOCKBACK);
    }

    pub fn center(&self) -> Vec2 {
        self.position + vec2(0.5, 0.5)
    }
}

pub trait CharacterConfigProvider {
    fn get_sprite_id() -> u32;
    fn get_acceleration() -> f32;
    fn get_braking() -> f32;
    fn get_max_health() -> u32;
    fn destroy_on_death() -> bool;
    fn draw_attack() -> bool;
    fn knockback_cooldown() -> f64;

    fn init_physics(
        position: Vec2,
        collider_set: &mut ColliderSet,
        rigid_body_set: &mut RigidBodySet,
    ) -> (ColliderHandle, RigidBodyHandle, Option<ColliderHandle>);
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
    ) -> (ColliderHandle, RigidBodyHandle, Option<ColliderHandle>) {
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
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .build();

        let attack_collider = ColliderBuilder::ball(PLAYER_ATTACK_RADIUS)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .sensor(true)
            .build();

        let body_handle = rigid_body_set.insert(body);
        let collider_handle =
            collider_set.insert_with_parent(collider, body_handle, rigid_body_set);
        let attack_collider_handle =
            collider_set.insert_with_parent(attack_collider, body_handle, rigid_body_set);
        (collider_handle, body_handle, Some(attack_collider_handle))
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

    fn destroy_on_death() -> bool {
        false
    }

    fn draw_attack() -> bool {
        true
    }

    fn knockback_cooldown() -> f64 {
        PLAYER_KNOCKBACK_COOLDOWN
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
    ) -> (ColliderHandle, RigidBodyHandle, Option<ColliderHandle>) {
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
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .build();

        let body_handle = rigid_body_set.insert(body);
        let collider_handle =
            collider_set.insert_with_parent(collider, body_handle, rigid_body_set);
        (collider_handle, body_handle, None)
    }

    fn get_max_health() -> u32 {
        GUARD_MAX_HEALTH
    }

    fn destroy_on_death() -> bool {
        true
    }

    fn draw_attack() -> bool {
        false
    }

    fn knockback_cooldown() -> f64 {
        GUARD_KNOCKBACK_COOLDOWN
    }
}
