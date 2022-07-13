use crate::audio::{play_sound, sound};
use specs::prelude::*;
use specs::{ReadStorage, System};

use crate::components::bounds::Bounds;
use crate::components::health::Health;
use crate::components::player::Player;
use crate::game::Delta;

const REACH: f64 = 0.2;

pub struct MeleeAttacker {
    damage: f64,
    cooldown: f64,
    delay: f64,
    remaining_cooldown: f64,
}

impl MeleeAttacker {
    pub fn new(damage: f64, cooldown: f64, delay: f64) -> Self {
        Self {
            damage,
            cooldown,
            delay,
            remaining_cooldown: cooldown,
        }
    }
}

impl Component for MeleeAttacker {
    type Storage = DenseVecStorage<Self>;
}

pub struct AttackPlayers;

#[derive(SystemData)]
pub struct MonsterAttackingData<'a> {
    delta: ReadExpect<'a, Delta>,
    bounds: ReadStorage<'a, Bounds>,
    healths: WriteStorage<'a, Health>,
    attackers: WriteStorage<'a, MeleeAttacker>,
    players: ReadStorage<'a, Player>,
}

impl<'a> System<'a> for AttackPlayers {
    type SystemData = MonsterAttackingData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (bounds, attacker) in (&data.bounds, &mut data.attackers).join() {
            if attacker.remaining_cooldown > 0.0 {
                attacker.remaining_cooldown -= data.delta.0;
                continue;
            }

            let players_iter = (&data.bounds, &mut data.healths, &data.players).join();
            for (player_bounds, player_health, _) in players_iter {
                if bounds.0.euclidean_distance(player_bounds.0) <= REACH {
                    player_health.damage(attacker.damage);

                    play_sound(sound::player_hit());
                    if player_health.is_dead() {
                        play_sound(sound::death());
                    }

                    attacker.remaining_cooldown += attacker.cooldown;
                } else {
                    if attacker.remaining_cooldown < attacker.delay {
                        attacker.remaining_cooldown += data.delta.0;
                    } else {
                        attacker.remaining_cooldown -= data.delta.0;
                    }
                }
            }
        }
    }
}
