use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = dungeonDemonsPlaySound)]
    fn dispatch(name: &str);
}

#[derive(Copy, Clone)]
pub struct Sound {
    name: &'static str,
}

impl Sound {
    fn new(name: &'static str) -> Self {
        Self {
            name: wasm_bindgen::intern(name),
        }
    }
}

pub fn play_sound(sound: Sound) {
    dispatch(sound.name);
}

pub mod sound {
    use lazy_static::lazy_static;

    use crate::audio::Sound;

    macro_rules! sound {
        ($name:ident) => {
            pub fn $name() -> Sound {
                lazy_static! {
                    static ref SOUND: Sound = Sound::new(stringify!($name));
                }

                *SOUND
            }
        };
    }

    sound!(shoot_slow);
    sound!(shoot_fast);
    sound!(shoot_sniper);
    sound!(shoot_shotgun);
    sound!(hit);
    sound!(kill);
    sound!(player_hit);
    sound!(death);
    sound!(round_start);
    sound!(round_end);
    sound!(purchase);
    sound!(pickup);
}
