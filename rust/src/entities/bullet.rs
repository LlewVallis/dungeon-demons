use specs::{Builder, Entity, World, WorldExt};

use crate::components::bounds::Bounds;
use crate::components::bullet::Bullet;
use crate::components::health::Health;
use crate::components::physics::Physics;
use crate::components::sprite::sprite::Sprite;
use crate::graphics::texture;
use crate::util::rect::Rect;
use crate::util::vector::Vec2;
use crate::{vec2, Mat3};

const SPEED: f64 = 20.0;
const SIZE: Vec2 = vec2(0.125, 0.125);

pub fn create(
    world: &mut World,
    owner: Entity,
    damage: f64,
    knockback: f64,
    penetration: f64,
    focus: Vec2,
    direction: Vec2,
) -> Entity {
    let sprite_transform = Mat3::rotation(direction.angle()) * Mat3::scale(SIZE);

    let mut physics = Physics::trigger(0.0);
    *physics.velocity_mut() = direction * SPEED;

    world
        .create_entity()
        .with(Bounds(Rect::focused(focus, SIZE)))
        .with(Sprite::new_transformed(texture::bullet(), sprite_transform))
        .with(physics)
        .with(Bullet::new(owner, damage, knockback))
        .with(Health::full((damage * penetration).max(1.0)))
        .build()
}
