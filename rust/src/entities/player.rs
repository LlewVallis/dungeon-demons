use std::f64::consts::PI;
use std::mem;

use lazy_static::lazy_static;
use specs::{Builder, Entity, World, WorldExt};

use crate::components::bounds::Bounds;
use crate::components::health::Health;
use crate::components::physics::{Collider, Physics};
use crate::components::player::{AttackResult, Player};
use crate::components::regen::HealthRegen;
use crate::components::sprite::character_animation::{
    CharacterAnimation, CharacterFrameSet, Facing,
};
use crate::ecs::WorldExtensions;
use crate::entities::bullet;
use crate::graphics::texture;
use crate::input::Event;
use crate::interaction::Interaction;
use crate::util::random::Random;
use crate::util::rect::Rect;
use crate::util::vector::Vec2;
use crate::weapon_positioning::{player_targeting_position, weapon_nose_position};
use crate::{vec2, Inputs};

const SPEED: f64 = 3.0;
const DRAG: f64 = 10.0;
const HEALTH: f64 = 100.0;
const SIZE: Vec2 = vec2(0.4, 0.5);
const STOPPING_SPEED: f64 = 0.25;

lazy_static! {
    static ref FRAME_SETS: [CharacterFrameSet; 10] = [
        CharacterFrameSet::from_texture_set(Vec2::right(), texture::player_right()),
        CharacterFrameSet::from_texture_set(Vec2::left(), texture::player_left()),
        CharacterFrameSet::from_texture_set(Vec2::top_right(), texture::player_top_right()),
        CharacterFrameSet::from_texture_set(Vec2::top_left(), texture::player_top_left()),
        CharacterFrameSet::from_texture_set(Vec2::bottom_right(), texture::player_bottom_right()),
        CharacterFrameSet::from_texture_set(Vec2::bottom_left(), texture::player_bottom_left()),
        CharacterFrameSet::from_texture(Vec2::top_right() / 10.0, texture::player_top_right_idle()),
        CharacterFrameSet::from_texture(Vec2::top_left() / 10.0, texture::player_top_left_idle()),
        CharacterFrameSet::from_texture(
            Vec2::bottom_right() / 10.0,
            texture::player_bottom_right_idle(),
        ),
        CharacterFrameSet::from_texture(
            Vec2::bottom_left() / 10.0,
            texture::player_bottom_left_idle(),
        ),
    ];
}

pub fn create(world: &mut World, focus: Vec2) -> Entity {
    world
        .create_entity()
        .with(Health::full(HEALTH))
        .with(HealthRegen::new())
        .with(Bounds(Rect::focused(focus, SIZE)))
        .with(Facing(Vec2::zero()))
        .with(CharacterAnimation::new_sized(
            &*FRAME_SETS,
            0.5,
            vec2(0.5, 0.5),
        ))
        .with(Physics::collider(DRAG))
        .with(Collider)
        .with(Player::new())
        .build()
}

pub fn handle_input(world: &mut World, mouse_position: Vec2) {
    update_player_inputs(world, mouse_position);

    let mut inputs = world.fetch_mut::<Inputs>();

    for event in inputs.poll() {
        match event {
            Event::KeyDown { key } if key == "Space" => {
                Interaction::attempt_interact(world);
            }
            Event::KeyDown { key } => {
                if let Some(slot) = parse_digit_key(&key) {
                    select_slot(world, slot);
                }
            }
            _ => {}
        }
    }

    let mouse_down = inputs.is_mouse_down();
    mem::drop(inputs);

    move_player(world);
    face_player(world);

    if mouse_down {
        attack(world);
    }
}

fn select_slot(world: &World, slot: usize) {
    if let Some(mut player) = world.controlled_player_write::<Player>() {
        player.select_gun(slot);
    }
}

fn parse_digit_key(key: &str) -> Option<usize> {
    key.trim_start_matches("Digit").parse().ok().map(
        |digit: usize| {
            if digit == 0 {
                9
            } else {
                digit - 1
            }
        },
    )
}

fn update_player_inputs(world: &World, mouse_position: Vec2) {
    if let Some(mut player) = world.controlled_player_write::<Player>() {
        let inputs = world.fetch::<Inputs>();
        player.update_inputs(&inputs, mouse_position);
    }
}

fn face_player(world: &World) {
    let player = match world.controlled_player() {
        Some(player) => player,
        None => return,
    };

    let physics = world.unwrap_read::<Physics>(player);
    let facing = &mut world.unwrap_write::<Facing>(player).0;

    let mut direction = if physics.velocity().length_squared() < STOPPING_SPEED.powi(2) {
        Vec2::zero()
    } else {
        physics.velocity().normalize_or_zero()
    };

    let inputs = world.fetch::<Inputs>();

    if inputs.is_mouse_down() {
        if inputs.mouse().x() * direction.x() < 0.0 {
            direction *= vec2(-1.0, 1.0);
        }

        if direction.x() == 0.0 {
            *direction.x_mut() += inputs.mouse().x().signum() * 0.1;
        }
    }

    *facing = direction;
}

fn move_player(world: &World) {
    if let Some(mut physics) = world.controlled_player_write::<Physics>() {
        let inputs = world.fetch::<Inputs>();
        let player_direction = inputs.movement_direction();
        physics.accelerate_to(player_direction * SPEED);
    }
}

fn attack(world: &mut World) {
    let player = match world.controlled_player() {
        Some(player) => player,
        None => return,
    };

    if let Some(attack) = AttackParameters::calculate(world, player) {
        world
            .unwrap_read::<Player>(player)
            .selected_gun()
            .play_sound();

        for _ in 0..attack.bullet_count {
            let variation = Random::global().next_f64_in(-1.0..1.0);
            let inaccuracy_angle = variation * attack.inaccuracy * PI / 2.0;
            let direction = attack.direction.rotate(inaccuracy_angle);

            bullet::create(
                world,
                player,
                attack.damage,
                attack.knockback,
                attack.penetration,
                attack.position,
                direction,
            );
        }
    }
}

struct AttackParameters {
    pub position: Vec2,
    pub direction: Vec2,
    pub damage: f64,
    pub knockback: f64,
    pub penetration: f64,
    pub inaccuracy: f64,
    pub bullet_count: usize,
}

impl AttackParameters {
    pub fn calculate(world: &mut World, player_entity: Entity) -> Option<Self> {
        let target = player_targeting_position(world, player_entity);
        let source = weapon_nose_position(world, player_entity);

        let direction = (target - source).normalize_or_zero();
        if direction.is_zero() {
            return None;
        }

        let mut player = world.unwrap_write::<Player>(player_entity);

        match player.try_attack() {
            AttackResult::Can {
                damage,
                knockback,
                penetration,
                inaccuracy,
                bullet_count,
            } => Some(AttackParameters {
                position: source,
                direction,
                damage,
                knockback,
                penetration,
                inaccuracy,
                bullet_count,
            }),
            AttackResult::Cant => None,
        }
    }
}
