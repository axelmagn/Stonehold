use macroquad::{
    color::{DARKGRAY, RED, WHITE},
    input::{is_key_down, KeyCode},
    shapes::draw_rectangle,
    text::draw_text,
    time::get_frame_time,
    window::{clear_background, next_frame, screen_width},
};

struct Game {
    // tmp
    rect_size: f32,
    position: f32,
    speed: f32,
    speed_min: f32,
    speed_max: f32,
    speed_sensitivity: f32,
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
        }
    }
}

impl Lifecycle for Game {
    fn handle_inputs(&mut self, delta_time: f32) {
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

    fn render(&mut self) {
        clear_background(DARKGRAY);
        draw_text(&format!("Speed: {:8.02}", self.speed), 50., 50., 22., WHITE);
        draw_rectangle(self.position, 200.0, self.rect_size, self.rect_size, RED);
    }
}

trait Lifecycle {
    fn handle_inputs(&mut self, delta_time: f32);

    fn render(&mut self);
}

#[macroquad::main("Stonehold")]
async fn main() {
    let mut game = Game::new(screen_width());

    loop {
        // abbreviate delta time
        let dt = get_frame_time();

        game.handle_inputs(dt);

        game.render();

        // wait for next frame
        next_frame().await
    }
}
