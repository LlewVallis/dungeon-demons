use specs::{Component, Entities, Join, ReadStorage, System, VecStorage};

#[derive(Copy, Clone)]
pub struct Health {
    remaining: f64,
    max: f64,
}

impl Health {
    pub fn new(remaining: f64, max: f64) -> Self {
        Self { remaining, max }
    }

    pub fn full(amount: f64) -> Self {
        Self::new(amount, amount)
    }

    pub fn damage(&mut self, amount: f64) {
        self.remaining -= amount;
        self.remaining = self.remaining.clamp(0.0, self.max);
    }

    pub fn heal(&mut self, amount: f64) {
        self.damage(-amount);
    }

    pub fn kill(&mut self) {
        self.remaining = 0.0;
    }

    pub fn remaining_relative(&self) -> f64 {
        self.remaining / self.max
    }

    pub fn remaining_absolute(&self) -> f64 {
        self.remaining
    }

    pub fn maximum(&self) -> f64 {
        self.max
    }

    pub fn is_dead(&self) -> bool {
        self.remaining == 0.0
    }
}

impl Component for Health {
    type Storage = VecStorage<Health>;
}

pub struct DeleteDeadEntities;

impl<'a> System<'a> for DeleteDeadEntities {
    type SystemData = (Entities<'a>, ReadStorage<'a, Health>);

    fn run(&mut self, (entities, healths): Self::SystemData) {
        for (entity, health) in (&entities, &healths).join() {
            if health.is_dead() {
                let _ = entities.delete(entity);
            }
        }
    }
}
