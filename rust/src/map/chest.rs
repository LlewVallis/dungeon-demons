use fxhash::FxHashSet;
use specs::{Entity, World};
use std::ops::Deref;
use std::rc::Rc;

use crate::components::player::Player;
use crate::components::sprite::sprite::Sprite;
use crate::components::sprite::FrameSprites;
use crate::ecs::WorldExtensions;
use crate::graphics::texture;
use crate::gun::{Gun, GunSpec, GunSpecGenerator};
use crate::util::rect::Rect;
use crate::util::vector::Vec2;
use crate::{vec2, Mat3};

pub struct Chest {
    position: Vec2,
    gun: Option<Rc<GunSpec>>,
    exhausted_players: FxHashSet<Entity>,
}

impl Chest {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            gun: None,
            exhausted_players: FxHashSet::default(),
        }
    }

    pub fn can_pickup(&self, player: Entity) -> bool {
        self.gun.is_some() && !self.exhausted_players.contains(&player)
    }

    pub fn pickup(&mut self, world: &World, player: Entity) {
        if !self.can_pickup(player) {
            return;
        }

        self.exhausted_players.insert(player);

        let mut player = world.unwrap_write::<Player>(player);

        let gun_spec = self.gun.as_ref().unwrap().clone();
        player.equip_gun(Gun::new(gun_spec));
    }

    pub fn gun(&self) -> Option<&GunSpec> {
        self.gun.as_ref().map(Rc::deref)
    }

    pub fn open(&mut self, world: &World) {
        if self.gun.is_some() {
            return;
        }

        let gun_spec = world
            .fetch_mut::<GunSpecGenerator>()
            .generate(self.position);

        self.gun = Some(Rc::new(gun_spec));
    }

    pub fn is_open(&self) -> bool {
        self.gun.is_some()
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn draw(&self, timestamp: f64, player: Option<Entity>, sprites: &mut FrameSprites) {
        if let Some(player) = player {
            if self.exhausted_players.contains(&player) {
                return;
            }
        } else {
            return;
        }

        if let Some(gun) = &self.gun {
            let y = 0.21 + (timestamp * 1.5).sin() * 0.05;

            let transform = Mat3::transform(vec2(0.0, y), vec2(0.4, 0.4));

            let gun_sprite = Sprite::new_transformed(gun.texture(), transform);
            sprites.draw_sprite(self.position, gun_sprite);
        }

        let texture = if self.is_open() {
            texture::open_chest()
        } else {
            texture::closed_chest()
        };

        let sprite = Sprite::new_sized(texture, self.bounds().size);
        sprites.draw_sprite(self.position, sprite);
    }

    pub fn bounds(&self) -> Rect {
        Rect::focused(self.position, vec2(0.5, 0.5))
    }
}
