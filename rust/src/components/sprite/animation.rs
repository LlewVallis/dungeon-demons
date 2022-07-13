use specs::prelude::*;
use specs::{Component, ReadExpect, ReadStorage, System, WriteExpect};

use crate::components::bounds::Bounds;
use crate::components::sprite::{FrameSprites, Sprite};
use crate::game::Timestamp;
use crate::util::vector::Vec2;
use crate::Mat3;

pub struct Animation {
    frames: &'static FrameSet,
    duration: f64,
    offset: Option<f64>,
    transform: Mat3,
}

impl Animation {
    pub fn new_transformed_with_offset(
        frames: &'static FrameSet,
        duration: f64,
        offset: f64,
        transform: Mat3,
    ) -> Self {
        Self {
            frames,
            duration,
            transform,
            offset: Some(offset),
        }
    }

    pub fn new_transformed(frames: &'static FrameSet, duration: f64, transform: Mat3) -> Self {
        Self {
            frames,
            duration,
            transform,
            offset: None,
        }
    }

    pub fn new_sized(frames: &'static FrameSet, duration: f64, size: Vec2) -> Self {
        Self::new_transformed(frames, duration, Mat3::scale(size))
    }

    pub fn current_frame(&mut self, timestamp: f64) -> usize {
        let count = self.frames.textures.len();
        let offset = self.offset.unwrap_or(timestamp);

        let offset_time = timestamp - offset;
        let scaled_time = offset_time * count as f64 / self.duration;
        let progress = scaled_time % count as f64;
        (progress.floor() as usize).max(0).min(count - 1)
    }

    pub fn update_sprite(&mut self, timestamp: f64) -> Sprite {
        self.offset.get_or_insert(timestamp);

        let frame = self.current_frame(timestamp);
        let texture = self.frames.textures[frame];
        Sprite::new_transformed(texture, self.transform)
    }
}

impl Component for Animation {
    type Storage = DenseVecStorage<Self>;
}

pub struct FrameSet {
    textures: Vec<Mat3>,
}

impl FrameSet {
    pub fn from_texture(texture: Mat3) -> Self {
        Self::from_texture_set([texture])
    }

    pub fn from_texture_set(texture_set: impl IntoIterator<Item = Mat3>) -> Self {
        let textures = texture_set.into_iter().collect();
        Self { textures }
    }
}

pub struct GenerateAnimationSprites;

#[derive(SystemData)]
pub struct GenerateAnimationSpritesData<'a> {
    frame_sprites: WriteExpect<'a, FrameSprites>,
    timestamp: ReadExpect<'a, Timestamp>,
    animations: WriteStorage<'a, Animation>,
    bounds: ReadStorage<'a, Bounds>,
}

impl<'a> System<'a> for GenerateAnimationSprites {
    type SystemData = GenerateAnimationSpritesData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (animation, bounds) in (&mut data.animations, &data.bounds).join() {
            let sprite = animation.update_sprite(data.timestamp.0);
            data.frame_sprites.draw_sprite(bounds.0.center(), sprite);
        }
    }
}
