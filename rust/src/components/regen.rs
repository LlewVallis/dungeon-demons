use crate::components::health::Health;
use crate::game::{Delta, Timestamp};
use specs::prelude::*;
use specs::{Component, HashMapStorage, ReadExpect, System, WriteStorage};

const HEAL_DELAY: f64 = 5.0;
const HEAL_DURATION: f64 = 10.0;

pub struct HealthRegen {
    last_health: f64,
    last_damage_time: f64,
}

impl HealthRegen {
    pub fn new() -> Self {
        Self {
            last_health: 0.0,
            last_damage_time: 0.0,
        }
    }
}

impl Component for HealthRegen {
    type Storage = HashMapStorage<HealthRegen>;
}

pub struct RegenerateHealth;

#[derive(SystemData)]
pub struct RegenerateHealthData<'a> {
    timestamp: ReadExpect<'a, Timestamp>,
    delta: ReadExpect<'a, Delta>,
    healths: WriteStorage<'a, Health>,
    regens: WriteStorage<'a, HealthRegen>,
}

impl<'a> System<'a> for RegenerateHealth {
    type SystemData = RegenerateHealthData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for (health, regen) in (&mut data.healths, &mut data.regens).join() {
            if health.remaining_absolute() < regen.last_health {
                regen.last_damage_time = data.timestamp.0;
            }

            regen.last_health = health.remaining_absolute();

            if regen.last_damage_time + HEAL_DELAY > data.timestamp.0 {
                continue;
            }

            health.heal(health.maximum() * data.delta.0 / HEAL_DURATION);
        }
    }
}
