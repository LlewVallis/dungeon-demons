use crate::components::bounds::Bounds;
use crate::components::health::Health;
use crate::components::player::Player;
use crate::progression::{Progression, SPAWNER_SEARCH_SIZE};
use crate::util::vector::Vec2;
use specs::prelude::*;
use specs::{Component, Join, ReadStorage, System, WriteStorage};

#[derive(Default)]
pub struct Enemy;

impl Component for Enemy {
    type Storage = DenseVecStorage<Self>;
}

pub struct KillLostEnemies;

#[derive(SystemData)]
pub struct KillLostEnemiesData<'a> {
    progression: WriteExpect<'a, Progression>,
    bounds: ReadStorage<'a, Bounds>,
    healths: WriteStorage<'a, Health>,
    enemies: ReadStorage<'a, Enemy>,
    players: ReadStorage<'a, Player>,
}

impl<'a> System<'a> for KillLostEnemies {
    type SystemData = KillLostEnemiesData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut player_positions = Vec::new();
        for (bounds, _) in (&data.bounds, &data.players).join() {
            player_positions.push(bounds.0.center());
        }

        if player_positions.is_empty() {
            return;
        }

        let iter = (&data.bounds, &mut data.healths, &data.enemies).join();
        'enemyLoop: for (bounds, health, _) in iter {
            for player in &player_positions {
                let distance = Vec2::distance_squared(*player, bounds.0.center());
                if distance < SPAWNER_SEARCH_SIZE.powi(2) {
                    continue 'enemyLoop;
                }
            }

            data.progression.report_enemy_death();
            health.kill();
        }
    }
}
