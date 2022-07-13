use crate::components::bounds::Bounds;
use specs::prelude::*;

use crate::components::sprite::FrameSprites;
use crate::util::vector::Vec2;
use crate::Mat3;

#[derive(Copy, Clone)]
pub struct Sprite {
    texture: Mat3,
    transform: Mat3,
}

impl Sprite {
    pub fn new_transformed(texture: Mat3, transform: Mat3) -> Self {
        Self { transform, texture }
    }

    pub fn new_sized(texture: Mat3, size: Vec2) -> Self {
        Self::new_transformed(texture, Mat3::scale(size))
    }

    pub fn texture(&self) -> Mat3 {
        self.texture
    }

    pub fn transform(&self) -> Mat3 {
        self.transform
    }
}

impl Component for Sprite {
    type Storage = DenseVecStorage<Self>;
}

pub struct GenerateStaticSprites;

#[derive(SystemData)]
pub struct GenerateStaticSpritesData<'a> {
    frame_sprites: WriteExpect<'a, FrameSprites>,
    sprites: ReadStorage<'a, Sprite>,
    bounds: ReadStorage<'a, Bounds>,
}

impl<'a> System<'a> for GenerateStaticSprites {
    type SystemData = GenerateStaticSpritesData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (sprite, bounds) in (&data.sprites, &data.bounds).join() {
            data.frame_sprites.draw_sprite(bounds.0.center(), *sprite);
        }
    }
}
