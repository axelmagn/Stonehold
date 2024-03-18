use crate::{
    camera::Cameras,
    character::Character,
    constants::{GUARD_SPRITE_ID, SIMULATED_RESOLUTION, TERRAIN_MAP_ID, TILESET_MAP_ID},
    door::{ExitDoor, GuardDoor},
    map::{
        mapgen::{MapGenResult, MapGenerator},
        Map,
    },
    physics::Physics,
};
use anyhow::Result;
use macroquad::{
    camera::set_camera,
    color::{Color, DARKGRAY, WHITE},
    logging::info,
    math::{uvec2, Rect},
    rand::srand,
    text::draw_text,
    time::get_time,
    window::{clear_background, next_frame},
};
use rapier2d::geometry::CollisionEvent;

pub enum GameState {
    MainMenu,
    Instructions,
    InGame(Game),
    GameOver,
}

pub struct Game {
    pub map: Map,
    pub player: Character,
    pub guards: Vec<Character>,
    pub guard_doors: Vec<GuardDoor>,
    pub exit_door: ExitDoor,
    pub physics: Physics,
    pub cameras: Cameras,
    pub score: u32,
    pub score_target: u32,
}

impl Game {
    pub fn new(map: Map) -> Self {
        let mut physics = Physics::default();
        let seed = (get_time() % 1. * (u64::MAX as f64)) as u64;
        info!("Random Seed: {}", seed);
        srand(seed);

        let mapgen = MapGenerator::new(uvec2(
            map.tile_map.raw_tiled_map.width,
            map.tile_map.raw_tiled_map.height,
        ));

        let MapGenResult {
            rooms,
            layer,
            guard_doors,
            exit_door,
        } = mapgen.generate_layer();
        let mut map = map;
        map.tile_map.layers.insert(TERRAIN_MAP_ID.into(), layer);
        info!("rooms: {:?}", rooms);

        let player = Character::create_player(
            rooms[0].center(),
            &mut physics.colliders,
            &mut physics.bodies,
        );

        let guards = rooms[1..]
            .iter()
            .map(|room| {
                Character::create_guard(room.center(), &mut physics.colliders, &mut physics.bodies)
            })
            .collect();

        let guard_doors: Vec<GuardDoor> = guard_doors
            .iter()
            .map(|position| GuardDoor::create(*position, &mut physics.colliders))
            .collect();

        // DEBUG
        // let score_target = guard_doors.len() as u32 / 2;
        let score_target = 1;

        let exit_door = ExitDoor::create(exit_door, &mut physics.colliders);

        Self {
            map,
            player,
            guards,
            guard_doors,
            exit_door,
            physics,
            cameras: Cameras::new(),
            score: 0,
            score_target,
        }
    }

    pub async fn load() -> Result<Self> {
        let map = Map::load().await?;

        Ok(Self::new(map))
    }

    pub fn setup(&mut self) {
        self.map.init_colliders(&mut self.physics.colliders);
    }

    pub async fn run(&mut self) -> Result<()> {
        self.setup();
        loop {
            self.collect_inputs();
            self.update();
            self.draw();
            next_frame().await
        }
    }

    fn collect_inputs(&mut self) {
        self.player.collect_player_inputs();

        for guard in &mut self.guards {
            guard.collect_guard_inputs(&self.player);
        }
    }

    fn update(&mut self) {
        // update player
        self.player.update(&mut self.physics);

        // update guards
        for guard in &mut self.guards {
            guard.update(&mut self.physics);
        }

        // tick physics
        let (collision_recv, contact_force_recv) = self.physics.step();

        self.player.post_physics(&mut self.physics);

        while let Ok(collision_event) = collision_recv.try_recv() {
            self.handle_collision(&collision_event);
        }

        // handle player attack
        if self.player.is_attacking && self.player.attack_collider_handle.is_some() {
            for guard in &mut self.guards {
                if guard.collider_handle.is_some()
                    && self.physics.narrow_phase.intersection_pair(
                        self.player.attack_collider_handle.unwrap(),
                        guard.collider_handle.unwrap(),
                    ) == Some(true)
                {
                    self.player.handle_attack_collision(guard);
                }
            }
        }

        // handle guard door collisions
        let mut removed_guards = Vec::new();
        for (_i, door) in self.guard_doors.iter_mut().enumerate() {
            if !door.is_open {
                continue;
            }
            for (j, guard) in &mut self.guards.iter_mut().enumerate() {
                if !guard.collider_handle.is_some() {
                    continue;
                }

                if self
                    .physics
                    .narrow_phase
                    .intersection_pair(door.collider_handle, guard.collider_handle.unwrap())
                    == Some(true)
                {
                    door.close_door(self.map.tile_map.layers.get_mut(TERRAIN_MAP_ID).unwrap());
                    removed_guards.push(j);
                }
            }
        }
        // clean up removed guards
        removed_guards.sort();
        for i in removed_guards.iter().rev() {
            self.guards[*i].destroy_physics(&mut self.physics);
            self.guards.remove(*i);
        }
        self.score += removed_guards.len() as u32;

        // open exit if needed
        if !self.exit_door.is_open && self.score >= self.score_target {
            self.exit_door
                .open_door(self.map.tile_map.layers.get_mut(TERRAIN_MAP_ID).unwrap());
        }

        // handle player exit
        if self.exit_door.is_open
            && self.player.collider_handle.is_some()
            && self.physics.narrow_phase.intersection_pair(
                self.player.collider_handle.unwrap(),
                self.exit_door.collider_handle,
            ) == Some(true)
        {
            panic!("YOU WIN!!!!");
        }

        while let Ok(_contact_force_event) = contact_force_recv.try_recv() {
            // Handle the contact force event.
            // info!("Received contact force event: {:?}", contact_force_event);
        }

        for guard in &mut self.guards {
            guard.post_physics(&mut self.physics);
        }

        // check guard distance to player
        for guard in &mut self.guards {
            guard.check_guard_distance(&self.player);
        }

        // update cameras (position on player, etc)
        self.cameras.update(self.player.position);
    }

    fn draw(&self) {
        clear_background(DARKGRAY);
        self.draw_world();
        self.draw_ui();
        self.draw_screen();
    }

    fn draw_world(&self) {
        // setup drawing for worldspace
        set_camera(&self.cameras.world_camera);

        // draw map
        self.map.draw();

        // draw player
        self.player.draw(&self.map.tile_map);

        // draw guards
        self.guards
            .iter()
            .for_each(|guard| guard.draw(&self.map.tile_map));
    }

    fn draw_ui(&self) {
        // setup drawing for UI space
        set_camera(&self.cameras.ui_camera);
        clear_background(Color::new(0., 0., 0., 0.));
        self.player.draw_ui(&self.map.tile_map);

        // draw score
        let score_rect = Rect::new(SIMULATED_RESOLUTION.x as f32 - 128., 16., 32., 32.);
        self.map
            .tile_map
            .spr(TILESET_MAP_ID, GUARD_SPRITE_ID, score_rect);
        draw_text(
            &format!("{}/{}", self.score, self.score_target),
            score_rect.x + 48.,
            score_rect.y + 32.,
            48.,
            WHITE,
        );
    }

    fn draw_screen(&self) {
        // draw full screen quad with previously rendered screen
        set_camera(&self.cameras.screen_camera);
        self.cameras.draw_world_render_to_screen();
        self.cameras.draw_ui_render_to_screen();
    }

    fn handle_collision(&mut self, collision_event: &CollisionEvent) {
        let c1_is_player = Some(collision_event.collider1()) == self.player.collider_handle;
        let guard = self
            .guards
            .iter_mut()
            .find(|guard| guard.collider_handle == Some(collision_event.collider2()));

        if c1_is_player && guard.is_some() {
            self.player
                .handle_player_guard_collision(guard.as_ref().unwrap());
        }
    }
}
