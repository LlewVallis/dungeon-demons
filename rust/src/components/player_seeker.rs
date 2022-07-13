use crate::components::bounds::Bounds;
use crate::components::physics::Physics;
use crate::components::player::Player;
use crate::components::sprite::character_animation::Facing;
use crate::game::Timestamp;
use float_ord::FloatOrd;
use specs::prelude::*;
use specs::{Component, World};
use specs::{Join, System, WriteStorage};

use crate::map::Map;
use crate::util::coord::Coord;
use crate::util::random::Random;
use crate::util::rect::Rect;
use crate::util::vector::Vec2;

const TARGET_REACHED_DISTANCE: f64 = 0.2;

pub struct PlayerSeeker {
    target: Option<Vec2>,
    target_invalid_timestamp: f64,
    speed: f64,
    player_targeting_offset: Vec2,
}

impl PlayerSeeker {
    pub fn new(speed: f64) -> Self {
        let player_targeting_offset = (Random::global().next_vec2() - 0.5) * 0.15;

        Self {
            speed,
            player_targeting_offset,
            target: None,
            target_invalid_timestamp: 0.0,
        }
    }

    fn invalidate_target(&mut self) {
        self.target = None;
        self.target_invalid_timestamp = 0.0;
    }
}

impl Component for PlayerSeeker {
    type Storage = DenseVecStorage<Self>;
}

pub struct SeekPlayers;

impl SeekPlayers {
    fn seek_direction(
        map: &Map,
        timestamp: f64,
        seeker: &mut PlayerSeeker,
        bounds: &mut Rect,
        player_bounds: &[Rect],
    ) -> Vec2 {
        let closest = player_bounds.iter().min_by_key(|player_bounds| {
            let dist = player_bounds.euclidean_distance(*bounds);
            FloatOrd(dist)
        });

        let closest = match closest {
            Some(bounds) => bounds,
            None => return Vec2::zero(),
        };

        let closest_target = closest.center() + seeker.player_targeting_offset;

        let center = bounds.center();
        let coord = center.coord();

        if center.distance(closest_target) < 0.75 {
            seeker.invalidate_target();
            (closest_target - center).normalize_or_zero()
        } else {
            if let Some(target_position) = seeker.target {
                if center.distance_squared(target_position) < TARGET_REACHED_DISTANCE.powi(2) {
                    seeker.invalidate_target();
                }
            }

            if seeker.target_invalid_timestamp <= timestamp {
                seeker.target = Self::update_target(map, coord, player_bounds);
                seeker.target_invalid_timestamp = timestamp + Self::repath_delay();
            }

            if let Some(target) = seeker.target {
                (target - center).normalize_or_zero()
            } else {
                Vec2::zero()
            }
        }
    }

    fn repath_delay() -> f64 {
        Random::global().next_f64_in(0.25..0.5)
    }

    fn update_target(map: &Map, coord: Coord, player_bounds: &[Rect]) -> Option<Vec2> {
        let offset = (Random::global().next_vec2() - 0.5) * 0.4;

        if let Some(next_step) = Self::next_step(map, coord, player_bounds) {
            Some(next_step.center() + offset)
        } else {
            None
        }
    }

    fn next_step(map: &Map, coord: Coord, player_bounds: &[Rect]) -> Option<Coord> {
        let targets = || player_bounds.iter().map(|bounds| bounds.center().coord());

        let path = map.pathfind(coord, targets, 100.0);
        path.and_then(|(steps, _)| steps.get(1).or(steps.get(0)).copied())
    }
}

#[derive(SystemData)]
pub struct SeekPlayersData<'a> {
    map: ReadExpect<'a, Map>,
    timestamp: ReadExpect<'a, Timestamp>,
    players: ReadStorage<'a, Player>,
    bounds: WriteStorage<'a, Bounds>,
    physicses: WriteStorage<'a, Physics>,
    facings: WriteStorage<'a, Facing>,
    seekers: WriteStorage<'a, PlayerSeeker>,
}

impl<'a> System<'a> for SeekPlayers {
    type SystemData = SeekPlayersData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let mut player_bounds = Vec::new();
        for (bounds, _) in (&data.bounds, &data.players).join() {
            player_bounds.push(bounds.0);
        }

        let iter = (
            &mut data.bounds,
            &mut data.physicses,
            (&mut data.facings).maybe(),
            &mut data.seekers,
        )
            .join();

        for (bounds, physics, facing, seeker) in iter {
            let direction = Self::seek_direction(
                &data.map,
                data.timestamp.0,
                seeker,
                &mut bounds.0,
                &player_bounds,
            );

            physics.accelerate_to(direction * seeker.speed);

            if let Some(facing) = facing {
                facing.0 = direction;
            }
        }
    }
}
