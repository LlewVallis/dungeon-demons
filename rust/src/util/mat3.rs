use crate::util::vector::{vec2, vec3, Vec2, Vec3};
use std::ops::{Mul, MulAssign};

#[derive(Copy, Clone, PartialEq)]
pub struct Mat3 {
    data: [[f64; 3]; 3],
}

impl Mat3 {
    pub fn new(data: [[f64; 3]; 3]) -> Self {
        Self { data }
    }

    pub fn identity() -> Self {
        Self::new([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]])
    }

    pub fn scale(vector: Vec2) -> Self {
        Self::new([
            [vector.x(), 0.0, 0.0],
            [0.0, vector.y(), 0.0],
            [0.0, 0.0, 1.0],
        ])
    }

    pub fn translation(vector: Vec2) -> Self {
        Self::new([
            [1.0, 0.0, vector.x()],
            [0.0, 1.0, vector.y()],
            [0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation(radians: f64) -> Self {
        let (sin, cos) = radians.sin_cos();

        Self::new([[cos, -sin, 0.0], [sin, cos, 0.0], [0.0, 0.0, 1.0]])
    }

    pub fn transform(translation: Vec2, scale: Vec2) -> Self {
        Self::translation(translation) * Self::scale(scale)
    }

    pub fn determinant(&self) -> f64 {
        self.m11() * (self.m22() * self.m33() - self.m32() * self.m23())
            - self.m12() * (self.m12() * self.m33() - self.m32() * self.m13())
            + self.m13() * (self.m12() * self.m23() - self.m22() * self.m13())
    }

    pub fn inverse(&self) -> Self {
        let x = 1.0 / self.determinant();

        return Self::new([
            [
                (self.m22() * self.m33() - self.m32() * self.m23()) * x,
                (self.m31() * self.m23() - self.m21() * self.m33()) * x,
                (self.m21() * self.m32() - self.m31() * self.m22()) * x,
            ],
            [
                (self.m32() * self.m13() - self.m12() * self.m33()) * x,
                (self.m11() * self.m33() - self.m31() * self.m13()) * x,
                (self.m31() * self.m12() - self.m11() * self.m32()) * x,
            ],
            [
                (self.m12() * self.m23() - self.m22() * self.m13()) * x,
                (self.m21() * self.m13() - self.m11() * self.m23()) * x,
                (self.m11() * self.m22() - self.m21() * self.m12()) * x,
            ],
        ]);
    }

    fn at(&self, x: usize, y: usize) -> f64 {
        self.data[y][x]
    }

    pub fn m11(&self) -> f64 {
        self.at(0, 0)
    }

    pub fn m21(&self) -> f64 {
        self.at(1, 0)
    }

    pub fn m31(&self) -> f64 {
        self.at(2, 0)
    }

    pub fn m12(&self) -> f64 {
        self.at(0, 1)
    }

    pub fn m22(&self) -> f64 {
        self.at(1, 1)
    }

    pub fn m32(&self) -> f64 {
        self.at(2, 1)
    }

    pub fn m13(&self) -> f64 {
        self.at(0, 2)
    }

    pub fn m23(&self) -> f64 {
        self.at(1, 2)
    }

    pub fn m33(&self) -> f64 {
        self.at(2, 2)
    }
}

impl Mul for Mat3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self::new([
            [
                self.m11() * rhs.m11() + self.m21() * rhs.m12() + self.m31() * rhs.m13(),
                self.m11() * rhs.m21() + self.m21() * rhs.m22() + self.m31() * rhs.m23(),
                self.m11() * rhs.m31() + self.m21() * rhs.m32() + self.m31() * rhs.m33(),
            ],
            [
                self.m12() * rhs.m11() + self.m22() * rhs.m12() + self.m32() * rhs.m13(),
                self.m12() * rhs.m21() + self.m22() * rhs.m22() + self.m32() * rhs.m23(),
                self.m12() * rhs.m31() + self.m22() * rhs.m32() + self.m32() * rhs.m33(),
            ],
            [
                self.m13() * rhs.m11() + self.m23() * rhs.m12() + self.m33() * rhs.m13(),
                self.m13() * rhs.m21() + self.m23() * rhs.m22() + self.m33() * rhs.m23(),
                self.m13() * rhs.m31() + self.m23() * rhs.m32() + self.m33() * rhs.m33(),
            ],
        ])
    }
}

impl MulAssign for Mat3 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Mul<Vec2> for Mat3 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Vec2 {
        vec2(
            self.m11() * rhs.x() + self.m21() * rhs.y() + self.m31(),
            self.m12() * rhs.x() + self.m22() * rhs.y() + self.m32(),
        )
    }
}

impl Mul<Vec3> for Mat3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        vec3(
            self.m11() * rhs.x() + self.m21() * rhs.y() + self.m31() * rhs.z(),
            self.m12() * rhs.x() + self.m22() * rhs.y() + self.m32() * rhs.z(),
            self.m13() * rhs.x() + self.m23() * rhs.y() + self.m33() * rhs.z(),
        )
    }
}
