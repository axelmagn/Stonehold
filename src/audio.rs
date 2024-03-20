use anyhow::Result;
use futures::try_join;
use macroquad::audio::{load_sound, Sound};

use crate::constants::{
    ALERT_SOUND_PATH, ATTACK_SOUND_PATH, CLICK_SOUND_PATH, DEFEAT_SOUND_PATH,
    DOOR_CLOSE_SOUND_PATH, KNOCKBACK_SOUND_PATH, VICTORY_SOUND_PATH,
};

// container class for different sounds
#[derive(Clone, Debug)]
pub struct Sounds {
    pub click: Sound,
    pub attack: Sound,
    pub knockback: Sound,
    pub alert: Sound,
    pub close_door: Sound,
    pub victory: Sound,
    pub defeat: Sound,
}

impl Sounds {
    pub async fn load() -> Result<Self> {
        let (click, attack, knockback, alert, close_door, victory, defeat) = try_join!(
            load_sound(CLICK_SOUND_PATH),
            load_sound(ATTACK_SOUND_PATH),
            load_sound(KNOCKBACK_SOUND_PATH),
            load_sound(ALERT_SOUND_PATH),
            load_sound(DOOR_CLOSE_SOUND_PATH),
            load_sound(VICTORY_SOUND_PATH),
            load_sound(DEFEAT_SOUND_PATH),
        )?;
        Ok(Self {
            click,
            attack,
            knockback,
            alert,
            close_door,
            victory,
            defeat,
        })
    }
}
