use specs::{Component, VecStorage};

use crate::util::rect::Rect;

#[derive(Copy, Clone)]
pub struct Bounds(pub Rect);

impl Component for Bounds {
    type Storage = VecStorage<Bounds>;
}
