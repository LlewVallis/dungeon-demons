use std::rc::Rc;

use crate::audio::{play_sound, sound};
use fxhash::FxHashMap;
use lazy_static::lazy_static;

use crate::graphics::texture;
use crate::util::random::Random;
use crate::util::vector::vec2;
use crate::util::vector::Vec2;
use crate::Mat3;

pub struct Gun {
    spec: Rc<GunSpec>,
    ammo: usize,
}

impl Gun {
    pub fn new(spec: Rc<GunSpec>) -> Self {
        Self {
            ammo: spec.ammo,
            spec,
        }
    }

    pub fn current_ammo(&self) -> usize {
        self.ammo
    }

    pub fn current_ammo_mut(&mut self) -> &mut usize {
        &mut self.ammo
    }

    pub fn max_ammo(&self) -> usize {
        self.spec.ammo
    }

    pub fn play_sound(&self) {
        match self.spec.archetype.category {
            ArchetypeCategory::Slow => play_sound(sound::shoot_slow()),
            ArchetypeCategory::Fast => play_sound(sound::shoot_fast()),
            ArchetypeCategory::Sniper => play_sound(sound::shoot_sniper()),
            ArchetypeCategory::Shotgun => play_sound(sound::shoot_shotgun()),
        }
    }

    pub fn spec(&self) -> &GunSpec {
        &self.spec
    }
}

pub struct GunSpec {
    name: String,
    damage: f64,
    knockback: f64,
    cooldown: f64,
    penetration: f64,
    accuracy: f64,
    bullet_count: usize,
    ammo: usize,
    archetype: &'static GunArchetype,
}

impl GunSpec {
    pub fn starter_gun() -> Self {
        Self {
            name: String::from("Starter Pistol").into(),
            damage: PISTOL.damage,
            knockback: PISTOL.knockback,
            cooldown: PISTOL.cooldown,
            penetration: PISTOL.penetration,
            accuracy: PISTOL.accuracy,
            bullet_count: PISTOL.bullet_count,
            ammo: PISTOL.ammo,
            archetype: &PISTOL,
        }
    }

    pub fn dps_heuristic(&self) -> f64 {
        self.damage / self.cooldown * self.bullet_count as f64 * self.hit_rate_heuristic()
    }

    pub fn kbps_heuristic(&self) -> f64 {
        self.knockback / self.cooldown * self.bullet_count as f64 * self.hit_rate_heuristic()
    }

    pub fn ammo_duration_heuristic(&self) -> f64 {
        self.ammo as f64 * self.cooldown
    }

    fn hit_rate_heuristic(&self) -> f64 {
        (1.0 - self.inaccuracy().powi(2)) * (self.penetration / 5.0 + 0.8)
    }

    pub fn is_similar(&self, other: &Self) -> bool {
        self.archetype.category == other.archetype.category
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn damage(&self) -> f64 {
        self.damage
    }

    pub fn knockback(&self) -> f64 {
        self.knockback
    }

    pub fn cooldown(&self) -> f64 {
        self.cooldown
    }

    pub fn penetration(&self) -> f64 {
        self.penetration
    }

    pub fn accuracy(&self) -> f64 {
        self.accuracy
    }

    pub fn bullet_count(&self) -> usize {
        self.bullet_count
    }

    pub fn ammo(&self) -> usize {
        self.ammo
    }

    pub fn inaccuracy(&self) -> f64 {
        1.0 / self.accuracy.max(1.0)
    }

    pub fn texture(&self) -> Mat3 {
        self.archetype.texture
    }

    pub fn handle_offset(&self) -> Vec2 {
        self.archetype.handle_offset
    }

    pub fn nose_offset(&self) -> Vec2 {
        self.archetype.nose_offset
    }

    pub fn texture_size(&self) -> f64 {
        self.archetype.texture_size
    }
}

struct GunArchetype {
    category: ArchetypeCategory,
    name: String,
    rarity: f64,
    damage: f64,
    knockback: f64,
    cooldown: f64,
    penetration: f64,
    accuracy: f64,
    bullet_count: usize,
    ammo: usize,
    texture: Mat3,
    handle_offset: Vec2,
    nose_offset: Vec2,
    texture_size: f64,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ArchetypeCategory {
    Slow,
    Fast,
    Sniper,
    Shotgun,
}

lazy_static! {
    static ref PISTOL: GunArchetype = GunArchetype {
        category: ArchetypeCategory::Slow,
        name: String::from("Pistol"),
        rarity: 1.0,
        damage: 50.0,
        knockback: 4.0,
        cooldown: 0.66,
        penetration: 0.75,
        accuracy: 50.0,
        bullet_count: 1,
        ammo: 75,
        texture: texture::pistol(),
        handle_offset: vec2(-2.5, -0.75) / 32.0,
        nose_offset: vec2(4.0, 1.5) / 32.0,
        texture_size: 0.375,
    };
    static ref LMG: GunArchetype = GunArchetype {
        category: ArchetypeCategory::Fast,
        name: String::from("Light Machine Gun"),
        rarity: 1.0,
        damage: 7.5,
        knockback: 0.5,
        cooldown: 0.075,
        penetration: 0.0,
        accuracy: 7.5,
        bullet_count: 1,
        ammo: 400,
        texture: texture::lmg(),
        handle_offset: vec2(-1.5, -0.5) / 32.0,
        nose_offset: vec2(6.0, 1.5) / 32.0,
        texture_size: 0.45,
    };
    static ref SHOTGUN: GunArchetype = GunArchetype {
        category: ArchetypeCategory::Shotgun,
        name: String::from("Shotgun"),
        rarity: 1.0,
        damage: 20.0,
        knockback: 2.0,
        cooldown: 1.0,
        penetration: 0.0,
        accuracy: 5.0,
        bullet_count: 6,
        ammo: 30,
        texture: texture::shotgun(),
        handle_offset: vec2(-4.0, -0.25) / 32.0,
        nose_offset: vec2(6.0, 1.5) / 32.0,
        texture_size: 0.45,
    };
    static ref RIFLE: GunArchetype = GunArchetype {
        category: ArchetypeCategory::Slow,
        name: String::from("Assault Rifle"),
        rarity: 1.0,
        damage: 25.0,
        knockback: 2.0,
        cooldown: 0.33,
        penetration: 0.33,
        accuracy: 35.0,
        bullet_count: 1,
        ammo: 150,
        texture: texture::rifle(),
        handle_offset: vec2(-2.5, -0.25) / 32.0,
        nose_offset: vec2(6.0, 1.5) / 32.0,
        texture_size: 0.5,
    };
    static ref SNIPER: GunArchetype = GunArchetype {
        category: ArchetypeCategory::Sniper,
        name: String::from("Sniper Rifle"),
        rarity: 1.0,
        damage: 90.0,
        knockback: 10.0,
        cooldown: 1.25,
        penetration: 1.0,
        accuracy: 66.0,
        bullet_count: 1,
        ammo: 30,
        texture: texture::sniper(),
        handle_offset: vec2(-2.0, -0.25) / 32.0,
        nose_offset: vec2(6.0, 1.5) / 32.0,
        texture_size: 0.5,
    };
    static ref SMG: GunArchetype = GunArchetype {
        category: ArchetypeCategory::Fast,
        name: String::from("Submachine Gun"),
        rarity: 1.0,
        damage: 17.5,
        knockback: 0.0,
        cooldown: 0.2,
        penetration: 0.0,
        accuracy: 15.0,
        bullet_count: 1,
        ammo: 250,
        texture: texture::smg(),
        handle_offset: vec2(0.25, -0.25) / 32.0,
        nose_offset: vec2(6.0, 2.0) / 32.0,
        texture_size: 0.5,
    };
    static ref ARCHETYPES: Vec<&'static GunArchetype> =
        vec![&PISTOL, &LMG, &SHOTGUN, &RIFLE, &SNIPER, &SMG];
}

pub struct GunSpecGenerator {
    archetype_counts: FxHashMap<usize, usize>,
    modifier_counts: FxHashMap<Modifier, usize>,
}

impl GunSpecGenerator {
    pub fn new() -> Self {
        Self {
            archetype_counts: FxHashMap::default(),
            modifier_counts: FxHashMap::default(),
        }
    }

    pub fn generate(&mut self, position: Vec2) -> GunSpec {
        let distance = position.length() + Random::global().next_f64_in(0.0..25.0);
        let quality = distance / 70.0 + 0.33;

        let archetype = self.generate_archetype();
        let modifier = self.generate_modifier();

        let mut spec = GunSpec {
            name: format!("{}{}", modifier.prefix(), archetype.name),
            damage: archetype.damage,
            knockback: archetype.knockback,
            cooldown: archetype.cooldown,
            penetration: archetype.penetration,
            accuracy: archetype.accuracy,
            bullet_count: archetype.bullet_count,
            ammo: archetype.ammo,
            archetype,
        };

        spec.damage += spec.damage * quality;
        modifier.apply(&mut spec);

        spec
    }

    fn generate_archetype(&mut self) -> &'static GunArchetype {
        let scores = ARCHETYPES
            .iter()
            .enumerate()
            .map(|(i, archetype)| {
                let count = *self.archetype_counts.entry(i).or_insert(0);
                1.0 / ((count + 1) as f64 * archetype.rarity)
            })
            .collect::<Vec<_>>();

        let archetype_index = Random::global().weighted_index(|| scores.iter().copied());

        *self.archetype_counts.entry(archetype_index).or_insert(0) += 1;
        &ARCHETYPES[archetype_index]
    }

    fn generate_modifier(&mut self) -> Modifier {
        let scores = MODIFIERS
            .iter()
            .map(|modifier| {
                let count = *self.modifier_counts.entry(*modifier).or_insert(0);
                1.0 / ((count + 1) as f64 * modifier.rarity())
            })
            .collect::<Vec<_>>();

        let modifier_index = Random::global().weighted_index(|| scores.iter().copied());
        let modifier = MODIFIERS[modifier_index];

        *self.modifier_counts.entry(modifier).or_insert(0) += 1;
        modifier
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum Modifier {
    Plain,
    Brutal,
    Plentiful,
    Rapid,
    Forceful,
    Unhinged,
    Wild,
}

impl Modifier {
    pub fn prefix(&self) -> &'static str {
        match self {
            Modifier::Plain => "",
            Modifier::Brutal => "Brutal ",
            Modifier::Plentiful => "Plentiful ",
            Modifier::Rapid => "Rapid ",
            Modifier::Forceful => "Forceful ",
            Modifier::Unhinged => "Unhinged ",
            Modifier::Wild => "Wild ",
        }
    }

    pub fn apply(&self, spec: &mut GunSpec) {
        match self {
            Modifier::Plain => {}
            Modifier::Brutal => {
                spec.damage *= 1.75;
                spec.cooldown *= 0.75;
                spec.ammo = spec.ammo * 2 / 3;
            }
            Modifier::Plentiful => {
                spec.ammo = spec.ammo * 3 / 2;
            }
            Modifier::Rapid => {
                spec.knockback *= 0.5;
                spec.cooldown *= 0.66;
                spec.accuracy *= 0.75;
            }
            Modifier::Forceful => {
                spec.cooldown *= 1.15;
                spec.knockback *= 2.5;
            }
            Modifier::Unhinged => {
                spec.damage *= 2.5;
                spec.knockback *= 4.0;
                spec.accuracy = (spec.accuracy * 0.25).min(4.0);
            }
            Modifier::Wild => {
                spec.damage *= 0.66;
                spec.bullet_count *= 2;
                spec.accuracy *= 0.5;
            }
        }
    }

    pub fn rarity(&self) -> f64 {
        match self {
            Modifier::Plain => 0.1,
            Modifier::Brutal => 1.0,
            Modifier::Plentiful => 1.0,
            Modifier::Rapid => 1.0,
            Modifier::Forceful => 1.5,
            Modifier::Unhinged => 2.0,
            Modifier::Wild => 1.5,
        }
    }
}

const MODIFIERS: &[Modifier] = &[
    Modifier::Plain,
    Modifier::Brutal,
    Modifier::Plentiful,
    Modifier::Rapid,
    Modifier::Forceful,
    Modifier::Unhinged,
    Modifier::Wild,
];
