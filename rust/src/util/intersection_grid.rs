use std::hash::Hash;
use std::mem::MaybeUninit;

use crate::util::coord::{coord, Coord};
use fxhash::FxHashMap;

use crate::util::rect::Rect;

const RESOLUTION: usize = 25;
const SEGMENT_SIZE: usize = 3;

const RESOLUTION_I32: i32 = RESOLUTION as i32;
const SEGMENT_SIZE_I32: i32 = SEGMENT_SIZE as i32;

pub struct IntersectionGrid<K: Clone + Hash + Eq, V: Clone> {
    chunks: FxHashMap<Coord, Chunk<K, V>>,
}

impl<K: Clone + Hash + Eq, V: Clone> IntersectionGrid<K, V> {
    pub fn new() -> Self {
        Self {
            chunks: FxHashMap::default(),
        }
    }

    pub fn insert(&mut self, bounds: Rect, key: K, value: V) {
        let (min_segment, max_segment) = Self::segment_min_max(bounds);
        let (min_chunk, max_chunk) = Self::chunk_min_max(min_segment, max_segment);

        for y in min_chunk.y()..=max_chunk.y() {
            for x in min_chunk.x()..=max_chunk.x() {
                let chunk_coord = coord(x, y);

                let (local_min, local_max) =
                    Self::local_min_max(chunk_coord, min_segment, max_segment);

                let chunk = self.chunks.entry(chunk_coord).or_insert_with(Chunk::new);

                if chunk_coord == max_chunk {
                    chunk.insert(bounds, key, value, local_min, local_max);
                    return;
                } else {
                    chunk.insert(bounds, key.clone(), value.clone(), local_min, local_max);
                }
            }
        }
    }

    pub fn query(&self, bounds: Rect) -> FxHashMap<&K, (Rect, &V)> {
        let (min_segment, max_segment) = Self::segment_min_max(bounds);
        let (min_chunk, max_chunk) = Self::chunk_min_max(min_segment, max_segment);

        let mut results = FxHashMap::default();

        for y in min_chunk.y()..=max_chunk.y() {
            for x in min_chunk.x()..=max_chunk.x() {
                let chunk_coord = coord(x, y);

                let (local_min, local_max) =
                    Self::local_min_max(chunk_coord, min_segment, max_segment);

                if let Some(chunk) = self.chunks.get(&chunk_coord) {
                    chunk.query(bounds, local_min, local_max, &mut results);
                }
            }
        }

        results
    }

    fn segment_min_max(bounds: Rect) -> (Coord, Coord) {
        (
            bounds.min().coord().div_euclid(SEGMENT_SIZE_I32),
            bounds.max().coord().div_euclid(SEGMENT_SIZE_I32),
        )
    }

    fn chunk_min_max(min_segment: Coord, max_segment: Coord) -> (Coord, Coord) {
        (
            min_segment.div_euclid(RESOLUTION_I32),
            max_segment.div_euclid(RESOLUTION_I32),
        )
    }

    fn local_min_max(chunk_coord: Coord, min_segment: Coord, max_segment: Coord) -> (Coord, Coord) {
        let local_start = coord(0, 0);
        let local_end = coord(RESOLUTION_I32, RESOLUTION_I32) - 1;

        let chunk_offset = chunk_coord * RESOLUTION_I32;
        let local_min = Coord::max(min_segment - chunk_offset, local_start);
        let local_max = Coord::min(max_segment - chunk_offset, local_end);

        (local_min, local_max)
    }
}

struct Chunk<K: Clone + Hash + Eq, V: Clone> {
    values: Box<[[Segment<K, V>; RESOLUTION]; RESOLUTION]>,
}

impl<K: Clone + Hash + Eq, V: Clone> Chunk<K, V> {
    pub fn new() -> Self {
        let mut values = Box::new_uninit_slice(RESOLUTION * RESOLUTION);
        for value in values.iter_mut() {
            *value = MaybeUninit::new(Segment::<K, V>::new());
        }

        let ptr = Box::into_raw(values) as *mut _;
        let values = unsafe { Box::from_raw(ptr) };

        Self { values }
    }

    pub fn insert(&mut self, bounds: Rect, key: K, value: V, local_min: Coord, local_max: Coord) {
        for y in local_min.y()..=local_max.y() {
            for x in local_min.x()..=local_max.x() {
                let segment = &mut self.values[y as usize][x as usize];

                if coord(x, y) == local_max {
                    segment.insert(bounds, key, value);
                    return;
                } else {
                    segment.insert(bounds, key.clone(), value.clone());
                }
            }
        }
    }

    pub fn query<'a>(
        &'a self,
        bounds: Rect,
        local_min: Coord,
        local_max: Coord,
        results: &mut FxHashMap<&'a K, (Rect, &'a V)>,
    ) {
        for y in local_min.y()..=local_max.y() {
            for x in local_min.x()..=local_max.x() {
                let segment = &self.values[y as usize][x as usize];
                segment.query(bounds, results);
            }
        }
    }
}

struct Segment<K: Clone + Hash + Eq, V: Clone> {
    elements: Vec<(Rect, K, V)>,
}

impl<K: Clone + Hash + Eq, V: Clone> Segment<K, V> {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn query<'a>(&'a self, bounds: Rect, results: &mut FxHashMap<&'a K, (Rect, &'a V)>) {
        for (element_bounds, key, value) in &self.elements {
            if Rect::touches(bounds, *element_bounds) {
                results.insert(key, (*element_bounds, value));
            }
        }
    }

    pub fn insert(&mut self, bounds: Rect, key: K, value: V) {
        self.elements.push((bounds, key, value))
    }
}
