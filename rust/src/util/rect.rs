use crate::{vec2, Mat3};
use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::util::coord::Coord;
use crate::util::vector::Vec2;

pub fn rect(position: Vec2, size: Vec2) -> Rect {
    Rect { position, size }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn focused(focus: Vec2, size: Vec2) -> Self {
        rect(focus - size / 2.0, size)
    }

    pub fn center(&self) -> Vec2 {
        self.position + self.size / 2.0
    }

    pub fn contains(&self, position: Vec2) -> bool {
        position.x() >= self.min().x()
            && position.x() < self.max().x()
            && position.y() >= self.min().y()
            && position.y() < self.max().y()
    }

    pub fn min(&self) -> Vec2 {
        self.position
    }

    pub fn max(&self) -> Vec2 {
        self.position + self.size
    }

    pub fn transform_aabb(&self, transform: Mat3) -> Self {
        let corners = [
            self.position,
            self.position + vec2(self.size.x(), 0.0),
            self.position + vec2(0.0, self.size.y()),
            self.position + self.size,
        ];

        let transformed_corners = corners.map(|corner| transform * corner);

        let mut min = Vec2::one() * f64::INFINITY;
        let mut max = Vec2::one() * f64::NEG_INFINITY;

        for corner in transformed_corners {
            min = min.min(corner);
            max = max.max(corner);
        }

        Self {
            position: min,
            size: max - min,
        }
    }

    pub fn touches(self, other: Self) -> bool {
        let dist = Self::signed_distance(self, other);
        dist.x() <= 0.0 && dist.y() <= 0.0
    }

    pub fn signed_distance(self, other: Self) -> Vec2 {
        let delta = self.center() - other.center();
        let total_radius = (self.size + other.size) / 2.0;
        delta.abs() - total_radius
    }

    pub fn euclidean_distance(self, other: Self) -> f64 {
        let dist = Self::signed_distance(self, other);
        let clamped_distance = Vec2::max(dist, Vec2::zero());
        clamped_distance.length()
    }

    pub fn normal_to(&self, other: Self) -> Vec2 {
        let dist = Self::signed_distance(*self, other);

        if dist.x() >= dist.y() {
            if self.center().x() <= other.center().x() {
                Vec2::right()
            } else {
                Vec2::left()
            }
        } else {
            if self.center().y() <= other.center().y() {
                Vec2::top()
            } else {
                Vec2::bottom()
            }
        }
    }

    pub fn expand(&self, amount: f64) -> Self {
        Self {
            position: self.position - amount,
            size: self.size + amount * 2.0,
        }
    }

    pub fn coords(&self) -> impl Iterator<Item = Coord> {
        let min = self.min().coord();
        let max = self.max().coord();
        Coord::between_inclusive(min, max)
    }
}

impl Add<Vec2> for Rect {
    type Output = Rect;

    fn add(self, rhs: Vec2) -> Self {
        rect(self.position + rhs, self.size)
    }
}

impl AddAssign<Vec2> for Rect {
    fn add_assign(&mut self, rhs: Vec2) {
        *self = *self + rhs;
    }
}

impl Sub<Vec2> for Rect {
    type Output = Rect;

    fn sub(self, rhs: Vec2) -> Self {
        rect(self.position - rhs, self.size)
    }
}

impl SubAssign<Vec2> for Rect {
    fn sub_assign(&mut self, rhs: Vec2) {
        *self = *self - rhs;
    }
}
