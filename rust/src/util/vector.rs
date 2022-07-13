use crate::util::coord::{coord, Coord};
use crate::Mat3;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

pub type Vec2 = Vector<2>;
pub type Vec3 = Vector<3>;

pub const fn vec2(x: f64, y: f64) -> Vec2 {
    Vector { components: [x, y] }
}

pub const fn vec3(x: f64, y: f64, z: f64) -> Vec3 {
    Vector {
        components: [x, y, z],
    }
}

impl Vector<2> {
    pub fn x(&self) -> f64 {
        self[0]
    }

    pub fn x_mut(&mut self) -> &mut f64 {
        &mut self[0]
    }

    pub fn y(&self) -> f64 {
        self[1]
    }

    pub fn y_mut(&mut self) -> &mut f64 {
        &mut self[1]
    }

    pub fn abs_x(&self) -> Self {
        vec2(self.x().abs(), self.y())
    }

    pub fn abs_y(&self) -> Self {
        vec2(self.x(), self.y().abs())
    }

    pub fn coord(&self) -> Coord {
        coord(self.x().floor() as i32, self.y().floor() as i32)
    }

    pub fn angle(&self) -> f64 {
        f64::atan2(self.y(), self.x())
    }

    pub fn rotate(&self, angle: f64) -> Self {
        Mat3::rotation(angle) * *self
    }

    pub fn top() -> Self {
        vec2(0.0, 1.0)
    }

    pub fn bottom() -> Self {
        vec2(0.0, -1.0)
    }

    pub fn left() -> Self {
        vec2(-1.0, 0.0)
    }

    pub fn right() -> Self {
        vec2(1.0, 0.0)
    }

    pub fn top_left() -> Self {
        vec2(-1.0, 1.0).normalize()
    }

    pub fn top_right() -> Self {
        vec2(1.0, 1.0).normalize()
    }

    pub fn bottom_left() -> Self {
        vec2(-1.0, -1.0).normalize()
    }

    pub fn bottom_right() -> Self {
        vec2(1.0, -1.0).normalize()
    }
}

impl Vector<3> {
    pub fn x(&self) -> f64 {
        self[0]
    }

    pub fn x_mut(&mut self) -> &mut f64 {
        &mut self[0]
    }

    pub fn y(&self) -> f64 {
        self[1]
    }

    pub fn y_mut(&mut self) -> &mut f64 {
        &mut self[1]
    }

    pub fn z(&self) -> f64 {
        self[2]
    }

    pub fn z_mut(&mut self) -> &mut f64 {
        &mut self[2]
    }

    pub fn abs_x(&self) -> Self {
        vec3(self.x().abs(), self.y(), self.z())
    }

    pub fn abs_y(&self) -> Self {
        vec3(self.x(), self.y().abs(), self.z())
    }

    pub fn abs_z(&self) -> Self {
        vec3(self.x(), self.y(), self.z().abs())
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq)]
pub struct Vector<const LENGTH: usize> {
    components: [f64; LENGTH],
}

impl<const LENGTH: usize> Vector<LENGTH> {
    pub fn zero() -> Self {
        Self {
            components: [0.0; LENGTH],
        }
    }

    pub fn one() -> Self {
        Self {
            components: [1.0; LENGTH],
        }
    }

    pub fn components(&self) -> [f64; LENGTH] {
        self.components
    }

    pub fn taxicab_length(&self) -> f64 {
        self.abs().components.iter().sum()
    }

    pub fn length_squared(&self) -> f64 {
        self.components.iter().map(|value| value * value).sum()
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn normalize_or(&self, default: Self) -> Self {
        let length = self.length();

        if length == 0.0 {
            default
        } else {
            *self / length
        }
    }

    pub fn normalize_or_zero(&self) -> Self {
        self.normalize_or(Self::zero())
    }

    pub fn normalize(&self) -> Self {
        *self / self.length()
    }

    pub fn is_zero(&self) -> bool {
        self.length_squared() == 0.0
    }

    pub fn taxicab_distance(self, other: Self) -> f64 {
        (self - other).taxicab_length()
    }

    pub fn distance_squared(self, other: Self) -> f64 {
        (self - other).length_squared()
    }

    pub fn distance(self, other: Self) -> f64 {
        self.distance_squared(other).sqrt()
    }

    pub fn dot(self, other: Self) -> f64 {
        (self * other).components.into_iter().sum()
    }

    pub fn abs(&self) -> Self {
        Self {
            components: self.components.map(f64::abs),
        }
    }

    pub fn min(self, other: Self) -> Self {
        Self::zip_with(self, other, f64::min)
    }

    pub fn max(self, other: Self) -> Self {
        Self::zip_with(self, other, f64::max)
    }

    pub fn zip_with(self, other: Self, mut zipper: impl FnMut(f64, f64) -> f64) -> Self {
        let components = self
            .components
            .zip(other.components)
            .map(|(a, b)| zipper(a, b));

        Self { components }
    }
}

impl<const LENGTH: usize> Index<usize> for Vector<LENGTH> {
    type Output = f64;

    fn index(&self, index: usize) -> &f64 {
        &self.components[index]
    }
}

impl<const LENGTH: usize> IndexMut<usize> for Vector<LENGTH> {
    fn index_mut(&mut self, index: usize) -> &mut f64 {
        &mut self.components[index]
    }
}

impl<const LENGTH: usize> Add for Vector<LENGTH> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl<const LENGTH: usize> AddAssign for Vector<LENGTH> {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..LENGTH {
            self[i] += rhs[i];
        }
    }
}

impl<const LENGTH: usize> Add<f64> for Vector<LENGTH> {
    type Output = Self;

    fn add(mut self, rhs: f64) -> Self {
        self += rhs;
        self
    }
}

impl<const LENGTH: usize> AddAssign<f64> for Vector<LENGTH> {
    fn add_assign(&mut self, rhs: f64) {
        for component in &mut self.components {
            *component += rhs;
        }
    }
}

impl<const LENGTH: usize> Sub for Vector<LENGTH> {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self {
        self -= rhs;
        self
    }
}

impl<const LENGTH: usize> SubAssign for Vector<LENGTH> {
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..LENGTH {
            self[i] -= rhs[i];
        }
    }
}

impl<const LENGTH: usize> Sub<f64> for Vector<LENGTH> {
    type Output = Self;

    fn sub(mut self, rhs: f64) -> Self {
        self -= rhs;
        self
    }
}

impl<const LENGTH: usize> SubAssign<f64> for Vector<LENGTH> {
    fn sub_assign(&mut self, rhs: f64) {
        for component in &mut self.components {
            *component -= rhs;
        }
    }
}

impl<const LENGTH: usize> Mul for Vector<LENGTH> {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self {
        self *= rhs;
        self
    }
}

impl<const LENGTH: usize> MulAssign for Vector<LENGTH> {
    fn mul_assign(&mut self, rhs: Self) {
        for i in 0..LENGTH {
            self[i] *= rhs[i];
        }
    }
}

impl<const LENGTH: usize> Mul<f64> for Vector<LENGTH> {
    type Output = Self;

    fn mul(mut self, rhs: f64) -> Self {
        self *= rhs;
        self
    }
}

impl<const LENGTH: usize> MulAssign<f64> for Vector<LENGTH> {
    fn mul_assign(&mut self, rhs: f64) {
        for component in &mut self.components {
            *component *= rhs;
        }
    }
}

impl<const LENGTH: usize> Div for Vector<LENGTH> {
    type Output = Self;

    fn div(mut self, rhs: Self) -> Self {
        self /= rhs;
        self
    }
}

impl<const LENGTH: usize> DivAssign for Vector<LENGTH> {
    fn div_assign(&mut self, rhs: Self) {
        for i in 0..LENGTH {
            self[i] /= rhs[i];
        }
    }
}

impl<const LENGTH: usize> Div<f64> for Vector<LENGTH> {
    type Output = Self;

    fn div(mut self, rhs: f64) -> Self {
        self /= rhs;
        self
    }
}

impl<const LENGTH: usize> DivAssign<f64> for Vector<LENGTH> {
    fn div_assign(&mut self, rhs: f64) {
        for component in &mut self.components {
            *component /= rhs;
        }
    }
}

impl<const LENGTH: usize> Neg for Vector<LENGTH> {
    type Output = Self;

    fn neg(mut self) -> Self {
        for component in &mut self.components {
            *component = -*component;
        }

        self
    }
}

impl<const LENGTH: usize> Debug for Vector<LENGTH> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_list().entries(&self.components).finish()
    }
}
