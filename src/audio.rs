use anyhow::Result;
use macroquad::audio::{load_sound, Sound};

use crate::constants::CLICK_SOUND_PATH;

// container class for different sounds
#[derive(Clone)]
pub struct Sounds {
    pub click: Sound,
}

impl Sounds {
    pub async fn load() -> Result<Self> {
        let click = load_sound(CLICK_SOUND_PATH).await?;
        Ok(Self { click })
    }
}
