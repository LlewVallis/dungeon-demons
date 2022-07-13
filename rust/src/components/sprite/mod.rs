use float_ord::FloatOrd;
use specs::prelude::*;

use crate::camera::Camera;
use crate::components::sprite::sprite::Sprite;
use crate::graphics::{DrawBuffer, EntityRendererSettings};
use crate::util::rect::Rect;
use crate::util::vector::Vec2;
use crate::{vec2, Mat3};

pub mod animation;
pub mod character_animation;
pub mod sprite;

pub struct FrameSprites {
    sprites: Vec<(Vec2, Sprite)>,
}

impl FrameSprites {
    pub fn new() -> Self {
        Self {
            sprites: Vec::new(),
        }
    }

    pub fn draw_sprite(&mut self, position: Vec2, sprite: Sprite) {
        self.sprites.push((position, sprite));
    }
}

pub struct DrawSprites;

#[derive(SystemData)]
pub struct DrawSpritesData<'a> {
    sprites: WriteExpect<'a, FrameSprites>,
    buffer: WriteExpect<'a, DrawBuffer>,
    camera: ReadExpect<'a, Camera>,
}

impl<'a> System<'a> for DrawSprites {
    type SystemData = DrawSpritesData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        data.sprites
            .sprites
            .sort_by_key(|(position, _)| FloatOrd(-position.y()));

        let view_bounds = data.camera.bounds();

        for (position, sprite) in &data.sprites.sprites {
            let translation = Mat3::translation(*position);
            let transform = translation * sprite.transform();

            let aabb = Rect::focused(Vec2::zero(), vec2(0.5, 0.5));
            let aabb = aabb.transform_aabb(transform);

            if !aabb.touches(view_bounds) {
                continue;
            }

            data.buffer.push(EntityRendererSettings {
                transform,
                uv_transform: sprite.texture(),
            });
        }

        data.sprites.sprites.clear();
    }
}
