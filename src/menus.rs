use anyhow::Result;
use macroquad::{
    color::{DARKGRAY, LIGHTGRAY, WHITE},
    math::{vec2, RectOffset},
    texture::Image,
    ui::{root_ui, widgets, Skin, Style},
    window::{clear_background, next_frame, screen_height, screen_width},
};

pub struct MainMenu {
    skin: Skin,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            skin: Self::make_skin(),
        }
    }

    pub fn make_skin() -> Skin {
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
                    include_bytes!(
                        "../assets/kenney_ui-pack-rpg-expansion/PNG/buttonLong_blue.png"
                    ),
                    None,
                )
                .unwrap(),
            )
            .background_hovered(
                Image::from_file_with_format(
                    include_bytes!(
                        "../assets/kenney_ui-pack-rpg-expansion/PNG/buttonLong_beige.png"
                    ),
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

    pub async fn run(&self) -> Result<()> {
        loop {
            self.draw();
            next_frame().await
        }
    }

    pub fn draw(&self) {
        clear_background(DARKGRAY);
        root_ui().push_skin(&self.skin);
        root_ui().window(0, vec2(0., 0.), vec2(300., 300.), |ui| {
            ui.label(
                Some(vec2(screen_width() / 2. - 350., screen_height() * 2. / 5.)),
                "Escape from Stonehold",
            );

            ui.button(
                vec2(screen_width() / 2. - 64., screen_height() * 3. / 5.),
                "Play",
            );
            // widgets::Label::new("Escape from Stonehold")
            //     .position(vec2(screen_width() / 2., screen_height() / 4.))
            //     .ui(ui);
            // widgets::Button::new("Play")
            //     .position(vec2(screen_width() / 2., screen_height() / 2.))
            //     .ui(ui);
        });
    }
}
