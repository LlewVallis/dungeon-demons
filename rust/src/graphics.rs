use crate::camera::Camera;
use crate::components::health::Health;
use crate::ecs::ReadControlledPlayerStorage;
use crate::game::Timestamp;
use crate::util::vector::{vec3, Vec3, Vector};
use crate::Mat3;
use specs::prelude::*;

pub struct DrawBuffer {
    bytes: Vec<u8>,
}

impl DrawBuffer {
    pub fn new() -> DrawBuffer {
        let mut result = Self { bytes: Vec::new() };

        result.reset();

        result
    }

    pub fn push_frame_settings(&mut self, frame_settings: FrameSettings) {
        self.write_matrix(frame_settings.view);
        self.write_vector(frame_settings.vignette_color);
        self.write_float(frame_settings.vignette_scale);
    }

    pub fn push(&mut self, entity: EntityRendererSettings) {
        self.increment_size();
        self.write_matrix(entity.transform);
        self.write_matrix(entity.uv_transform);
    }

    fn write_matrix(&mut self, matrix: Mat3) {
        self.write_float(matrix.m11());
        self.write_float(matrix.m21());
        self.write_float(matrix.m31());
        self.write_float(matrix.m12());
        self.write_float(matrix.m22());
        self.write_float(matrix.m32());
        self.write_float(matrix.m13());
        self.write_float(matrix.m23());
        self.write_float(matrix.m33());
    }

    fn write_vector<const N: usize>(&mut self, vector: Vector<N>) {
        for component in vector.components() {
            self.write_float(component);
        }
    }

    fn write_float(&mut self, value: f64) {
        self.bytes.extend(f32::to_le_bytes(value as f32));
    }

    fn increment_size(&mut self) {
        let (bytes, _) = self.bytes.split_array_mut::<4>();
        let size = u32::from_le_bytes(*bytes);
        *bytes = u32::to_le_bytes(size + 1);
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.bytes.as_ptr()
    }

    pub fn reset(&mut self) {
        self.bytes.clear();
        self.bytes.extend([0, 0, 0, 0]);
    }
}

#[derive(Copy, Clone)]
pub struct FrameSettings {
    pub view: Mat3,
    pub vignette_color: Vec3,
    pub vignette_scale: f64,
}

#[derive(Copy, Clone)]
pub struct EntityRendererSettings {
    pub transform: Mat3,
    pub uv_transform: Mat3,
}

pub struct ResetDrawBuffer;

impl ResetDrawBuffer {
    fn vignette(health: f64, timestamp: f64) -> (Vec3, f64) {
        let scaled_time = 5.0 * timestamp;

        let base_red = 0.4 + (scaled_time.sin() / 2.0 + 0.5).powi(2) * 0.1;
        let base_color = vec3(base_red, base_red / 5.0, base_red / 5.0);

        let color = base_color * (1.0 - health).sqrt();
        let scale = 10.0 + 70.0 * health.powi(2);

        (color, scale)
    }
}

#[derive(SystemData)]
pub struct ResetDrawBufferData<'a> {
    timestamp: ReadExpect<'a, Timestamp>,
    camera: ReadExpect<'a, Camera>,
    buffer: WriteExpect<'a, DrawBuffer>,
    health: ReadControlledPlayerStorage<'a, Health>,
}

impl<'a> System<'a> for ResetDrawBuffer {
    type SystemData = ResetDrawBufferData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let health = data
            .health
            .value()
            .map(|healt| healt.remaining_relative())
            .unwrap_or(1.0);
        let (vignette_color, vignette_scale) = Self::vignette(health, data.timestamp.0);

        let frame = FrameSettings {
            vignette_color,
            vignette_scale,
            view: data.camera.view(),
        };

        data.buffer.reset();
        data.buffer.push_frame_settings(frame);
    }
}

#[allow(dead_code)]
pub mod texture {
    use crate::{vec2, Mat3};

    const TEXTURE_PIXELS: f64 = 32.0;
    const SHEET_PIXELS: f64 = 512.0;

    macro_rules! texture {
        ($name:ident, $x:literal, $y:literal, $width:literal, $height:literal) => {
            pub fn $name() -> Mat3 {
                Mat3::transform(
                    vec2($x, $y) * TEXTURE_PIXELS / SHEET_PIXELS,
                    vec2($width, $height) * TEXTURE_PIXELS / SHEET_PIXELS,
                )
            }
        };
        ($name:ident, $x:literal, $y:literal) => {
            texture!($name, $x, $y, 1.0, 1.0);
        };
    }

    macro_rules! texture_set {
        ($name:ident, $count:literal, $x:literal, $y:literal, $width:literal, $height:literal) => {
            pub fn $name() -> [Mat3; $count] {
                let mut result = [Mat3::identity(); $count];

                for i in 0..$count {
                    result[i] = Mat3::transform(
                        vec2($x + $width * (i as f64), $y) * TEXTURE_PIXELS / SHEET_PIXELS,
                        vec2($width, $height) * TEXTURE_PIXELS / SHEET_PIXELS,
                    )
                }

                result
            }
        };
        ($name:ident, $count:literal, $x:literal, $y:literal) => {
            texture_set!($name, $count, $x, $y, 1.0, 1.0);
        };
    }

    texture_set!(wall_top, 8, 0.0, 0.0);

    texture!(wall_top_left_corner, 0.0, 1.0);
    texture_set!(wall_left, 4, 1.0, 1.0);
    texture!(wall_bottom_left_corner, 5.0, 1.0);

    texture!(wall_top_right_corner, 0.0, 2.0);
    texture_set!(wall_right, 4, 1.0, 2.0);
    texture!(wall_bottom_right_corner, 5.0, 2.0);

    texture_set!(wall_bottom, 4, 0.0, 3.0);

    texture_set!(wall_bottom_right, 2, 0.0, 4.0);
    texture_set!(wall_bottom_left, 2, 2.0, 4.0);
    texture_set!(wall_bottom_left_right, 2, 4.0, 4.0);

    texture_set!(floor, 6, 0.0, 5.0);
    texture!(shadow, 6.0, 1.0);
    texture!(shadow_corner, 7.0, 1.0);

    texture_set!(barrier, 4, 0.0, 6.0);

    texture_set!(floor_decoration, 2, 0.0, 7.0, 0.5, 0.5);
    texture_set!(space_decoration, 4, 8.0, 0.0);

    texture!(spawner, 1.0, 7.0, 0.5, 0.5);

    texture_set!(player_left, 4, 0.0, 7.5, 0.5, 0.5);
    texture_set!(player_right, 4, 2.0, 7.5, 0.5, 0.5);
    texture_set!(player_top_left, 4, 4.0, 7.5, 0.5, 0.5);
    texture_set!(player_top_right, 4, 6.0, 7.5, 0.5, 0.5);
    texture_set!(player_bottom_left, 4, 8.0, 7.5, 0.5, 0.5);
    texture_set!(player_bottom_right, 4, 10.0, 7.5, 0.5, 0.5);
    texture!(player_top_left_idle, 12.0, 7.5, 0.5, 0.5);
    texture!(player_top_right_idle, 12.5, 7.5, 0.5, 0.5);
    texture!(player_bottom_left_idle, 13.0, 7.5, 0.5, 0.5);
    texture!(player_bottom_right_idle, 13.5, 7.5, 0.5, 0.5);

    texture_set!(demon_left, 4, 0.0, 8.0, 0.5, 0.5);
    texture_set!(demon_right, 4, 2.0, 8.0, 0.5, 0.5);

    texture_set!(spawning_demon, 6, 4.0, 8.0, 0.5, 0.5);

    texture!(bullet, 0.0, 8.5, 0.25, 0.25);

    texture!(closed_chest, 1.5, 7.0, 0.5, 0.5);
    texture!(open_chest, 2.0, 7.0, 0.5, 0.5);

    texture!(pistol, 0.0, 9.0, 0.75, 0.75);
    texture!(lmg, 0.75, 9.0, 0.75, 0.75);
    texture!(shotgun, 1.5, 9.0, 0.75, 0.75);
    texture!(rifle, 2.25, 9.0, 0.75, 0.75);
    texture!(sniper, 3.0, 9.0, 0.75, 0.75);
    texture!(smg, 3.75, 9.0, 0.75, 0.75);

    texture!(gun_hud_background_inactive, 0.0, 10.0, 2.0, 2.0);
    texture!(gun_hud_background_active, 2.0, 10.0, 2.0, 2.0);
}
