use macroquad::prelude::*;

#[macroquad::main("Stonehold")]
async fn main() {
    let rect_size = 120.;
    let mut position = (screen_width() - rect_size) / 2.0;
    let mut speed = 500.;
    let speed_min = 100.;
    let speed_max = 1000.;
    let speed_sensitivity = 100.;

    loop {
        // abbreviate delta time
        let dt = get_frame_time();

        // update speed
        if is_key_down(KeyCode::Up) {
            speed += speed_sensitivity * dt;
        }
        if is_key_down(KeyCode::Down) {
            speed -= speed_sensitivity * dt;
        }
        speed = speed.clamp(speed_min, speed_max);

        // update position
        if is_key_down(KeyCode::Left) {
            position -= speed * dt;
        }
        if is_key_down(KeyCode::Right) {
            position += speed * dt;
        }
        position = position.clamp(0., screen_width() - rect_size);

        // render
        clear_background(DARKGRAY);
        draw_text(&format!("Speed: {:8.02}", speed), 50., 50., 22., WHITE);
        draw_rectangle(position, 200.0, rect_size, rect_size, RED);

        // wait for next frame
        next_frame().await
    }
}
