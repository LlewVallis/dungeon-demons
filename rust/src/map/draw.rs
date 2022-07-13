use std::f64::consts::PI;

use fxhash::{hash32, FxHashMap};
use specs::{Entity, ReadExpect, System, World, WriteExpect};

use crate::camera::Camera;
use crate::components::sprite::FrameSprites;
use crate::ecs::ReadControlledPlayer;
use crate::game::Timestamp;
use crate::graphics::{texture, DrawBuffer, EntityRendererSettings};
use crate::map::chunk::CHUNK_SIZE_I32;
use crate::map::{Map, Tile};
use crate::util::coord::{coord, Coord};
use crate::util::random::hash32_vec;
use crate::util::rect::rect;
use crate::util::vector::Vec2;
use crate::{vec2, Mat3};
use specs::prelude::*;

pub const RENDER_REGION_SIZE: i32 = 8;

const MAX_SPRITE_RADIUS: i32 = 5;

pub struct DrawMapBase;

#[derive(SystemData)]
pub struct DrawMapBaseData<'a> {
    map: ReadExpect<'a, Map>,
    timestamp: ReadExpect<'a, Timestamp>,
    camera: ReadExpect<'a, Camera>,
    buffer: WriteExpect<'a, DrawBuffer>,
    sprites: WriteExpect<'a, FrameSprites>,
    controlled_player: ReadControlledPlayer<'a>,
}

impl<'a> System<'a> for DrawMapBase {
    type SystemData = DrawMapBaseData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        data.map.draw_static(*data.camera, &mut data.buffer);
        data.map.draw_dynamic(
            data.timestamp.0,
            *data.camera,
            data.controlled_player.value(),
            &mut data.sprites,
        );
    }
}

pub struct DrawMapOverlay;

#[derive(SystemData)]
pub struct DrawMapOverlayData<'a> {
    map: ReadExpect<'a, Map>,
    buffer: WriteExpect<'a, DrawBuffer>,
}

impl<'a> System<'a> for DrawMapOverlay {
    type SystemData = DrawMapOverlayData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        for region in data.map.render_regions.borrow().values() {
            region.draw_overlay(&mut data.buffer);
        }
    }
}

impl Map {
    fn draw_static(&self, camera: Camera, buffer: &mut DrawBuffer) {
        let min = camera.min_coord();
        let max = camera.max_coord();

        let min_region = min.div_euclid(RENDER_REGION_SIZE);
        let max_region = max.div_euclid(RENDER_REGION_SIZE);

        let mut render_regions = self.render_regions.borrow_mut();
        let mut new_render_regions = FxHashMap::default();

        for coord in Coord::between_inclusive(min_region, max_region) {
            let base = coord * RENDER_REGION_SIZE;

            let removed = render_regions.remove(&(coord.x(), coord.y()));
            let region = match removed {
                Some(region) => region,
                None => RenderRegion::new(self, base),
            };

            new_render_regions.insert((coord.x(), coord.y()), region);
        }

        for region in new_render_regions.values() {
            region.draw_bottom(buffer);
        }

        for region in new_render_regions.values() {
            region.draw_top(buffer);
        }

        *render_regions = new_render_regions;
    }

    fn draw_dynamic(
        &self,
        timestamp: f64,
        camera: Camera,
        player: Option<Entity>,
        sprites: &mut FrameSprites,
    ) {
        let min = camera.min_coord() - MAX_SPRITE_RADIUS;
        let max = camera.max_coord() + MAX_SPRITE_RADIUS;

        let min_chunk = (min + CHUNK_SIZE_I32 / 2).div_euclid(CHUNK_SIZE_I32);
        let max_chunk = (max + CHUNK_SIZE_I32 / 2).div_euclid(CHUNK_SIZE_I32);

        for chunk_coord in Coord::between_inclusive(min_chunk, max_chunk) {
            let chunk = self.chunks.at(chunk_coord);

            for chest in chunk.chests() {
                chest.draw(timestamp, player, sprites);
            }
        }
    }
}

pub struct RenderRegion {
    bottom_buffer: Vec<EntityRendererSettings>,
    top_buffer: Vec<EntityRendererSettings>,
    overlay_buffer: Vec<EntityRendererSettings>,
}

impl RenderRegion {
    pub fn new(map: &Map, min: Coord) -> Self {
        let mut bottom_buffer = Vec::new();
        let mut top_buffer = Vec::new();
        let mut overlay_buffer = Vec::new();

        let max = min + RENDER_REGION_SIZE;

        for y in min.y()..max.y() {
            for x in min.x()..max.x() {
                Self::draw_tile(map, &mut bottom_buffer, coord(x, y));
            }
        }

        for y in min.y()..max.y() {
            for x in min.x()..max.x() {
                Self::draw_shadows(map, &mut overlay_buffer, coord(x, y));
            }
        }

        let bounds = rect(min.start(), Vec2::one() * RENDER_REGION_SIZE as f64);

        for spawner in map.spawners_in(bounds) {
            top_buffer.push(EntityRendererSettings {
                transform: Mat3::transform(spawner, vec2(0.5, 0.5)),
                uv_transform: texture::spawner(),
            });
        }

        for decoration in map.decorations_in(bounds) {
            let texture_set = texture::floor_decoration();
            let texture_index = hash32_vec(decoration) as usize % texture_set.len();
            let texture = texture_set[texture_index];

            top_buffer.push(EntityRendererSettings {
                transform: Mat3::transform(decoration, vec2(0.5, 0.5)),
                uv_transform: texture,
            });
        }

        Self {
            bottom_buffer,
            top_buffer,
            overlay_buffer,
        }
    }

    pub fn draw_bottom(&self, buffer: &mut DrawBuffer) {
        for entity in &self.bottom_buffer {
            buffer.push(*entity);
        }
    }

    pub fn draw_top(&self, buffer: &mut DrawBuffer) {
        for entity in &self.top_buffer {
            buffer.push(*entity);
        }
    }

    pub fn draw_overlay(&self, buffer: &mut DrawBuffer) {
        for entity in &self.overlay_buffer {
            buffer.push(*entity);
        }
    }

    fn draw_tile(map: &Map, buffer: &mut Vec<EntityRendererSettings>, coord: Coord) {
        let tile = map.at(coord);

        match tile {
            Tile::Floor => {
                Self::draw_floor(buffer, coord);
            }
            Tile::Wall => {
                Self::draw_wall(map, buffer, coord);
            }
            Tile::Barrier => {
                Self::draw_floor(buffer, coord);
                Self::draw_texture_set(buffer, coord, texture::barrier());
            }
        }
    }

    fn draw_floor(buffer: &mut Vec<EntityRendererSettings>, coord: Coord) {
        let rotation = (hash32(&coord) % 4) as f64 * PI / 2.0;
        let transform = Mat3::rotation(rotation);

        Self::draw_texture_set_transformed(buffer, coord, transform, texture::floor());
    }

    fn draw_shadows(map: &Map, buffer: &mut Vec<EntityRendererSettings>, coord: Coord) {
        if Self::casts_shadow(map, coord) {
            return;
        }

        let top = Self::casts_shadow(map, coord.top());
        let left = Self::casts_shadow(map, coord.left());
        let bottom = Self::casts_shadow(map, coord.bottom());
        let right = Self::casts_shadow(map, coord.right());

        if top {
            Self::draw_shadow(buffer, coord, 0.0);
        }

        if left {
            Self::draw_shadow(buffer, coord, 0.5 * PI);
        }

        if bottom {
            Self::draw_shadow(buffer, coord, PI);
        }

        if right {
            Self::draw_shadow(buffer, coord, 1.5 * PI);
        }

        if !top && !left && Self::casts_shadow(map, coord.top_left()) {
            Self::draw_shadow_corner(buffer, coord, 0.0);
        }

        if !bottom && !left && Self::casts_shadow(map, coord.bottom_left()) {
            Self::draw_shadow_corner(buffer, coord, 0.5 * PI);
        }

        if !bottom && !right && Self::casts_shadow(map, coord.bottom_right()) {
            Self::draw_shadow_corner(buffer, coord, PI);
        }

        if !top && !right && Self::casts_shadow(map, coord.top_right()) {
            Self::draw_shadow_corner(buffer, coord, 1.5 * PI);
        }
    }

    fn draw_shadow(buffer: &mut Vec<EntityRendererSettings>, coord: Coord, rotation: f64) {
        let transform = Mat3::rotation(rotation);
        Self::draw_texture_transformed(buffer, coord, transform, texture::shadow());
    }

    fn draw_shadow_corner(buffer: &mut Vec<EntityRendererSettings>, coord: Coord, rotation: f64) {
        let transform = Mat3::rotation(rotation);
        Self::draw_texture_transformed(buffer, coord, transform, texture::shadow_corner());
    }

    fn casts_shadow(map: &Map, coord: Coord) -> bool {
        match map.at(coord) {
            Tile::Wall => true,
            Tile::Floor | Tile::Barrier => false,
        }
    }

    fn draw_wall(map: &Map, buffer: &mut Vec<EntityRendererSettings>, coord: Coord) {
        struct Report {
            top: bool,
            bottom: bool,
            left: bool,
            right: bool,
            top_left: bool,
            bottom_left: bool,
            top_right: bool,
            bottom_right: bool,
        }

        let report = Report {
            top: Self::is_non_wall(map, coord.top()),
            bottom: Self::is_non_wall(map, coord.bottom()),
            left: Self::is_non_wall(map, coord.left()),
            right: Self::is_non_wall(map, coord.right()),
            top_left: Self::is_non_wall(map, coord.top_left()),
            bottom_left: Self::is_non_wall(map, coord.bottom_left()),
            top_right: Self::is_non_wall(map, coord.top_right()),
            bottom_right: Self::is_non_wall(map, coord.bottom_right()),
        };

        if !report.top && hash32(&coord) % 3 == 0 {
            Self::draw_texture_set(buffer, coord, texture::space_decoration());
        }

        match report {
            Report { bottom: true, .. } => {
                Self::draw_texture_set(buffer, coord, texture::wall_top());
                return;
            }
            Report {
                top: true,
                left,
                right,
                bottom_left,
                bottom_right,
                ..
            } if (left || bottom_left) && (right || bottom_right) => {
                Self::draw_texture_set(buffer, coord, texture::wall_bottom_left_right());
                return;
            }
            Report {
                top: true,
                left,
                bottom_left,
                ..
            } if left || bottom_left => {
                Self::draw_texture_set(buffer, coord, texture::wall_bottom_right());
                return;
            }
            Report {
                top: true,
                right,
                bottom_right,
                ..
            } if right || bottom_right => {
                Self::draw_texture_set(buffer, coord, texture::wall_bottom_left());
                return;
            }
            Report { top: true, .. } => {
                Self::draw_texture_set(buffer, coord, texture::wall_bottom());
                return;
            }
            _ => {}
        };

        match report {
            Report { left: true, .. }
            | Report {
                top_left: true,
                bottom_left: true,
                ..
            } => {
                Self::draw_texture_set(buffer, coord, texture::wall_right());
            }
            Report { top_left: true, .. } => {
                Self::draw_texture(buffer, coord, texture::wall_bottom_right_corner());
            }
            Report {
                bottom_left: true, ..
            } => {
                Self::draw_texture(buffer, coord, texture::wall_top_right_corner());
            }
            _ => {}
        }

        match report {
            Report { right: true, .. }
            | Report {
                top_right: true,
                bottom_right: true,
                ..
            } => {
                Self::draw_texture_set(buffer, coord, texture::wall_left());
            }
            Report {
                top_right: true, ..
            } => {
                Self::draw_texture(buffer, coord, texture::wall_bottom_left_corner());
            }
            Report {
                bottom_right: true, ..
            } => {
                Self::draw_texture(buffer, coord, texture::wall_top_left_corner());
            }
            _ => {}
        }
    }

    fn is_non_wall(map: &Map, coord: Coord) -> bool {
        map.at(coord) != Tile::Wall
    }

    fn draw_texture_transformed(
        buffer: &mut Vec<EntityRendererSettings>,
        coord: Coord,
        transform: Mat3,
        texture: Mat3,
    ) {
        buffer.push(EntityRendererSettings {
            transform: Mat3::translation(coord.center()) * transform,
            uv_transform: texture,
        });
    }

    fn draw_texture(buffer: &mut Vec<EntityRendererSettings>, coord: Coord, texture: Mat3) {
        Self::draw_texture_transformed(buffer, coord, Mat3::identity(), texture)
    }

    fn draw_texture_set_transformed<const N: usize>(
        buffer: &mut Vec<EntityRendererSettings>,
        coord: Coord,
        transform: Mat3,
        texture_set: [Mat3; N],
    ) {
        let texture = texture_set[hash32(&coord) as usize % N];
        Self::draw_texture_transformed(buffer, coord, transform, texture);
    }

    fn draw_texture_set<const N: usize>(
        buffer: &mut Vec<EntityRendererSettings>,
        coord: Coord,
        texture_set: [Mat3; N],
    ) {
        Self::draw_texture_set_transformed(buffer, coord, Mat3::identity(), texture_set);
    }
}
