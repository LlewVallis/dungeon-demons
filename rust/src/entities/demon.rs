use lazy_static::lazy_static;
use specs::{Builder, Entity, World, WorldExt};

use crate::components::bounds::Bounds;
use crate::components::bullet::BulletTarget;
use crate::components::enemy::Enemy;
use crate::components::health::Health;
use crate::components::melee_attacker::MeleeAttacker;
use crate::components::physics::{Collider, Physics};
use crate::components::player_seeker::PlayerSeeker;
use crate::components::spawning_demon::SpawningDemon;
use crate::components::sprite::animation::Animation;
use crate::components::sprite::animation::FrameSet;
use crate::components::sprite::character_animation::CharacterFrameSet;
use crate::components::sprite::character_animation::{CharacterAnimation, Facing};
use crate::graphics::texture;
use crate::util::random::Random;
use crate::util::rect::Rect;
use crate::util::vector::Vec2;
use crate::vec2;

const SIZE: Vec2 = vec2(0.333, 0.425);
const DRAG: f64 = 5.0;

const DAMAGE: f64 = 25.0;
const COOLDOWN: f64 = 1.0;
const DELAY: f64 = 0.2;

lazy_static! {
    static ref FRAME_SETS: [CharacterFrameSet; 2] = [
        CharacterFrameSet::from_texture_set(Vec2::left(), texture::demon_left()),
        CharacterFrameSet::from_texture_set(Vec2::right(), texture::demon_right()),
    ];
    static ref SPAWNING_FRAMES: FrameSet = FrameSet::from_texture_set(texture::spawning_demon());
}

pub fn create_spawning(world: &mut World, focus: Vec2, health: f64, speed: f64) -> Entity {
    let spawn_time = Random::global().next_f64_in(0.5..1.15);

    world
        .create_entity()
        .with(SpawningDemon::new(spawn_time, speed))
        .with(Bounds(Rect::focused(focus, SIZE)))
        .with(Animation::new_sized(
            &*SPAWNING_FRAMES,
            spawn_time,
            vec2(0.5, 0.5),
        ))
        .with(Collider)
        .with(Health::full(health))
        .with(BulletTarget)
        .with(Enemy)
        .build()
}

pub fn create(world: &mut World, focus: Vec2, health: Health, speed: f64) -> Entity {
    world
        .create_entity()
        .with(Bounds(Rect::focused(focus, SIZE)))
        .with(Facing(Vec2::zero()))
        .with(CharacterAnimation::new_sized(
            &*FRAME_SETS,
            0.5,
            vec2(0.5, 0.5),
        ))
        .with(PlayerSeeker::new(speed))
        .with(MeleeAttacker::new(DAMAGE, COOLDOWN, DELAY))
        .with(Physics::collider(DRAG))
        .with(Collider)
        .with(health)
        .with(BulletTarget)
        .with(Enemy)
        .build()
}
