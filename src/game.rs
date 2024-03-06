use crate::{
    camera::Cameras,
    character::Character,
    constants::{
        GUARD_START_POSITIONS, PLAYER_SPRITE_ID, PLAYER_START_POSITION, TERRAIN_MAP_ID,
        TILESET_COLUMNS, TILESET_MAP_ID, TILESET_TEXTURE_PATH, TILESET_TILE_SIZE,
    },
    map::{Map, MapGenerator},
    physics::Physics,
};
use anyhow::Result;
use macroquad::{
    camera::set_camera,
    color::{Color, DARKGRAY, WHITE},
    logging::info,
    math::{uvec2, vec2},
    text::draw_text,
    time::get_fps,
    window::{clear_background, next_frame},
};

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
        // TODO(axelmagn): dynamic position
        let player = Character::create_player(
            PLAYER_START_POSITION,
            &mut physics.colliders,
            &mut physics.bodies,
        );

        let guards = GUARD_START_POSITIONS
            .iter()
            .map(|pos| Character::create_guard(*pos, &mut physics.colliders, &mut physics.bodies))
            .collect();

        // DEBUG: mapgen (BROKEN)
        // let mapgen = MapGenerator {
        //     ground_tile_id: 1,
        //     wall_tile_id: 49,
        //     tileset_id: TILESET_MAP_ID.into(),
        //     size: uvec2(64, 48),
        //     min_room_size: uvec2(3, 3),
        //     max_room_size: uvec2(10, 10),
        //     max_room_count: 10,
        // };
        // let layer = mapgen.generate_layer();
        // let mut map = map;
        // map.tile_map.layers.insert(TERRAIN_MAP_ID.into(), layer);

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
    }

    fn update(&mut self) {
        // update player
        self.player.update(&mut self.physics);

        // tick physics
        let (collision_recv, contact_force_recv) = self.physics.step();

        // DEBUG
        while let Ok(collision_event) = collision_recv.try_recv() {
            // Handle the collision event.
            info!("Received collision event: {:?}", collision_event);
        }
        while let Ok(contact_force_event) = contact_force_recv.try_recv() {
            // Handle the contact force event.
            info!("Received contact force event: {:?}", contact_force_event);
        }

        self.player.post_physics(&mut self.physics);

        for guard in &mut self.guards {
            guard.post_physics(&mut self.physics);
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
        draw_text(&format!("FPS: {:4.0}", get_fps()), 10., 20., 20., WHITE);
    }

    fn draw_screen(&self) {
        // draw full screen quad with previously rendered screen
        set_camera(&self.cameras.screen_camera);
        self.cameras.draw_world_render_to_screen();
        self.cameras.draw_ui_render_to_screen();
    }
}
