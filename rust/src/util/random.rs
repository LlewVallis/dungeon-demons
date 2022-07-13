use std::collections::Bound;
use std::hash::Hasher;
use std::ops::{Range, RangeBounds};
use std::sync::{Mutex, MutexGuard};

use fxhash::FxHasher32;
use lazy_static::lazy_static;

use crate::util::vector::{Vec2, Vector};
use crate::vec2;

const CONVOLUTION_CONSTANT: u32 = 2147483647;

const LCM_MULTIPLIER: u32 = 1664525;
const LCM_INCREMENT: u32 = 1013904223;

pub fn hash32_vec<const N: usize>(vector: Vector<N>) -> u32 {
    let mut hasher = FxHasher32::default();

    for component in vector.components() {
        hasher.write_u64(component.to_bits());
    }

    hasher.finish() as u32
}

lazy_static! {
    static ref GLOBAL_RANDOM: Mutex<Random> = Mutex::new(Random::new(0));
}

#[derive(Clone)]
pub struct Random {
    state: u32,
}

impl Random {
    pub fn global() -> MutexGuard<'static, Random> {
        GLOBAL_RANDOM.lock().unwrap()
    }

    pub fn new(state: u32) -> Self {
        Self { state }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(LCM_MULTIPLIER)
            .wrapping_add(LCM_INCREMENT);

        let x = self.state;
        let x = ((x >> 16) ^ x).wrapping_mul(CONVOLUTION_CONSTANT);
        x
    }

    pub fn next_u32_in(&mut self, range: &impl RangeBounds<u32>) -> u32 {
        let start = match range.start_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => n + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => n - 1,
            Bound::Unbounded => u32::MAX,
        };

        let distance = (end - start).wrapping_add(1);

        if distance == 0 {
            self.next_u32() + start
        } else {
            self.next_u32() % distance + start
        }
    }

    pub fn next_f64(&mut self) -> f64 {
        self.next_u32() as f64 / u32::MAX as f64
    }

    pub fn next_f64_in(&mut self, range: Range<f64>) -> f64 {
        self.next_f64() * (range.end - range.start) + range.start
    }

    pub fn next_binomial(&mut self, n: usize, p: f64) -> usize {
        if p >= 0.99 {
            return n;
        }

        if p <= 0.01 {
            return 0;
        }

        let mut remaining = n;
        let mut count = 0;

        loop {
            let wait = self.next_f64().log(1.0 - p).ceil() as usize;
            if wait > remaining {
                return count;
            }

            count += 1;
            remaining -= wait;
        }
    }

    pub fn next_binomial_between(&mut self, min: usize, average: f64, max: usize) -> usize {
        assert!(min as f64 <= average);
        assert!(average <= max as f64);

        if min == max {
            return min;
        }

        let n = max - min - 1;
        let expected = average - min as f64;
        let p = expected / n as f64;

        self.next_binomial(n, p) + min
    }

    pub fn next_vec2(&mut self) -> Vec2 {
        vec2(self.next_f64(), self.next_f64())
    }

    pub fn element<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        assert!(slice.len() <= u32::MAX as usize);
        let index = self.next_u32_in(&(0..slice.len() as u32));
        &slice[index as usize]
    }

    pub fn weighted_index<F, I>(&mut self, weights: F) -> usize
    where
        F: Fn() -> I,
        I: Iterator<Item = f64>,
    {
        let mut total = 0.0;
        let mut count = 0;

        for weight in weights() {
            total += weight;
            count += 1;
        }

        assert!(count > 0);

        let selected = self.next_f64_in(0.0..total);

        let mut cumulative = 0.0;
        for (i, weight) in weights().enumerate() {
            cumulative += weight;

            if cumulative > selected {
                return i;
            }
        }

        0
    }
}
