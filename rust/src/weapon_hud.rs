use crate::camera::Camera;
use crate::components::player::Player;
use crate::ecs::ReadControlledPlayerStorage;
use crate::game::HudEnabled;
use crate::graphics::{texture, DrawBuffer, EntityRendererSettings};
use crate::util::vector::Vec2;
use crate::{vec2, Mat3};
use specs::prelude::*;
use specs::{System, WriteExpect};

const BASE_SIZE: f64 = 0.25;
const BASE_GAP: f64 = -0.05;
const MARGIN: f64 = 0.03;

pub struct DrawWeaponHud;

#[derive(SystemData)]
pub struct DrawWeaponHudData<'a> {
    camera: ReadExpect<'a, Camera>,
    buffer: WriteExpect<'a, DrawBuffer>,
    hud_enabled: ReadExpect<'a, HudEnabled>,
    player: ReadControlledPlayerStorage<'a, Player>,
}

impl<'a> System<'a> for DrawWeaponHud {
    type SystemData = DrawWeaponHudData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        if !data.hud_enabled.0 {
            return;
        }

        let player = match data.player.value() {
            Some(player) => player,
            None => return,
        };

        let guns = player.guns();

        let base_y = 0.2 * data.camera.vmin_ratio() + MARGIN - 1.0;
        let size = BASE_SIZE * data.camera.vmin_ratio();
        let gap = BASE_GAP * data.camera.vmin_ratio();

        for (i, gun) in guns.iter().enumerate() {
            let y = (size + gap) * (guns.len() - i - 1) as f64 + base_y;

            let scale = Vec2::one() * size;
            let translation = vec2(data.camera.aspect_ratio() - size / 2.0 - MARGIN, y);

            let inverse_view = data.camera.view().inverse();
            let transform = inverse_view * Mat3::transform(translation, scale);

            let background = if i == player.selected_gun_index() {
                texture::gun_hud_background_active()
            } else {
                texture::gun_hud_background_inactive()
            };

            data.buffer.push(EntityRendererSettings {
                transform: transform * Mat3::scale(vec2(1.3, 1.0)),
                uv_transform: background,
            });

            data.buffer.push(EntityRendererSettings {
                transform,
                uv_transform: gun.spec().texture(),
            });
        }
    }
}
