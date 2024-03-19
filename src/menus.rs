use anyhow::Result;
use macroquad::{
    color::{DARKGRAY, WHITE},
    math::{vec2, RectOffset},
    texture::Image,
    ui::{root_ui, Skin},
    window::{clear_background, next_frame, screen_height, screen_width},
};

use crate::game::GameState;

pub struct MainMenu {
    skin: Skin,
    next_state: Option<GameState>,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            skin: base_skin(),
            next_state: None,
        }
    }

    pub async fn run(&mut self) -> Result<GameState> {
        loop {
            if let Some(next_state) = self.next_state {
                return Ok(next_state);
            }
            self.draw();
            next_frame().await
        }
    }

    pub fn draw(&mut self) {
        clear_background(DARKGRAY);
        root_ui().push_skin(&self.skin);
        root_ui().window(0, vec2(0., 0.), vec2(300., 300.), |ui| {
            ui.label(
                Some(vec2(screen_width() / 2. - 350., screen_height() * 2. / 5.)),
                "Escape from Stonehold",
            );

            if ui.button(
                vec2(screen_width() / 2. - 64., screen_height() * 3. / 5.),
                "Play",
            ) {
                // TODO(axelmagn): play sound
                // TODO(axelmagn): transition to instructions
                self.next_state = Some(GameState::InGame);
            };
        });
    }
}

pub fn base_skin() -> Skin {
    // TODO(axelmagn): customize for different screens
    let label_style = root_ui()
        .style_builder()
        .font(include_bytes!(
            "../assets/kenney_kenney-fonts/Fonts/Kenney Blocks.ttf"
        ))
        .unwrap()
        .text_color(WHITE)
        .font_size(48)
        .build();

    let window_style = root_ui()
        .style_builder()
        .background(
            Image::from_file_with_format(
                include_bytes!("../assets/kenney_ui-pack-rpg-expansion/PNG/panel_brown.png"),
                None,
            )
            .unwrap(),
        )
        .background_margin(RectOffset::new(20., 20., 10., 10.))
        .build();

    let button_style = root_ui()
        .style_builder()
        .background(
            Image::from_file_with_format(
                include_bytes!("../assets/kenney_ui-pack-rpg-expansion/PNG/buttonLong_blue.png"),
                None,
            )
            .unwrap(),
        )
        .background_hovered(
            Image::from_file_with_format(
                include_bytes!("../assets/kenney_ui-pack-rpg-expansion/PNG/buttonLong_beige.png"),
                None,
            )
            .unwrap(),
        )
        .background_margin(RectOffset::new(20., 20., 10., 10.))
        .background_clicked(
            Image::from_file_with_format(
                include_bytes!(
                    "../assets/kenney_ui-pack-rpg-expansion/PNG/buttonLong_beige_pressed.png"
                ),
                None,
            )
            .unwrap(),
        )
        .font(include_bytes!(
            "../assets/kenney_kenney-fonts/Fonts/Kenney Pixel Square.ttf"
        ))
        .unwrap()
        .text_color(WHITE)
        .text_color_hovered(WHITE)
        .text_color_clicked(WHITE)
        .font_size(32)
        .build();

    Skin {
        label_style,
        window_style,
        button_style,
        ..root_ui().default_skin()
    }
}
