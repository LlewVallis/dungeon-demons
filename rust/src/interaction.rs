use crate::audio::{play_sound, sound};
use serde::{Serialize, Serializer};
use specs::{Entity, World};

use crate::components::bounds::Bounds;
use crate::components::player::Player;
use crate::ecs::WorldExtensions;
use crate::game::IsMobile;
use crate::gun::GunSpec;
use crate::map::chest::Chest;
use crate::map::{Map, Tile};
use crate::util::coord::Coord;
use crate::util::rect::Rect;
use crate::util::vector::Vec2;
use crate::vec2;

const INTERACT_RANGE: f64 = 0.33;

pub struct Interaction {
    heading: UiText,
    caption: UiText,
    interaction_type: InteractionType,
}

impl Interaction {
    pub fn attempt_interact(world: &World) {
        let interaction = match Self::current(world) {
            Some(interaction) => interaction,
            None => return,
        };

        interaction.interaction_type.perform(world);
    }

    pub fn current(world: &World) -> Option<Interaction> {
        let bounds = world.controlled_player_read::<Bounds>()?.0;

        let search = bounds.expand(INTERACT_RANGE);

        let mut current = None;
        let mut current_distance = INTERACT_RANGE;

        Self::current_barrier(world, search, &mut current, &mut current_distance);
        Self::current_chest(world, search, &mut current, &mut current_distance);
        Self::current_pickup(world, search, &mut current, &mut current_distance);

        current
    }

    fn current_barrier(
        world: &World,
        search: Rect,
        current: &mut Option<Interaction>,
        current_distance: &mut f64,
    ) {
        let map = world.fetch::<Map>();

        for coord in search.coords() {
            let distance = Rect::euclidean_distance(coord.bounds(), search);
            if distance >= *current_distance {
                continue;
            }

            if map.at(coord) != Tile::Barrier {
                continue;
            }

            let cost = 500;

            *current_distance = distance;
            *current = Some(Interaction {
                heading: UiText::of(format!("Remove barrier ${}", cost), TextColor::white()),
                caption: Self::cost_caption(world, cost),
                interaction_type: InteractionType::Barrier {
                    cost,
                    position: coord,
                },
            });
        }
    }

    fn current_chest(
        world: &World,
        search: Rect,
        current: &mut Option<Interaction>,
        current_distance: &mut f64,
    ) {
        let map = world.fetch::<Map>();

        for chest in map.chests_in(search) {
            if chest.is_open() {
                continue;
            }

            let distance = Rect::euclidean_distance(chest.bounds(), search);

            if distance >= *current_distance {
                continue;
            }

            let cost = 500;

            *current_distance = distance;
            *current = Some(Interaction {
                heading: UiText::of(format!("Open chest ${}", cost), TextColor::white()),
                caption: Self::cost_caption(world, cost),
                interaction_type: InteractionType::Chest {
                    cost,
                    position: chest.position(),
                },
            });
        }
    }

    fn current_pickup(
        world: &World,
        search: Rect,
        current: &mut Option<Interaction>,
        current_distance: &mut f64,
    ) {
        let map = world.fetch::<Map>();

        let player = match world.controlled_player() {
            Some(player) => player,
            None => return,
        };

        for chest in map.chests_in(search) {
            if !chest.can_pickup(player) {
                continue;
            }

            let distance = Rect::euclidean_distance(chest.bounds(), search);

            if distance >= *current_distance {
                continue;
            }

            let gun = chest.gun().unwrap();

            *current_distance = distance;
            *current = Some(Interaction {
                heading: UiText::of(format!("Pickup {}", gun.name()), TextColor::white()),
                caption: Self::pickup_caption(world, player, gun),
                interaction_type: InteractionType::Pickup {
                    position: chest.position(),
                },
            });
        }
    }

    fn pickup_caption(world: &World, player: Entity, new: &GunSpec) -> UiText {
        let player = world.unwrap_read::<Player>(player);

        let old = player.selected_gun().spec();

        let mut caption = UiText::new();

        if GunSpec::is_similar(new, old) {
            Self::pickup_caption_similar(&mut caption, old, new);
        } else {
            Self::pickup_caption_non_similar(&mut caption, old, new);
        }

        caption
    }

    fn pickup_caption_similar(caption: &mut UiText, old: &GunSpec, new: &GunSpec) {
        Self::pickup_caption_percent(caption, "DMG", old.damage(), new.damage());
        Self::pickup_caption_percent(caption, "SPD", 1.0 / old.cooldown(), 1.0 / new.cooldown());
        Self::pickup_caption_percent(caption, "ACC", old.accuracy(), new.accuracy());
        Self::pickup_caption_percent(caption, "KB", old.knockback(), new.knockback());
        Self::pickup_caption_absolute(caption, "AMMO", old.ammo(), new.ammo());
        Self::pickup_caption_absolute(caption, "BULLETS", old.bullet_count(), new.bullet_count());
    }

    fn pickup_caption_non_similar(caption: &mut UiText, old: &GunSpec, new: &GunSpec) {
        Self::pickup_caption_comparison(caption, "~DMG", old.dps_heuristic(), new.dps_heuristic());
        Self::pickup_caption_comparison(caption, "~KB", old.kbps_heuristic(), new.kbps_heuristic());
        Self::pickup_caption_comparison(
            caption,
            "~AMMO",
            old.ammo_duration_heuristic(),
            new.ammo_duration_heuristic(),
        );
    }

    fn pickup_caption_comparison(caption: &mut UiText, label: &str, old: f64, new: f64) {
        if old == new {
            return;
        }

        let percent = (new - old) / old * 100.0;

        let indicators = if percent >= 100.0 {
            "+++"
        } else if percent >= 50.0 {
            "++"
        } else if percent >= 0.0 {
            "+"
        } else if percent >= 50.0 {
            "-"
        } else if percent >= 100.0 {
            "--"
        } else {
            "---"
        };

        let text = format!("{}{} ", label, indicators);

        let color = if new >= old {
            TextColor::green()
        } else {
            TextColor::red()
        };

        caption.push(text, color);
    }

    fn pickup_caption_percent(caption: &mut UiText, label: &str, old: f64, new: f64) {
        let percent = ((new - old) / old * 100.0).round() as i32;
        if percent == 0 {
            return;
        }

        let text = if percent > 500 {
            format!("{} >+500% ", label)
        } else if percent < -500 {
            format!("{} <-500% ", label)
        } else {
            format!("{} {:+}% ", label, percent)
        };

        let color = if new >= old {
            TextColor::green()
        } else {
            TextColor::red()
        };

        caption.push(text, color);
    }

    fn pickup_caption_absolute(caption: &mut UiText, label: &str, old: usize, new: usize) {
        if new == old {
            return;
        }

        let text = format!("{} {:+} ", label, new as isize - old as isize);

        let color = if new >= old {
            TextColor::green()
        } else {
            TextColor::red()
        };

        caption.push(text, color);
    }

    pub fn heading(&self) -> &UiText {
        &self.heading
    }

    pub fn caption(&self) -> &UiText {
        &self.caption
    }

    fn cost_caption(world: &World, cost: usize) -> UiText {
        let credits = Self::controlled_player_credits(world).unwrap_or(0);
        let is_mobile = world.fetch::<IsMobile>().0;

        if credits < cost {
            UiText::of("Insufficient funds", TextColor::red())
        } else if is_mobile {
            UiText::of("Tap joystick to purchase", TextColor::green())
        } else {
            UiText::of("Press SPACE to purchase", TextColor::green())
        }
    }

    fn controlled_player_credits(world: &World) -> Option<usize> {
        world
            .controlled_player_read::<Player>()
            .map(|player| player.credits())
    }
}

enum InteractionType {
    Barrier { position: Coord, cost: usize },
    Chest { position: Vec2, cost: usize },
    Pickup { position: Vec2 },
}

impl InteractionType {
    fn perform(&self, world: &World) {
        let mut map = world.fetch_mut::<Map>();

        let player = match world.controlled_player() {
            Some(player) => player,
            None => return,
        };

        match *self {
            InteractionType::Barrier { position, cost } => {
                if !self.consume_credits(world, player, cost) {
                    return;
                }

                log::debug!("Removed barrier at {:?}", position);
                play_sound(sound::purchase());
                map.set(position, Tile::Floor)
            }
            InteractionType::Chest { position, cost } => {
                if !self.consume_credits(world, player, cost) {
                    return;
                }

                if let Some(chest) = self.chest_mut_at(&mut map, position) {
                    log::debug!("Opened chest at {:?}", position);
                    play_sound(sound::purchase());
                    chest.open(world);
                } else {
                    log::warn!("Attempted to purchase removed chest at {:?}", position);
                }
            }
            InteractionType::Pickup { position } => {
                if let Some(chest) = self.chest_mut_at(&mut map, position) {
                    log::debug!("Picked up from chest at {:?}", position);
                    play_sound(sound::pickup());
                    chest.pickup(world, player);
                } else {
                    log::warn!("Attempted to pickup from removed chest at {:?}", position);
                }
            }
        }
    }

    fn chest_mut_at<'a>(&self, map: &'a mut Map, position: Vec2) -> Option<&'a mut Chest> {
        let search = Rect::focused(position, vec2(0.5, 0.5));
        let mut chests = map.chests_in_mut(search);
        chests.next()
    }

    fn consume_credits(&self, world: &World, player: Entity, cost: usize) -> bool {
        let mut player = world.unwrap_write::<Player>(player);

        if player.credits() < cost {
            return false;
        }

        *player.credits_mut() -= cost;
        true
    }
}

#[derive(Serialize)]
#[serde(transparent)]
pub struct UiText {
    segments: Vec<(String, TextColor)>,
}

impl UiText {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn of(text: impl Into<String>, color: TextColor) -> Self {
        let mut result = Self::new();
        result.push(text, color);
        result
    }

    pub fn push(&mut self, text: impl Into<String>, color: TextColor) {
        self.segments.push((text.into(), color));
    }
}

#[derive(Copy, Clone)]
pub struct TextColor(pub u8, pub u8, pub u8);

impl TextColor {
    pub fn white() -> Self {
        TextColor(255, 255, 255)
    }

    pub fn green() -> Self {
        TextColor(113, 255, 100)
    }

    pub fn red() -> Self {
        TextColor(255, 89, 92)
    }
}

impl Serialize for TextColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let output = format!("rgb({}, {}, {})", self.0, self.1, self.2);
        serializer.serialize_str(&output)
    }
}
