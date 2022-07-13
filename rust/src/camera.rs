use crate::util::coord::Coord;
use crate::util::rect::Rect;
use crate::util::vector::Vec2;
use crate::Mat3;

#[derive(Copy, Clone)]
pub struct Camera {
    bounds: Rect,
}

impl Camera {
    pub fn new(bounds: Rect) -> Self {
        Self { bounds }
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    pub fn vmin_ratio(&self) -> f64 {
        self.aspect_ratio().min(1.0)
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.bounds.size.x() / self.bounds.size.y()
    }

    pub fn view(&self) -> Mat3 {
        Mat3::scale(Vec2::one() / self.bounds.size.y() * 2.0)
            * Mat3::translation(-self.bounds.center())
    }

    pub fn min_coord(&self) -> Coord {
        self.bounds.min().coord()
    }

    pub fn max_coord(&self) -> Coord {
        self.bounds.max().coord()
    }
}
