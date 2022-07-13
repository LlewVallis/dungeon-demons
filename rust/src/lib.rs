#![feature(split_array)]
#![feature(box_syntax)]
#![feature(int_roundings)]
#![feature(drain_filter)]
#![feature(int_log)]
#![feature(new_uninit)]
#![feature(array_zip)]

extern crate core;

use std::panic;

use js_sys::Function;
use log::LevelFilter;
use wasm_bindgen::prelude::*;

use crate::game::Game;
use crate::input::Inputs;
use crate::util::mat3::Mat3;
use crate::util::vector::vec2;

mod audio;
mod camera;
mod components;
mod ecs;
mod entities;
mod game;
mod graphics;
mod gun;
mod input;
mod interaction;
mod logging;
mod map;
mod progression;
mod util;
mod weapon_hud;
mod weapon_positioning;

#[wasm_bindgen(start)]
pub fn start() {
    log::set_logger(logging::LOGGER).unwrap();
    log::set_max_level(LevelFilter::Trace);

    log::info!("WASM library started");

    panic::set_hook(Box::new(|info| {
        log::error!("Panicked: {}", info);
    }));
}

#[wasm_bindgen]
pub struct Backend {
    game: Game,
}

#[wasm_bindgen]
impl Backend {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Self {
        Self {
            game: Game::new(seed),
        }
    }

    #[wasm_bindgen(js_name = keyDown)]
    pub fn key_down(&mut self, key: String) {
        self.game.inputs().key_down(key);
    }

    #[wasm_bindgen(js_name = keyUp)]
    pub fn key_up(&mut self, key: String) {
        self.game.inputs().key_up(key);
    }

    #[wasm_bindgen(js_name = updateMouse)]
    pub fn update_mouse(&mut self, x: f64, y: f64) {
        *self.game.inputs().mouse_mut() = vec2(x, y);
    }

    #[wasm_bindgen(js_name = mouseDown)]
    pub fn mouse_down(&mut self, x: f64, y: f64) {
        self.game.inputs().mouse_down(vec2(x, y));
    }

    #[wasm_bindgen(js_name = mouseUp)]
    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.game.inputs().mouse_up(vec2(x, y));
    }

    #[wasm_bindgen(js_name = entityCount)]
    pub fn entity_count(&self) -> usize {
        self.game.entity_count()
    }

    #[wasm_bindgen]
    pub fn round(&self) -> usize {
        self.game.round()
    }

    #[wasm_bindgen]
    pub fn credits(&self) -> usize {
        self.game.credits().unwrap_or(0)
    }

    #[wasm_bindgen(js_name = interactionLine)]
    pub fn interaction_heading(&self) -> String {
        self.game.interaction_heading()
    }

    #[wasm_bindgen(js_name = interactionCaption)]
    pub fn interaction_caption(&self) -> String {
        self.game.interaction_caption()
    }

    #[wasm_bindgen(js_name = currentAmmo)]
    pub fn current_ammo(&self) -> usize {
        self.game.current_ammo().unwrap_or(0)
    }

    #[wasm_bindgen(js_name = maxAmmo)]
    pub fn max_ammo(&self) -> usize {
        self.game.max_ammo().unwrap_or(0)
    }

    #[wasm_bindgen]
    pub fn tick(&mut self, delta: f64) {
        self.game.tick(delta);
    }

    #[wasm_bindgen]
    pub fn draw(&mut self, drawer: Function, aspect_ratio: f64) {
        let ptr = self.game.draw(aspect_ratio);

        let this = JsValue::null();
        let ptr = JsValue::from_f64(ptr as usize as f64);

        let _ = drawer.call1(&this, &ptr);
    }

    #[wasm_bindgen(js_name = enableHud)]
    pub fn enable_hud(&mut self) {
        self.game.enable_hud();
    }

    #[wasm_bindgen(js_name = isOver)]
    pub fn is_over(&self) -> bool {
        self.game.is_over()
    }
}

fn current_time() -> f64 {
    web_sys::window().unwrap().performance().unwrap().now()
}
