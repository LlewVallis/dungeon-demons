use crate::game::{ControlledPlayer, Delta, Timestamp};
use specs::prelude::*;
use specs::{Component, Entity, ReadStorage, World, WorldExt, WriteStorage};
use std::ops::{Deref, DerefMut};

pub trait WorldExtensions {
    fn timestamp(&self) -> f64;

    fn delta(&self) -> f64;

    fn controlled_player(&self) -> Option<Entity>;

    fn unwrap_read<T: Component>(&self, entity: Entity) -> UnwrapRead<T>;

    fn unwrap_write<T: Component>(&self, entity: Entity) -> UnwrapWrite<T>;

    fn controlled_player_read<T: Component>(&self) -> Option<UnwrapRead<T>> {
        self.controlled_player()
            .map(|player| self.unwrap_read::<T>(player))
    }

    fn controlled_player_write<T: Component>(&self) -> Option<UnwrapWrite<T>> {
        self.controlled_player()
            .map(|player| self.unwrap_write::<T>(player))
    }
}

pub struct UnwrapRead<'a, T: Component> {
    storage: ReadStorage<'a, T>,
    entity: Entity,
}

impl<'a, T: Component> Deref for UnwrapRead<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.storage.get(self.entity).unwrap()
    }
}

pub struct UnwrapWrite<'a, T: Component> {
    storage: WriteStorage<'a, T>,
    entity: Entity,
}

impl<'a, T: Component> Deref for UnwrapWrite<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.storage.get(self.entity).unwrap()
    }
}

impl<'a, T: Component> DerefMut for UnwrapWrite<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.storage.get_mut(self.entity).unwrap()
    }
}

impl WorldExtensions for World {
    fn timestamp(&self) -> f64 {
        self.fetch::<Timestamp>().0
    }

    fn delta(&self) -> f64 {
        self.fetch::<Delta>().0
    }

    fn controlled_player(&self) -> Option<Entity> {
        self.try_fetch::<ControlledPlayer>().map(|player| player.0)
    }

    fn unwrap_read<T: Component>(&self, entity: Entity) -> UnwrapRead<T> {
        UnwrapRead {
            storage: self.read_component(),
            entity,
        }
    }

    fn unwrap_write<T: Component>(&self, entity: Entity) -> UnwrapWrite<T> {
        UnwrapWrite {
            storage: self.write_component(),
            entity,
        }
    }
}

#[derive(SystemData)]
pub struct ReadControlledPlayer<'a> {
    controlled_player: Option<ReadExpect<'a, ControlledPlayer>>,
}

impl<'a> ReadControlledPlayer<'a> {
    pub fn value(&self) -> Option<Entity> {
        self.controlled_player.as_ref().map(|player| player.0)
    }
}

#[derive(SystemData)]
pub struct ReadControlledPlayerStorage<'a, T: Component> {
    controlled_player: ReadControlledPlayer<'a>,
    storage: ReadStorage<'a, T>,
}

impl<'a, T: Component> ReadControlledPlayerStorage<'a, T> {
    pub fn value(&self) -> Option<&T> {
        self.controlled_player
            .value()
            .and_then(|player| self.storage.get(player))
    }
}
