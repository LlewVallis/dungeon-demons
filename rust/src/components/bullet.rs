use crate::audio::{play_sound, sound};
use crate::components::health::Health;
use crate::components::physics::Physics;
use crate::components::player::Player;
use crate::progression::Progression;
use crate::util::vector::Vec2;
use specs::prelude::*;
use specs::{Component, NullStorage, ReadStorage, System, WriteStorage};

const KILL_CREDITS: usize = 100;
const HIT_CREDITS: usize = 10;

#[derive(Copy, Clone)]
pub struct Bullet {
    owner: Entity,
    damage: f64,
    knockback: f64,
}

impl Bullet {
    pub fn new(owner: Entity, damage: f64, knockback: f64) -> Self {
        Self {
            owner,
            damage,
            knockback,
        }
    }
}

impl Component for Bullet {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct BulletTarget;

impl Component for BulletTarget {
    type Storage = NullStorage<Self>;
}

pub struct UpdateBullets;

impl UpdateBullets {
    fn damage(
        bullet: Bullet,
        target_health: &mut Health,
        progression: &mut Progression,
        players: &mut WriteStorage<Player>,
    ) {
        target_health.damage(bullet.damage);

        if target_health.is_dead() {
            play_sound(sound::kill());
            progression.report_enemy_killed();
            Self::give_credits(bullet.owner, KILL_CREDITS, players);
        } else {
            Self::give_credits(bullet.owner, HIT_CREDITS, players);
        }
    }

    fn give_credits(player: Entity, amount: usize, players: &mut WriteStorage<Player>) {
        if let Some(player) = players.get_mut(player) {
            *player.credits_mut() += amount;
        }
    }

    fn knockback(bullet: Bullet, bullet_velocity: Vec2, target_physics: &mut Physics) {
        let knockback = bullet_velocity.normalize_or_zero() * bullet.knockback;
        *target_physics.velocity_mut() += knockback;
    }
}

#[derive(SystemData)]
pub struct UpdateBulletsData<'a> {
    entities: Entities<'a>,
    progression: WriteExpect<'a, Progression>,
    healths: WriteStorage<'a, Health>,
    physicses: WriteStorage<'a, Physics>,
    bullets: ReadStorage<'a, Bullet>,
    targets: ReadStorage<'a, BulletTarget>,
    players: WriteStorage<'a, Player>,
}

impl<'a> System<'a> for UpdateBullets {
    type SystemData = UpdateBulletsData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut hits = Vec::new();

        let iter = (
            &data.entities,
            &data.physicses,
            &mut data.healths,
            &data.bullets,
        )
            .join();
        for (bullet_entity, bullet_physics, bullet_health, _) in iter {
            if bullet_physics.collisions().hit_wall().unwrap() {
                bullet_health.kill();
                continue;
            }

            for entity in bullet_physics.collisions().hit_entities().unwrap() {
                if data.targets.contains(entity) {
                    hits.push((bullet_entity, entity));
                    break;
                }
            }
        }

        for (bullet_entity, target_entity) in hits {
            if let Some(health) = data.healths.get(bullet_entity) {
                if health.is_dead() {
                    continue;
                }
            }

            if let Some(health) = data.healths.get(target_entity) {
                if health.is_dead() {
                    continue;
                }
            }

            play_sound(sound::hit());

            let bullet = *data.bullets.get(bullet_entity).unwrap();
            let velocity = data.physicses.get(bullet_entity).unwrap().velocity();

            if let Some(physics) = data.physicses.get_mut(target_entity) {
                Self::knockback(bullet, velocity, physics);
            }

            let mut dealt_damage = bullet.damage;

            if let Some(health) = data.healths.get_mut(target_entity) {
                dealt_damage = dealt_damage.min(health.remaining_absolute());
                Self::damage(bullet, health, &mut data.progression, &mut data.players);
            }

            if let Some(health) = data.healths.get_mut(bullet_entity) {
                health.damage(dealt_damage * 1.01);
            }
        }
    }
}
