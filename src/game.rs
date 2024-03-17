use crate::{
    camera::Cameras,
    character::Character,
    constants::TERRAIN_MAP_ID,
    map::{
        mapgen::{MapGenResult, MapGenerator},
        Map,
    },
    physics::Physics,
};
use anyhow::Result;
use macroquad::{
    camera::set_camera,
    color::{Color, DARKGRAY},
    logging::info,
    math::uvec2,
    rand::srand,
    time::get_time,
    window::{clear_background, next_frame},
};
use rapier2d::geometry::CollisionEvent;

pub struct Game {
    pub map: Map,
    pub player: Character,
    pub guards: Vec<Character>,
    pub physics: Physics,
    pub cameras: Cameras,
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

        let MapGenResult { rooms, layer, .. } = mapgen.generate_layer();
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

        Self {
            map,
            player,
            guards,
            physics,
            cameras: Cameras::new(),
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
    }

    fn draw_screen(&self) {
        // draw full screen quad with previously rendered screen
        set_camera(&self.cameras.screen_camera);
        self.cameras.draw_world_render_to_screen();
        self.cameras.draw_ui_render_to_screen();
    }

    fn handle_collision(&mut self, collision_event: &CollisionEvent) {
        let c1_is_player = Some(collision_event.collider1()) == self.player.collider_handle;
        let c1_is_attack = Some(collision_event.collider1()) == self.player.attack_collider_handle;
        let guard = self
            .guards
            .iter_mut()
            .find(|guard| guard.collider_handle == Some(collision_event.collider2()));

        if c1_is_player && guard.is_some() {
            self.player
                .handle_player_guard_collision(guard.as_ref().unwrap());
        }

        if c1_is_attack && guard.is_some() {
            self.player.handle_attack_collision(guard.unwrap());
        }
    }
}
