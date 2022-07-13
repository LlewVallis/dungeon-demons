use smallvec::SmallVec;
use smolset::SmolSet;
use specs::prelude::*;
use specs::{Component, ReadExpect, System, VecStorage, World, WriteStorage};

use crate::components::bounds::Bounds;
use crate::game::Delta;
use crate::map::Map;
use crate::util::coord::Coord;
use crate::util::intersection_grid::IntersectionGrid;
use crate::util::rect::Rect;
use crate::util::vector::Vec2;

const STOPPING_SPEED: f64 = 0.05;
const COLLISION_BOUNCE: f64 = 0.0005;
const COLLISION_IGNORE_DISTANCE: f64 = 0.001;

const MAX_MOVE_DISTANCE: f64 = 0.1;
const MAX_MOVE_STEPS: usize = 15;

const MAX_DELTA: f64 = 0.001;

pub struct Physics {
    velocity: Vec2,
    tick_acceleration: Vec2,
    drag: f64,
    collision_report: CollisionReport,
    respond: bool,
}

impl Physics {
    pub fn collider(drag: f64) -> Self {
        Self {
            drag,
            collision_report: CollisionReport::ignoring(),
            velocity: Vec2::zero(),
            tick_acceleration: Vec2::zero(),
            respond: true,
        }
    }

    pub fn trigger(drag: f64) -> Self {
        Self {
            drag,
            collision_report: CollisionReport::reporting(),
            velocity: Vec2::zero(),
            tick_acceleration: Vec2::zero(),
            respond: false,
        }
    }

    pub fn accelerate(&mut self, amount: Vec2) {
        self.tick_acceleration += amount;
    }

    pub fn accelerate_to(&mut self, target_velocity: Vec2) {
        self.accelerate(target_velocity * self.drag)
    }

    pub fn velocity(&self) -> Vec2 {
        self.velocity
    }

    pub fn velocity_mut(&mut self) -> &mut Vec2 {
        &mut self.velocity
    }

    pub fn collisions(&self) -> &CollisionReport {
        &self.collision_report
    }
}

impl Component for Physics {
    type Storage = VecStorage<Self>;
}

pub struct CollisionReport {
    inner: CollisionReportInner,
}

enum CollisionReportInner {
    Ignoring,
    Reporting {
        walls: bool,
        entities: SmolSet<[Entity; 3]>,
    },
}

impl CollisionReport {
    fn reporting() -> Self {
        Self {
            inner: CollisionReportInner::Reporting {
                walls: false,
                entities: SmolSet::new(),
            },
        }
    }

    fn ignoring() -> Self {
        Self {
            inner: CollisionReportInner::Ignoring,
        }
    }

    fn clear(&mut self) {
        *self = match &self.inner {
            CollisionReportInner::Ignoring => Self::ignoring(),
            CollisionReportInner::Reporting { .. } => Self::reporting(),
        };
    }

    fn on_hit_entity(&mut self, entity: Entity) {
        match &mut self.inner {
            CollisionReportInner::Ignoring => {}
            CollisionReportInner::Reporting { entities, .. } => {
                entities.insert(entity);
            }
        }
    }

    fn on_hit_wall(&mut self) {
        match &mut self.inner {
            CollisionReportInner::Ignoring => {}
            CollisionReportInner::Reporting { walls, .. } => {
                *walls = true;
            }
        }
    }

    pub fn hit_entities(&self) -> Option<impl Iterator<Item = Entity> + '_> {
        match &self.inner {
            CollisionReportInner::Reporting { entities, .. } => Some(entities.iter().copied()),
            CollisionReportInner::Ignoring => None,
        }
    }

    pub fn hit_wall(&self) -> Option<bool> {
        match &self.inner {
            CollisionReportInner::Reporting { walls, .. } => Some(*walls),
            CollisionReportInner::Ignoring => None,
        }
    }
}

#[derive(Default)]
pub struct Collider;

impl Component for Collider {
    type Storage = NullStorage<Collider>;
}

type Colliders = IntersectionGrid<Entity, ()>;

pub struct SimulatePhysics;

impl SimulatePhysics {
    fn create_colliders(data: &mut MoveData) -> Colliders {
        let mut grid = IntersectionGrid::new();

        let iter = (&data.entities, &data.bounds, &data.colliders).join();
        for (entity, bounds, _) in iter {
            grid.insert(bounds.0, entity, ());
        }

        grid
    }

    fn move_entity(
        amount: Vec2,
        entity: Entity,
        bounds: &mut Rect,
        report: &mut CollisionReport,
        respond: bool,
        map: &Map,
        colliders: &Colliders,
        delta: f64,
    ) {
        let steps = (amount.length() / MAX_MOVE_DISTANCE).ceil() as usize;
        let steps = steps.clamp(1, MAX_MOVE_STEPS);

        for _ in 0..steps {
            let amount = amount / steps as f64;
            let delta = delta / steps as f64;

            Self::move_axis(
                0, amount, entity, bounds, report, respond, map, colliders, delta,
            );
            Self::move_axis(
                1, amount, entity, bounds, report, respond, map, colliders, delta,
            );
        }
    }

    fn move_axis(
        axis: usize,
        amount: Vec2,
        entity: Entity,
        bounds: &mut Rect,
        report: &mut CollisionReport,
        respond: bool,
        map: &Map,
        colliders: &Colliders,
        delta: f64,
    ) {
        bounds.position[axis] += amount[axis];
        let collisions = Self::find_collisions(entity, *bounds, report, map, colliders);

        if respond {
            Self::resolve_axis(axis, bounds, collisions, delta)
        }
    }

    fn resolve_axis(
        axis: usize,
        bounds: &mut Rect,
        collisions: SmallVec<[(Rect, bool); 4]>,
        delta: f64,
    ) {
        let mut axis_vector = Vec2::zero();
        axis_vector[axis] = 1.0;

        let start_bounds = *bounds;
        for (collider, wall) in collisions {
            let distance = (start_bounds.center()[axis] - collider.center()[axis]).abs();
            if distance < COLLISION_IGNORE_DISTANCE {
                continue;
            }

            let normal = collider.normal_to(start_bounds);

            let limit = if normal == axis_vector {
                collider.max()[axis] + COLLISION_BOUNCE
            } else if normal == -axis_vector {
                collider.min()[axis] - bounds.size[axis] - COLLISION_BOUNCE
            } else {
                continue;
            };

            let mut push = limit - bounds.position[axis];

            if !wall {
                let expected_distance = (start_bounds.size[axis] + collider.size[axis]) / 2.0;
                let penetration = 1.0 - distance / expected_distance;
                let max_push = penetration * 4.0 + 1.0;

                push = push.clamp(-max_push * delta, max_push * delta);
            }

            bounds.position[axis] += push;
        }
    }

    fn find_collisions(
        entity: Entity,
        bounds: Rect,
        report: &mut CollisionReport,
        map: &Map,
        colliders: &Colliders,
    ) -> SmallVec<[(Rect, bool); 4]> {
        let mut results = SmallVec::new();

        for (collider_entity, (collider, _)) in colliders.query(bounds) {
            if *collider_entity != entity {
                results.push((collider, false));
                report.on_hit_entity(*collider_entity);
            }
        }

        let min_coord = bounds.min().coord();
        let max_coord = bounds.max().coord();

        for coord in Coord::between_inclusive(min_coord, max_coord) {
            if !map.at(coord).is_walkable() {
                results.push((coord.bounds(), true));
                report.on_hit_wall();
            }
        }

        results
    }
}

#[derive(SystemData)]
pub struct MoveData<'a> {
    entities: Entities<'a>,
    map: ReadExpect<'a, Map>,
    delta: ReadExpect<'a, Delta>,
    bounds: WriteStorage<'a, Bounds>,
    physicses: WriteStorage<'a, Physics>,
    colliders: ReadStorage<'a, Collider>,
}

impl<'a> System<'a> for SimulatePhysics {
    type SystemData = MoveData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let delta = data.delta.0;
        let steps = (delta / MAX_DELTA).ceil() as usize;

        let iter = (&mut data.physicses, &data.bounds).join();
        for (physics, _) in iter {
            physics.velocity += physics.tick_acceleration * delta;
            physics.collision_report.clear();
        }

        for _ in 0..steps {
            let delta = delta / steps as f64;

            let colliders = Self::create_colliders(&mut data);

            let iter = (&data.entities, &mut data.bounds, &mut data.physicses).join();
            for (entity, bounds, physics) in iter {
                let amount = physics.velocity * delta;

                Self::move_entity(
                    amount,
                    entity,
                    &mut bounds.0,
                    &mut physics.collision_report,
                    physics.respond,
                    &data.map,
                    &colliders,
                    delta,
                );
            }
        }

        let iter = (&mut data.physicses, &data.bounds).join();
        for (physics, _) in iter {
            physics.velocity -= physics.velocity * physics.drag * delta;

            if physics.tick_acceleration.is_zero()
                && physics.velocity.length_squared() < STOPPING_SPEED.powi(2)
            {
                physics.velocity = Vec2::zero();
            }

            physics.tick_acceleration = Vec2::zero();
        }
    }
}
