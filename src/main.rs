use anyhow::Result;
use macroquad::{
    camera::{set_camera, Camera2D},
    color::{DARKGRAY, RED, WHITE},
    file::load_string,
    input::{is_key_down, KeyCode},
    math::Rect,
    shapes::draw_rectangle,
    text::draw_text,
    texture::load_texture,
    time::get_frame_time,
    window::{clear_background, next_frame, screen_width},
};
use macroquad_tiled::{load_map, Map};

struct Game {
    // tmp
    rect_size: f32,
    position: f32,
    speed: f32,
    speed_min: f32,
    speed_max: f32,
    speed_sensitivity: f32,

    // map
    assets: Option<GameAssets>,

    // camera
    camera: Camera2D,
}

impl Game {
    fn new(screen_width: f32) -> Self {
        let rect_size = 120.;
        let position = (screen_width - rect_size) / 2.;

        Self {
            rect_size,
            position,
            speed: 500.,
            speed_min: 100.,
            speed_max: 1000.,
            speed_sensitivity: 100.,

            assets: None,

            // TODO(axelmagn): fix magic numbers
            camera: Camera2D::from_display_rect(Self::get_display_rect()),
        }
    }

    fn get_assets_ref(&self) -> &GameAssets {
        &self.assets.as_ref().expect("assets not loaded")
    }

    fn get_display_rect() -> Rect {
        Rect::new(0., 0., 1024., 786.)
    }
}

impl Lifecycle for Game {
    async fn setup(&mut self) -> Result<()> {
        let tile_texture =
            load_texture("assets/kenney_tiny-dungeon/Tilemap/tilemap_packed.png").await?;
        let tileset_json = load_string("assets/tiled/tiny_dungeon.tsj").await?;
        let tile_map_json = load_string("assets/tiled/sandbox01.tmj").await?;

        let tile_map = load_map(
            &tile_map_json,
            &[(
                "../kenney_tiny-dungeon/Tilemap/tilemap_packed.png",
                tile_texture,
            )],
            &[("tiny_dungeon.tsx", &tileset_json)],
        )?;

        self.assets = Some(GameAssets { tile_map });

        Ok(())
    }

    async fn handle_inputs(&mut self, delta_time: f32) {
        let dt = delta_time;

        // update speed
        if is_key_down(KeyCode::Up) {
            self.speed += self.speed_sensitivity * dt;
        }
        if is_key_down(KeyCode::Down) {
            self.speed -= self.speed_sensitivity * dt;
        }
        self.speed = self.speed.clamp(self.speed_min, self.speed_max);

        // update position
        if is_key_down(KeyCode::Left) {
            self.position -= self.speed * dt;
        }
        if is_key_down(KeyCode::Right) {
            self.position += self.speed * dt;
        }
        self.position = self.position.clamp(0., screen_width() - self.rect_size);
    }

    async fn render(&mut self) {
        clear_background(DARKGRAY);
        set_camera(&self.camera);

        self.get_assets_ref()
            .tile_map
            .draw_tiles("terrain", Self::get_display_rect(), None);

        draw_text(&format!("Speed: {:8.02}", self.speed), 50., 50., 22., WHITE);
        draw_rectangle(self.position, 200.0, self.rect_size, self.rect_size, RED);
    }
}

struct GameAssets {
    tile_map: Map,
}

trait Lifecycle {
    async fn setup(&mut self) -> Result<()>;

    async fn handle_inputs(&mut self, delta_time: f32);

    async fn render(&mut self);
}

#[macroquad::main("Stonehold")]
async fn main() {
    let mut game = Game::new(screen_width());

    game.setup().await.expect("Game setup failed");

    loop {
        // abbreviate delta time
        let dt = get_frame_time();

        game.handle_inputs(dt).await;

        game.render().await;

        // wait for next frame
        next_frame().await
    }
}
