use float_ord::FloatOrd;
use specs::prelude::*;
use specs::{ReadExpect, System, WriteExpect};

use crate::audio::{play_sound, sound};
use crate::components::bounds::Bounds;
use crate::components::player::Player;
use crate::entities::demon;
use crate::game::Delta;
use crate::map::Map;
use crate::util::random::Random;
use crate::util::rect::Rect;
use crate::util::vector::Vec2;
use crate::vec2;

pub const SPAWNER_SEARCH_SIZE: f64 = 50.0;

const SPAWNER_PATHFIND_THRESHOLD: f64 = 75.0;

const IDEAL_SPAWN_DISTANCE: f64 = 8.0;

const DEFAULT_SPAWN_LOCATION: Vec2 = vec2(4.5, 0.5);

pub struct Progression {
    round: usize,
    remaining_enemies: usize,
    spawned_enemies: usize,
    active_enemies: usize,
    time_until_spawn: f64,
}

impl Progression {
    pub fn new() -> Self {
        Self {
            round: 0,
            remaining_enemies: Self::enemies_of_round(0),
            spawned_enemies: 0,
            active_enemies: 0,
            time_until_spawn: 0.0,
        }
    }

    pub fn report_enemy_death(&mut self) {
        self.spawned_enemies -= 1;
        self.active_enemies -= 1;
    }

    pub fn report_enemy_killed(&mut self) {
        self.remaining_enemies -= 1;
        self.active_enemies -= 1;

        if self.remaining_enemies == 0 {
            self.round += 1;
            self.remaining_enemies = self.total_enemies();
            self.spawned_enemies = 0;
            self.active_enemies = 0;
            self.time_until_spawn = self.relief_time();

            log::info!("Progressed to round {}", self.round);
            play_sound(sound::round_end());
        }
    }

    fn total_enemies(&self) -> usize {
        Self::enemies_of_round(self.round)
    }

    fn concurrent_enemies(&self) -> usize {
        ((self.total_enemies() as f64).sqrt() * 2.0).round() as usize
    }

    fn enemies_of_round(round: usize) -> usize {
        let round = round as f64;
        let enemies = 0.00005 * round.powi(3) + 0.05 * round.powi(2) + 1.5 * round + 5.0;
        enemies.round() as usize
    }

    pub fn round(&self) -> usize {
        self.round
    }

    fn relief_time(&self) -> f64 {
        (2.0 * self.round as f64 + 5.0).min(45.0)
    }

    fn spawn_delay(&self) -> f64 {
        let start_bias = 2.0 / (self.round.pow(2) as f64 / 25.0 + 1.0);
        (self.round as f64 + 5.0) / self.total_enemies() as f64 + start_bias
    }

    fn enemy_health(&self) -> f64 {
        let base = self.round as f64 * 25.0 + 25.0;
        let multiplier = 1.025f64.powi(self.round as i32);
        base * multiplier
    }

    fn enemy_speed(&self) -> f64 {
        let max = 3.0;
        let min = 1.0;
        max - (max - min) / (self.round as f64 / 2.5 + 1.0)
    }
}

pub struct SpawnEnemies;

impl SpawnEnemies {
    fn find_spawn(map: &Map, player_positions: &[Vec2]) -> Vec2 {
        if player_positions.is_empty() {
            return DEFAULT_SPAWN_LOCATION;
        }

        let player = *Random::global().element(&player_positions);

        let candidates = Self::find_spawns(map, player, &player_positions);
        if candidates.is_empty() {
            return DEFAULT_SPAWN_LOCATION;
        }

        let candidate_index = Random::global()
            .weighted_index(|| candidates.iter().map(|(_, weight)| weight).copied());

        candidates[candidate_index].0
    }

    fn find_spawns(
        map: &Map,
        player_position: Vec2,
        player_positions: &[Vec2],
    ) -> Vec<(Vec2, f64)> {
        let search = Rect::focused(player_position, Vec2::one() * SPAWNER_SEARCH_SIZE);

        let player_coords = player_positions
            .iter()
            .map(|position| position.coord())
            .collect::<Vec<_>>();

        map.spawners_in(search)
            .filter(|spawn| {
                let targets = || player_coords.iter().copied();
                map.pathfind(spawn.coord(), targets, SPAWNER_PATHFIND_THRESHOLD)
                    .is_some()
            })
            .map(|spawn| {
                let closest = player_positions
                    .iter()
                    .map(|player| Vec2::distance(*player, spawn))
                    .min_by_key(|distance| FloatOrd(*distance))
                    .unwrap();

                let target_distance = Vec2::distance(player_position, spawn);
                let ideal_deviation = (target_distance - IDEAL_SPAWN_DISTANCE).abs();

                let ideal_deviation_penalty = ideal_deviation / 10.0;
                let closeness_penalty = 0.75 / closest.powi(2);
                let score = (5.0 - ideal_deviation_penalty - closeness_penalty).max(0.0).cbrt();

                (spawn, score)
            })
            .collect::<Vec<_>>()
    }
}

#[derive(SystemData)]
pub struct SpawnEnemiesData<'a> {
    lazy_update: ReadExpect<'a, LazyUpdate>,
    map: ReadExpect<'a, Map>,
    delta: ReadExpect<'a, Delta>,
    progression: WriteExpect<'a, Progression>,
    bounds: ReadStorage<'a, Bounds>,
    players: ReadStorage<'a, Player>,
}

impl<'a> System<'a> for SpawnEnemies {
    type SystemData = SpawnEnemiesData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let progression = &mut data.progression;

        if progression.active_enemies >= progression.concurrent_enemies() {
            return;
        }

        if progression.spawned_enemies >= progression.total_enemies() {
            return;
        }

        progression.time_until_spawn -= data.delta.0;

        if progression.time_until_spawn > 0.0 {
            return;
        }

        if progression.spawned_enemies == 0 {
            play_sound(sound::round_start());
        }

        progression.time_until_spawn += progression.spawn_delay();
        progression.spawned_enemies += 1;
        progression.active_enemies += 1;

        let player_positions = (&data.bounds, &data.players)
            .join()
            .map(|(bounds, _)| bounds.0.center())
            .collect::<Vec<_>>();

        let spawn = Self::find_spawn(&data.map, &player_positions);

        let health = progression.enemy_health();
        let speed = progression.enemy_speed();

        data.lazy_update.exec_mut(move |world| {
            demon::create_spawning(world, spawn, health, speed);
        });
    }
}
