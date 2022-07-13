use crate::game::Delta;
use crate::gun::{Gun, GunSpec};
use crate::util::vector::Vec2;
use crate::Inputs;
use specs::{Component, HashMapStorage, Join, ReadExpect, System, WriteStorage};
use std::rc::Rc;

const STARTING_CREDITS: usize = 500;

pub struct Player {
    guns: Vec<Gun>,
    max_guns: usize,
    selected_gun: usize,
    last_selected_gun: usize,
    attack_cooldown: f64,
    credits: usize,
    mouse_position: Vec2,
    mouse_down: bool,
}

impl Player {
    pub fn new() -> Self {
        let gun = Gun::new(Rc::new(GunSpec::starter_gun()));

        Self {
            guns: vec![gun],
            max_guns: 3,
            selected_gun: 0,
            last_selected_gun: 0,
            attack_cooldown: 0.0,
            credits: STARTING_CREDITS,
            mouse_position: Vec2::zero(),
            mouse_down: false,
        }
    }

    pub fn update_inputs(&mut self, inputs: &Inputs, mouse_position: Vec2) {
        self.mouse_position = mouse_position;
        self.mouse_down = inputs.is_mouse_down();
    }

    pub fn is_mouse_down(&self) -> bool {
        self.mouse_down
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    pub fn try_attack(&mut self) -> AttackResult {
        let gun = &mut self.guns[self.selected_gun];

        if self.attack_cooldown <= 0.0 && *gun.current_ammo_mut() > 0 {
            self.attack_cooldown = gun.spec().cooldown();
            *gun.current_ammo_mut() -= 1;

            AttackResult::Can {
                damage: gun.spec().damage(),
                knockback: gun.spec().knockback(),
                penetration: gun.spec().penetration(),
                inaccuracy: gun.spec().inaccuracy(),
                bullet_count: gun.spec().bullet_count(),
            }
        } else {
            AttackResult::Cant
        }
    }

    pub fn credits(&self) -> usize {
        self.credits
    }

    pub fn credits_mut(&mut self) -> &mut usize {
        &mut self.credits
    }

    pub fn selected_gun(&self) -> &Gun {
        &self.guns[self.selected_gun]
    }

    pub fn equip_gun(&mut self, gun: Gun) {
        if self.guns.len() < self.max_guns {
            self.selected_gun = self.guns.len();
            self.last_selected_gun = self.selected_gun;
            self.guns.push(gun);
        } else {
            self.guns[self.selected_gun] = gun;
        }
    }

    pub fn select_gun(&mut self, slot: usize) {
        if slot < self.guns.len() {
            self.last_selected_gun = self.selected_gun;
            self.selected_gun = slot;
        }
    }

    pub fn guns(&self) -> &[Gun] {
        &self.guns
    }

    pub fn selected_gun_index(&self) -> usize {
        self.selected_gun
    }
}

impl Component for Player {
    type Storage = HashMapStorage<Self>;
}

pub enum AttackResult {
    Cant,
    Can {
        damage: f64,
        knockback: f64,
        penetration: f64,
        inaccuracy: f64,
        bullet_count: usize,
    },
}

pub struct ReduceAttackCooldowns;

impl<'a> System<'a> for ReduceAttackCooldowns {
    type SystemData = (ReadExpect<'a, Delta>, WriteStorage<'a, Player>);

    fn run(&mut self, (delta, mut players): Self::SystemData) {
        for player in (&mut players).join() {
            if player.attack_cooldown > 0.0 {
                player.attack_cooldown -= delta.0;
            }
        }
    }
}
