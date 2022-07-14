use crate::util::vector::Vec2;
use crate::vec2;
use std::collections::HashSet;
use std::mem;

pub struct Inputs {
    mouse: Vec2,
    joystick: Vec2,
    mouse_down: bool,
    events: Vec<Event>,
    down_keys: HashSet<String>,
}

impl Inputs {
    pub fn new() -> Self {
        Self {
            mouse: Vec2::zero(),
            joystick: Vec2::zero(),
            mouse_down: false,
            events: Vec::new(),
            down_keys: HashSet::new(),
        }
    }

    pub fn joystick_mut(&mut self) -> &mut Vec2 {
        &mut self.joystick
    }
    
    pub fn joystick_tap(&mut self) {
        self.events.push(Event::JoystickTap);
    }

    pub fn mouse(&self) -> Vec2 {
        self.mouse
    }

    pub fn mouse_mut(&mut self) -> &mut Vec2 {
        &mut self.mouse
    }

    pub fn is_mouse_down(&self) -> bool {
        self.mouse_down
    }

    pub fn is_key_down(&self, key: &str) -> bool {
        self.down_keys.contains(key)
    }

    pub fn key_down(&mut self, key: String) {
        if !self.down_keys.contains(&key) {
            self.down_keys.insert(key.clone());
        }

        self.events.push(Event::KeyDown { key });
    }

    pub fn key_up(&mut self, key: String) {
        self.down_keys.remove(&key);

        self.events.push(Event::KeyUp { key });
    }

    pub fn mouse_down(&mut self, position: Vec2) {
        self.mouse_down = true;
        self.events.push(Event::MouseDown { position });
    }

    pub fn mouse_up(&mut self, position: Vec2) {
        self.mouse_down = false;
        self.events.push(Event::MouseUp { position });
    }

    pub fn poll(&mut self) -> impl Iterator<Item = Event> {
        let events = mem::replace(&mut self.events, Vec::new());
        events.into_iter()
    }

    pub fn movement_direction(&self) -> Vec2 {
        let mut direction = self.joystick;

        if self.is_key_down("KeyW") || self.is_key_down("ArrowUp") {
            direction += vec2(0.0, 1.0);
        }

        if self.is_key_down("KeyA") || self.is_key_down("ArrowLeft") {
            direction += vec2(-1.0, 0.0);
        }

        if self.is_key_down("KeyS") || self.is_key_down("ArrowDown") {
            direction += vec2(0.0, -1.0);
        }

        if self.is_key_down("KeyD") || self.is_key_down("ArrowRight") {
            direction += vec2(1.0, 0.0);
        }

        if direction.length() > 1.0 {
            direction.normalize()
        } else {
            direction
        }
    }
}

pub enum Event {
    KeyUp { key: String },
    KeyDown { key: String },
    MouseDown { position: Vec2 },
    MouseUp { position: Vec2 },
    JoystickTap,
}
