use specs::{Entity, Join, RunNow, World, WorldExt};
use std::ops::DerefMut;

use crate::camera::Camera;
use crate::components::bounds::Bounds;
use crate::components::bullet::{Bullet, BulletTarget, UpdateBullets};
use crate::components::enemy::{Enemy, KillLostEnemies};
use crate::components::health::{DeleteDeadEntities, Health};
use crate::components::melee_attacker::{AttackPlayers, MeleeAttacker};
use crate::components::physics::{Collider, Physics, SimulatePhysics};
use crate::components::player::{Player, ReduceAttackCooldowns};
use crate::components::player_seeker::{PlayerSeeker, SeekPlayers};
use crate::components::regen::{HealthRegen, RegenerateHealth};
use crate::components::spawning_demon::{FinishDemonSpawning, SpawningDemon};
use crate::components::sprite::animation::{Animation, GenerateAnimationSprites};
use crate::components::sprite::character_animation::{
    CharacterAnimation, FaceToVelocities, FacesVelocity, Facing, GenerateCharacterAnimationSprites,
};
use crate::components::sprite::sprite::{GenerateStaticSprites, Sprite};
use crate::components::sprite::{DrawSprites, FrameSprites};
use crate::ecs::WorldExtensions;
use crate::entities::player;
use crate::graphics::{DrawBuffer, ResetDrawBuffer};
use crate::gun::GunSpecGenerator;
use crate::interaction::{Interaction, UiText};
use crate::map::draw::{DrawMapBase, DrawMapOverlay};
use crate::map::Map;
use crate::progression::{Progression, SpawnEnemies};
use crate::util::random::Random;
use crate::util::rect::Rect;
use crate::util::vector::Vec2;
use crate::weapon_hud::DrawWeaponHud;
use crate::weapon_positioning::DrawHeldWeapons;
use crate::{vec2, Inputs};

pub const SCREEN_HEIGHT: f64 = 7.5;

pub struct ControlledPlayer(pub Entity);

pub struct Delta(pub f64);

pub struct Timestamp(pub f64);

pub struct DisplayedInteraction(pub Option<Interaction>);

pub struct HudEnabled(pub bool);

pub struct Focus(Vec2);

pub struct Game {
    world: World,
}

impl Game {
    pub fn new(seed: u32) -> Self {
        *Random::global() = Random::new(seed);

        let mut world = World::new();

        world.insert(Inputs::new());
        world.insert(Map::new(seed));
        world.insert(Timestamp(0.0));
        world.insert(Progression::new());
        world.insert(GunSpecGenerator::new());
        world.insert(FrameSprites::new());
        world.insert(DrawBuffer::new());
        world.insert(Camera::new(Rect::focused(Vec2::zero(), Vec2::one())));
        world.insert(DisplayedInteraction(None));
        world.insert(HudEnabled(false));
        world.insert(Focus(vec2(0.5, 0.5)));

        world.register::<Bounds>();
        world.register::<Sprite>();
        world.register::<Animation>();
        world.register::<CharacterAnimation>();
        world.register::<Facing>();
        world.register::<FacesVelocity>();
        world.register::<Physics>();
        world.register::<Collider>();
        world.register::<Health>();
        world.register::<HealthRegen>();
        world.register::<PlayerSeeker>();
        world.register::<MeleeAttacker>();
        world.register::<Player>();
        world.register::<Enemy>();
        world.register::<Bullet>();
        world.register::<BulletTarget>();
        world.register::<SpawningDemon>();

        let player = player::create(&mut world, vec2(0.5, 0.5));
        world.insert(ControlledPlayer(player));

        Game { world }
    }

    pub fn tick(&mut self, delta: f64) {
        self.world.insert(Delta(delta));
        self.world.fetch_mut::<Timestamp>().0 += delta;

        ReduceAttackCooldowns.run_now(&self.world);

        self.world.fetch_mut::<DisplayedInteraction>().0 = Interaction::current(&self.world);
        self.handle_input();

        RegenerateHealth.run_now(&self.world);
        SeekPlayers.run_now(&self.world);
        AttackPlayers.run_now(&self.world);
        SimulatePhysics.run_now(&self.world);
        UpdateBullets.run_now(&self.world);
        KillLostEnemies.run_now(&self.world);
        SpawnEnemies.run_now(&self.world);
        FinishDemonSpawning.run_now(&self.world);
        DeleteDeadEntities.run_now(&self.world);

        self.world.maintain();
        self.maintain_controlled_player();

        if let Some(bounds) = self.world.controlled_player_read::<Bounds>() {
            self.world.fetch_mut::<Focus>().0 = bounds.0.center();
        }
    }

    pub fn enable_hud(&mut self) {
        self.world.fetch_mut::<HudEnabled>().0 = true;
    }

    pub fn is_over(&self) -> bool {
        self.world.controlled_player().is_none()
    }

    pub fn inputs(&mut self) -> impl DerefMut<Target = Inputs> + '_ {
        self.world.fetch_mut()
    }

    fn handle_input(&mut self) {
        let focus = self.world.fetch::<Focus>().0;
        let mouse = self.world.fetch::<Inputs>().mouse();
        let mouse_position = mouse * SCREEN_HEIGHT / 2.0 + focus;
        player::handle_input(&mut self.world, mouse_position);
    }

    fn maintain_controlled_player(&mut self) {
        let should_delete = match self.world.controlled_player() {
            Some(player) => !self.world.is_alive(player),
            None => false,
        };

        if should_delete {
            log::info!("Removed player from game");
            self.world.remove::<ControlledPlayer>();
        }
    }

    pub fn draw(&mut self, aspect_ratio: f64) -> *const u8 {
        *self.world.fetch_mut() = self.camera(aspect_ratio);

        ResetDrawBuffer.run_now(&self.world);
        DrawMapBase.run_now(&self.world);
        FaceToVelocities.run_now(&self.world);
        GenerateStaticSprites.run_now(&self.world);
        GenerateAnimationSprites.run_now(&self.world);
        GenerateCharacterAnimationSprites.run_now(&self.world);
        DrawHeldWeapons.run_now(&self.world);
        DrawSprites.run_now(&self.world);
        DrawMapOverlay.run_now(&self.world);
        DrawWeaponHud.run_now(&self.world);

        self.world.fetch::<DrawBuffer>().as_ptr()
    }

    fn camera(&self, aspect_ratio: f64) -> Camera {
        let focus = self.world.fetch::<Focus>().0;
        let size = vec2(aspect_ratio, 1.0) * SCREEN_HEIGHT;
        Camera::new(Rect::focused(focus, size))
    }

    pub fn entity_count(&self) -> usize {
        (&self.world.entities()).join().count()
    }

    pub fn round(&self) -> usize {
        self.world.fetch::<Progression>().round()
    }

    pub fn credits(&self) -> Option<usize> {
        self.world
            .controlled_player_read::<Player>()
            .map(|player| player.credits())
    }

    pub fn interaction_heading(&self) -> String {
        let default = UiText::new();
        let interaction = self.world.fetch::<DisplayedInteraction>();

        let heading = interaction
            .0
            .as_ref()
            .map(Interaction::heading)
            .unwrap_or(&default);

        serde_json::to_string(heading).unwrap()
    }

    pub fn interaction_caption(&self) -> String {
        let default = UiText::new();
        let interaction = self.world.fetch::<DisplayedInteraction>();

        let heading = interaction
            .0
            .as_ref()
            .map(Interaction::caption)
            .unwrap_or(&default);

        serde_json::to_string(heading).unwrap()
    }

    pub fn current_ammo(&self) -> Option<usize> {
        self.world
            .controlled_player_read::<Player>()
            .map(|player| player.selected_gun().current_ammo())
    }

    pub fn max_ammo(&self) -> Option<usize> {
        self.world
            .controlled_player_read::<Player>()
            .map(|player| player.selected_gun().max_ammo())
    }
}
