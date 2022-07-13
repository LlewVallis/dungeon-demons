use float_ord::FloatOrd;
use specs::prelude::*;
use specs::{
    Component, Join, NullStorage, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage,
};

use crate::components::bounds::Bounds;
use crate::components::physics::Physics;
use crate::components::sprite::animation::{Animation, FrameSet};
use crate::components::sprite::sprite::Sprite;
use crate::components::sprite::FrameSprites;
use crate::game::Timestamp;
use crate::util::vector::Vec2;
use crate::Mat3;

const SAMPLE_BIAS_SCALE: f64 = 0.1;

pub struct CharacterAnimation {
    frame_sets: &'static [CharacterFrameSet],
    duration: f64,
    offset: Option<f64>,
    transform: Mat3,
    preference: usize,
}

impl CharacterAnimation {
    pub fn new_transformed(
        frame_sets: &'static [CharacterFrameSet],
        duration: f64,
        transform: Mat3,
    ) -> Self {
        Self {
            duration,
            frame_sets,
            transform,
            offset: None,
            preference: 0,
        }
    }

    pub fn new_sized(frame_sets: &'static [CharacterFrameSet], duration: f64, size: Vec2) -> Self {
        Self::new_transformed(frame_sets, duration, Mat3::scale(size))
    }

    pub fn current_frame_set(&self, facing: Vec2) -> usize {
        let facing_bias = self.frame_sets[self.preference].sample;

        let (index, _) = self
            .frame_sets
            .iter()
            .enumerate()
            .min_by_key(|(_, frame_set)| {
                let dist_sq = Vec2::distance_squared(frame_set.sample, facing);
                let bias_dist_sq = Vec2::distance_squared(frame_set.sample, facing_bias);
                FloatOrd(dist_sq + bias_dist_sq * SAMPLE_BIAS_SCALE)
            })
            .unwrap();

        index
    }

    pub fn current_animation_frame(&self, facing: Vec2, timestamp: f64) -> usize {
        let frame_set_index = self.current_frame_set(facing);
        let frame_set = &self.frame_sets[frame_set_index];

        Animation::new_transformed_with_offset(
            &frame_set.frame_set,
            self.duration,
            self.offset.unwrap_or(timestamp),
            self.transform,
        )
        .current_frame(timestamp)
    }

    pub fn update_sprite(&mut self, facing: Vec2, timestamp: f64) -> Sprite {
        let frame_set_index = self.current_frame_set(facing);
        let frame_set = &self.frame_sets[frame_set_index];

        self.preference = frame_set_index;

        let offset = *self.offset.get_or_insert(timestamp);

        let mut animation = Animation::new_transformed_with_offset(
            &frame_set.frame_set,
            self.duration,
            offset,
            self.transform,
        );

        animation.update_sprite(timestamp)
    }
}

impl Component for CharacterAnimation {
    type Storage = DenseVecStorage<Self>;
}

pub struct CharacterFrameSet {
    sample: Vec2,
    frame_set: FrameSet,
}

impl CharacterFrameSet {
    pub fn new(sample: Vec2, frame_set: FrameSet) -> Self {
        Self { sample, frame_set }
    }

    pub fn from_texture(sample: Vec2, texture: Mat3) -> Self {
        Self::new(sample, FrameSet::from_texture(texture))
    }

    pub fn from_texture_set(sample: Vec2, texture_set: impl IntoIterator<Item = Mat3>) -> Self {
        Self::new(sample, FrameSet::from_texture_set(texture_set))
    }
}

pub struct Facing(pub Vec2);

impl Component for Facing {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct FacesVelocity;

impl Component for FacesVelocity {
    type Storage = NullStorage<Self>;
}

pub struct FaceToVelocities;

impl<'a> System<'a> for FaceToVelocities {
    type SystemData = (
        WriteStorage<'a, Facing>,
        ReadStorage<'a, Physics>,
        ReadStorage<'a, FacesVelocity>,
    );

    fn run(&mut self, (mut facings, physicses, faces_velocities): Self::SystemData) {
        let iter = (&mut facings, &physicses, &faces_velocities).join();
        for (facing, physics, _) in iter {
            facing.0 = physics.velocity();
        }
    }
}

pub struct GenerateCharacterAnimationSprites;

#[derive(SystemData)]
pub struct GenerateCharacterAnimationSpritesData<'a> {
    frame_sprites: WriteExpect<'a, FrameSprites>,
    timestamp: ReadExpect<'a, Timestamp>,
    character_animation: WriteStorage<'a, CharacterAnimation>,
    bounds: ReadStorage<'a, Bounds>,
    facings: ReadStorage<'a, Facing>,
}

impl<'a> System<'a> for GenerateCharacterAnimationSprites {
    type SystemData = GenerateCharacterAnimationSpritesData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let iter = (&mut data.character_animation, &data.bounds, &data.facings).join();
        for (character_animation, bounds, facing) in iter {
            let sprite = character_animation.update_sprite(facing.0, data.timestamp.0);
            data.frame_sprites.draw_sprite(bounds.0.center(), sprite);
        }
    }
}
