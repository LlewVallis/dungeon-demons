use std::cell::{RefCell, UnsafeCell};

use float_ord::FloatOrd;
use fxhash::FxHashMap;
use pathfinding::prelude::astar;

use crate::map::chest::Chest;
use chunk::{Chunk, CHUNK_SIZE_I32};
use draw::{RenderRegion, RENDER_REGION_SIZE};

use crate::util::coord::Coord;
use crate::util::rect::Rect;
use crate::util::vector::Vec2;

pub mod chest;
mod chunk;
pub mod draw;
mod generation;

pub struct Map {
    chunks: ChunkStorage,
    render_regions: RefCell<FxHashMap<(i32, i32), RenderRegion>>,
}

impl Map {
    pub fn new(seed: u32) -> Self {
        Self {
            chunks: ChunkStorage::new(seed),
            render_regions: RefCell::new(FxHashMap::default()),
        }
    }

    pub fn at(&self, coord: Coord) -> Tile {
        let offset = coord.offset(CHUNK_SIZE_I32 / 2, CHUNK_SIZE_I32 / 2);
        let (chunk_coord, local_coord) = offset.chunk(CHUNK_SIZE_I32);

        let chunk = self.chunks.at(chunk_coord);
        chunk.at(local_coord)
    }

    pub fn set(&mut self, coord: Coord, tile: Tile) {
        let offset = coord.offset(CHUNK_SIZE_I32 / 2, CHUNK_SIZE_I32 / 2);
        let (chunk_coord, local_coord) = offset.chunk(CHUNK_SIZE_I32);

        let chunk = self.chunks.at_mut(chunk_coord);
        *chunk.at_mut(local_coord) = tile;

        let render_region_x = coord.x().div_euclid(RENDER_REGION_SIZE);
        let render_region_y = coord.y().div_euclid(RENDER_REGION_SIZE);

        self.render_regions
            .borrow_mut()
            .remove(&(render_region_x, render_region_y));
    }

    fn chunk_coords_in(&self, rect: Rect) -> impl Iterator<Item = Coord> {
        let min_coord = rect.min().coord();
        let max_coord = rect.max().coord();

        let min_chunk = (min_coord + CHUNK_SIZE_I32 / 2).div_euclid(CHUNK_SIZE_I32);
        let max_chunk = (max_coord + CHUNK_SIZE_I32 / 2).div_euclid(CHUNK_SIZE_I32);

        Coord::between_inclusive(min_chunk, max_chunk)
    }

    fn chunks_in(&self, rect: Rect) -> impl Iterator<Item = &Chunk> {
        self.chunk_coords_in(rect)
            .map(|coord| self.chunks.at(coord))
    }

    fn chunks_in_mut(&mut self, rect: Rect) -> impl Iterator<Item = &mut Chunk> {
        self.chunk_coords_in(rect).map(|coord| {
            let chunk = self.chunks.at_mut(coord);
            unsafe { &mut *(chunk as *mut _) }
        })
    }

    pub fn spawners_in(&self, rect: Rect) -> impl Iterator<Item = Vec2> + '_ {
        self.chunks_in(rect)
            .flat_map(|chunk| chunk.spawners())
            .filter(move |position| rect.contains(*position))
    }

    pub fn chests_in(&self, rect: Rect) -> impl Iterator<Item = &Chest> {
        self.chunks_in(rect)
            .flat_map(|chunk| chunk.chests())
            .filter(move |chest| rect.contains(chest.position()))
    }

    pub fn chests_in_mut(&mut self, rect: Rect) -> impl Iterator<Item = &mut Chest> {
        self.chunks_in_mut(rect)
            .flat_map(|chunk| chunk.chests_mut())
            .filter(move |chest| rect.contains(chest.position()))
    }

    pub fn decorations_in(&self, rect: Rect) -> impl Iterator<Item = Vec2> + '_ {
        self.chunks_in(rect)
            .flat_map(|chunk| chunk.decorations())
            .filter(move |position| rect.contains(*position))
    }

    pub fn pathfind<T, I>(
        &self,
        start: Coord,
        targets: T,
        threshold: f64,
    ) -> Option<(Vec<Coord>, usize)>
    where
        T: Fn() -> I,
        I: Iterator<Item = Coord>,
    {
        if targets().next().is_none() {
            return None;
        }

        let in_range = |coord: Coord| {
            let distance_squared = targets()
                .map(|target| {
                    let dist = coord.center().distance_squared(target.center());
                    dist * 10.0
                })
                .min_by_key(|value| FloatOrd(*value))
                .unwrap();

            distance_squared <= threshold * threshold
        };

        let successors = |coord: &Coord| {
            let directs = [coord.top(), coord.left(), coord.right(), coord.bottom()];

            let directs = directs
                .into_iter()
                .filter(|coord| self.at(*coord).is_walkable())
                .map(|coord| (coord, 10));

            let diagonals = [
                [coord.top_right(), coord.top(), coord.right()],
                [coord.bottom_right(), coord.bottom(), coord.right()],
                [coord.bottom_left(), coord.bottom(), coord.left()],
                [coord.top_left(), coord.top(), coord.left()],
            ];

            let diagonals = diagonals
                .into_iter()
                .filter(|coords| coords.iter().all(|coord| self.at(*coord).is_walkable()))
                .map(|[coord, _, _]| (coord, 14));

            directs
                .chain(diagonals)
                .filter(|(coord, _)| in_range(*coord))
        };

        let heuristic = |coord: &Coord| {
            targets()
                .map(|target| {
                    let dist = coord.center().distance(target.center());
                    (dist * 10.0) as u32
                })
                .min()
                .unwrap()
        };

        let success = |coord: &Coord| targets().any(|target| target == *coord);

        let result = astar(&start, successors, heuristic, success);
        result.map(|(path, cost)| (path, cost.div_ceil(10) as usize))
    }
}

const CHUNK_LOOKUP_CACHE_SIZE: usize = 2;

struct ChunkStorage {
    seed: u32,
    chunks: UnsafeCell<FxHashMap<Coord, Box<Chunk>>>,
    chunk_lookup_cache: UnsafeCell<[Option<(Coord, *mut Chunk)>; CHUNK_LOOKUP_CACHE_SIZE]>,
}

impl ChunkStorage {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            chunks: UnsafeCell::new(FxHashMap::default()),
            chunk_lookup_cache: UnsafeCell::new([None; CHUNK_LOOKUP_CACHE_SIZE]),
        }
    }

    pub fn at(&self, coord: Coord) -> &Chunk {
        unsafe { &*self.lookup(coord) }
    }

    pub fn at_mut(&mut self, coord: Coord) -> &mut Chunk {
        unsafe { &mut *self.lookup(coord) }
    }

    fn lookup(&self, coord: Coord) -> *mut Chunk {
        let cache = unsafe { &mut *self.chunk_lookup_cache.get() };

        for line in *cache {
            if let Some((cache_coord, chunk)) = line {
                if cache_coord == coord {
                    return chunk;
                }
            }
        }

        let chunks = unsafe { &mut *self.chunks.get() };

        let chunk = chunks
            .entry(coord)
            .or_insert_with(|| box Chunk::new(coord, self.seed));
        let chunk = chunk.as_mut() as *mut _;

        for i in 1..CHUNK_LOOKUP_CACHE_SIZE {
            cache[i] = cache[i - 1];
        }

        cache[0] = Some((coord, chunk));

        chunk
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Tile {
    Floor,
    Wall,
    Barrier,
}

impl Tile {
    pub fn is_walkable(&self) -> bool {
        match *self {
            Tile::Floor => true,
            Tile::Wall | Tile::Barrier => false,
        }
    }
}
