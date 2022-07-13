use crate::util::rect::{rect, Rect};
use crate::util::vector::Vec2;
use crate::vec2;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

pub const fn coord(x: i32, y: i32) -> Coord {
    Coord { x, y }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct Coord {
    x: i32,
    y: i32,
}

impl Coord {
    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn distance(self, other: Coord) -> u32 {
        i32::abs_diff(self.x, other.x) + i32::abs_diff(self.y, other.y)
    }

    pub fn euclidean_distance(self, other: Coord) -> f64 {
        self.center().distance(other.center())
    }

    pub fn div_euclid(&self, value: i32) -> Coord {
        coord(self.x().div_euclid(value), self.y().div_euclid(value))
    }

    pub fn rem_euclid(&self, value: i32) -> Coord {
        coord(self.x().rem_euclid(value), self.y().rem_euclid(value))
    }

    pub fn chunk(&self, size: i32) -> (Coord, Coord) {
        (self.div_euclid(size), self.rem_euclid(size))
    }

    pub fn min(self, other: Coord) -> Coord {
        coord(i32::min(self.x, other.x), i32::min(self.y, other.y))
    }

    pub fn max(self, other: Coord) -> Coord {
        coord(i32::max(self.x, other.x), i32::max(self.y, other.y))
    }

    pub fn between_inclusive(min: Coord, max: Coord) -> impl Iterator<Item = Coord> {
        (min.y..=max.y).flat_map(move |y| (min.x..=max.x).map(move |x| coord(x, y)))
    }

    pub fn is_between_inclusive(&self, min: Coord, max: Coord) -> bool {
        self.x() >= min.x() && self.x() <= max.x() && self.y() >= min.y() && self.y() <= max.y()
    }

    pub fn bounds(&self) -> Rect {
        rect(self.start(), Vec2::one())
    }

    pub fn center(&self) -> Vec2 {
        self.start() + 0.5
    }

    pub fn start(&self) -> Vec2 {
        vec2(self.x as f64, self.y as f64)
    }

    pub fn end(&self) -> Vec2 {
        self.start() + 1.0
    }

    pub fn offset(&self, x: i32, y: i32) -> Coord {
        coord(self.x + x, self.y + y)
    }

    pub fn directly_adjacent(&self) -> [Coord; 4] {
        [self.top(), self.bottom(), self.left(), self.right()]
    }

    pub fn bottom_left(&self) -> Coord {
        self.offset(-1, -1)
    }

    pub fn bottom(&self) -> Coord {
        self.offset(0, -1)
    }

    pub fn bottom_right(&self) -> Coord {
        self.offset(1, -1)
    }

    pub fn left(&self) -> Coord {
        self.offset(-1, 0)
    }

    pub fn right(&self) -> Coord {
        self.offset(1, 0)
    }

    pub fn top_left(&self) -> Coord {
        self.offset(-1, 1)
    }

    pub fn top(&self) -> Coord {
        self.offset(0, 1)
    }

    pub fn top_right(&self) -> Coord {
        self.offset(1, 1)
    }
}

impl Add for Coord {
    type Output = Coord;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign for Coord {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Coord {
    type Output = Coord;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl SubAssign for Coord {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Add<i32> for Coord {
    type Output = Coord;

    fn add(self, rhs: i32) -> Self::Output {
        coord(self.x + rhs, self.y + rhs)
    }
}

impl AddAssign<i32> for Coord {
    fn add_assign(&mut self, rhs: i32) {
        *self = *self + rhs;
    }
}

impl Sub<i32> for Coord {
    type Output = Coord;

    fn sub(self, rhs: i32) -> Self::Output {
        coord(self.x - rhs, self.y - rhs)
    }
}

impl SubAssign<i32> for Coord {
    fn sub_assign(&mut self, rhs: i32) {
        *self = *self - rhs;
    }
}

impl Mul<i32> for Coord {
    type Output = Coord;

    fn mul(self, rhs: i32) -> Self::Output {
        coord(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<i32> for Coord {
    fn mul_assign(&mut self, rhs: i32) {
        *self = *self * rhs;
    }
}

impl Debug for Coord {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_list().entry(&self.x).entry(&self.y).finish()
    }
}
