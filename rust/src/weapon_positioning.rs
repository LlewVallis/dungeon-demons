use specs::prelude::*;
use std::f64::consts::PI;

use crate::components::bounds::Bounds;
use crate::components::player::Player;
use crate::components::sprite::character_animation::{CharacterAnimation, Facing};
use crate::components::sprite::sprite::Sprite;
use crate::components::sprite::FrameSprites;
use crate::ecs::WorldExtensions;
use crate::game::Timestamp;
use crate::gun::GunSpec;
use crate::util::vector::Vec2;
use crate::{vec2, Mat3};

const FLIPPING_ANGLE: f64 = 0.375 * PI;
const FLIPPING_HANDLE_REDUCTION: f64 = 0.005;

const HAND_OFFSET_RESOLUTION: f64 = 32.0;

const HAND_OFFSETS: &[&[Vec2]] = &[
    &[
        vec2(5.0, -3.0),
        vec2(4.0, -2.0),
        vec2(5.0, -3.0),
        vec2(6.0, -3.0),
    ],
    &[
        vec2(5.0, -3.0),
        vec2(4.0, -2.0),
        vec2(5.0, -3.0),
        vec2(6.0, -3.0),
    ],
    &[
        vec2(5.0, -3.0),
        vec2(5.0, -2.0),
        vec2(5.0, -3.0),
        vec2(5.0, -3.0),
    ],
    &[
        vec2(5.0, -3.0),
        vec2(5.0, -2.0),
        vec2(5.0, -3.0),
        vec2(5.0, -3.0),
    ],
    &[
        vec2(5.0, -3.0),
        vec2(5.0, -3.0),
        vec2(5.0, -3.0),
        vec2(5.0, -4.0),
    ],
    &[
        vec2(5.0, -3.0),
        vec2(5.0, -3.0),
        vec2(5.0, -3.0),
        vec2(5.0, -4.0),
    ],
    &[vec2(5.0, -3.0)],
    &[vec2(5.0, -3.0)],
    &[vec2(5.0, -3.0)],
    &[vec2(5.0, -3.0)],
];

const MIN_TARGETING_DISTANCE: f64 = 0.5;
const TARGETING_ITERATIONS: usize = 10;

pub fn player_targeting_position(world: &World, player_entity: Entity) -> Vec2 {
    let bounds = world.unwrap_read::<Bounds>(player_entity).0;
    let player = world.unwrap_read::<Player>(player_entity);

    target_offset(player.mouse_position(), bounds.center()) + bounds.center()
}

pub fn weapon_nose_position(world: &World, player_entity: Entity) -> Vec2 {
    let timestamp = world.fetch::<Timestamp>().0;

    let bounds = world.unwrap_read::<Bounds>(player_entity).0;
    let player = world.unwrap_read::<Player>(player_entity);
    let facing = world.unwrap_read::<Facing>(player_entity).0;
    let animation = world.unwrap_read::<CharacterAnimation>(player_entity);

    let gun = player.selected_gun().spec();

    let targeting_offset = target_offset(player.mouse_position(), bounds.center());
    let hand_offset = hand_offset(&*animation, facing, timestamp, targeting_offset);

    let direction = vec2(facing.x().signum(), 1.0);
    let flip_direction = flip_direction(targeting_offset);

    let weapon_angle = weapon_angle(gun, hand_offset, targeting_offset);

    let handle_to_nose = (gun.nose_offset() - gun.handle_offset()) * flip_direction;
    let nose_offset = handle_to_nose.rotate(weapon_angle) * direction;

    bounds.center() + hand_offset * direction + nose_offset
}

fn target_offset(mouse_position: Vec2, player_position: Vec2) -> Vec2 {
    let mut offset = mouse_position - player_position;

    if offset.length_squared() < MIN_TARGETING_DISTANCE.powi(2) {
        offset = offset.normalize_or(Vec2::right()) * MIN_TARGETING_DISTANCE
    }

    offset
}

fn should_flip(target_offset: Vec2) -> bool {
    target_offset.abs_x().angle() > FLIPPING_ANGLE
}

fn flip_direction(target_offset: Vec2) -> Vec2 {
    if should_flip(target_offset) {
        vec2(1.0, -1.0)
    } else {
        vec2(1.0, 1.0)
    }
}

fn hand_offset(
    animation: &CharacterAnimation,
    facing: Vec2,
    timestamp: f64,
    target_offset: Vec2,
) -> Vec2 {
    let frame_set = animation.current_frame_set(facing);
    let frame = animation.current_animation_frame(facing, timestamp);

    let offset = HAND_OFFSETS[frame_set][frame] / HAND_OFFSET_RESOLUTION;

    if should_flip(target_offset) {
        offset + Vec2::left() * FLIPPING_HANDLE_REDUCTION
    } else {
        offset
    }
}

fn weapon_angle(gun: &GunSpec, hand_offset: Vec2, target_offset: Vec2) -> f64 {
    let flip_direction = flip_direction(target_offset);

    let mouse_offset = target_offset.abs_x();

    let handle_to_nose = (gun.nose_offset() - gun.handle_offset()) * flip_direction;

    let mut weapon_angle = 0.0;

    for i in 0..TARGETING_ITERATIONS {
        let progress = i as f64 / (TARGETING_ITERATIONS - 1) as f64;
        let source = hand_offset + handle_to_nose.rotate(weapon_angle) * progress;

        let offset = mouse_offset - source;
        weapon_angle = offset.angle();
    }

    weapon_angle
}

pub struct DrawHeldWeapons;

#[derive(SystemData)]
pub struct DrawHeldWeaponsData<'a> {
    timestamp: ReadExpect<'a, Timestamp>,
    sprites: WriteExpect<'a, FrameSprites>,
    bounds: ReadStorage<'a, Bounds>,
    character_animations: ReadStorage<'a, CharacterAnimation>,
    facings: ReadStorage<'a, Facing>,
    players: ReadStorage<'a, Player>,
}

impl<'a> System<'a> for DrawHeldWeapons {
    type SystemData = DrawHeldWeaponsData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let iter = (
            &data.bounds,
            &data.character_animations,
            &data.facings,
            &data.players,
        )
            .join();

        for (bounds, animation, facing, player) in iter {
            if !player.is_mouse_down() || player.selected_gun().current_ammo() == 0 {
                continue;
            }

            let gun = player.selected_gun().spec();

            let target_offset = target_offset(player.mouse_position(), bounds.0.center());

            let hand_offset = hand_offset(animation, facing.0, data.timestamp.0, target_offset);
            let weapon_angle = weapon_angle(gun, hand_offset, target_offset);

            let direction = vec2(facing.0.x().signum(), 1.0);
            let flip_direction = flip_direction(target_offset);

            let transform = Mat3::scale(flip_direction * gun.texture_size());
            let transform = Mat3::translation(-gun.handle_offset() * flip_direction) * transform;
            let transform = Mat3::rotation(weapon_angle) * transform;
            let transform = Mat3::translation(hand_offset) * transform;
            let transform = Mat3::scale(direction) * transform;

            let sprite = Sprite::new_transformed(gun.texture(), transform);
            data.sprites.draw_sprite(bounds.0.center(), sprite);
        }
    }
}
