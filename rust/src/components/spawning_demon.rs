use specs::prelude::*;
use specs::{Component, Entities, LazyUpdate, ReadExpect, ReadStorage, System};

use crate::components::bounds::Bounds;
use crate::components::health::Health;
use crate::entities::demon;
use crate::game::Delta;

pub struct SpawningDemon {
    remaining_time: f64,
    speed: f64,
}

impl SpawningDemon {
    pub fn new(time: f64, speed: f64) -> Self {
        Self {
            speed,
            remaining_time: time,
        }
    }
}

impl Component for SpawningDemon {
    type Storage = HashMapStorage<Self>;
}

pub struct FinishDemonSpawning;

#[derive(SystemData)]
pub struct FinishDemonSpawningData<'a> {
    entities: Entities<'a>,
    lazy_update: ReadExpect<'a, LazyUpdate>,
    delta: ReadExpect<'a, Delta>,
    spawning_demons: WriteStorage<'a, SpawningDemon>,
    bounds: ReadStorage<'a, Bounds>,
    healths: ReadStorage<'a, Health>,
}

impl<'a> System<'a> for FinishDemonSpawning {
    type SystemData = FinishDemonSpawningData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let iter = (
            &data.entities,
            &mut data.spawning_demons,
            &data.bounds,
            &data.healths,
        )
            .join();

        for (entity, spawning_demon, bounds, health) in iter {
            spawning_demon.remaining_time -= data.delta.0;

            if spawning_demon.remaining_time <= 0.0 {
                let center = bounds.0.center();
                let health = *health;
                let speed = spawning_demon.speed;

                data.lazy_update.exec_mut(move |world| {
                    let _ = world.delete_entity(entity);
                    demon::create(world, center, health, speed);
                });
            }
        }
    }
}
