use std::collections::HashSet;

use float_ord::FloatOrd;
use fxhash::hash32;
use pathfinding::prelude::{astar, dijkstra, kruskal};

use crate::map::chunk::{Chunk, CHUNK_SIZE, CHUNK_SIZE_I32};
use crate::map::Tile;
use crate::util::coord::{coord, Coord};
use crate::util::random::Random;
use crate::util::triangulation::triangulate;
use crate::util::vector::Vec2;
use crate::{current_time, vec2};

const MAX_ROOM_ATTEMPTS: usize = CHUNK_SIZE.pow(2);

const ROOM_GAP: i32 = 1;

const MIN_ROOM_SIZE: usize = 3;
const AVERAGE_ROOM_SIZE: f64 = 4.25;
const MAX_ROOM_SIZE: usize = 12;

const TILES_PER_CHUNK_ENTRY: usize = 8;

const STARTING_ROOM_SHIFT: i32 = 2;
const EXTRA_EDGE_CHANCE: f64 = 0.33;

const PATHFINDING_TUNNEL_COST: u32 = 10;

const SIDE_LENGTH_PER_SPAWNER: i32 = 4;
const DECORATION_CHANCE: f64 = 0.033;
const CHEST_DISTANCE: f64 = 10.0;

const BARRIER_CHANCE: f64 = 0.5;

impl Chunk {
    pub fn generate(&mut self, seed: u32) {
        let start = current_time();

        let mut random = Random::new(hash32(&self.coord()).wrapping_add(seed));

        let rooms = self.generate_rooms(&mut random);
        for room in &rooms {
            self.create_room(*room);
        }

        let awkward_points = Self::compute_awkward_points(&rooms);

        self.connect_all_rooms(&mut random, &rooms, &awkward_points);

        for entry in self.chunk_entries() {
            self.create_chunk_entry(entry, &awkward_points);
        }

        self.add_barriers(&rooms, &mut random);
        self.add_features(&mut random, &rooms);

        log::debug!(
            "Generated chunk ({}, {}) in {}ms",
            self.coord().x(),
            self.coord().y(),
            current_time() - start
        );
    }

    fn generate_rooms(&self, random: &mut Random) -> Vec<Room> {
        let mut rooms = vec![];

        if self.coord() == coord(0, 0) {
            rooms.push(Room::starting())
        }

        for _ in 0..MAX_ROOM_ATTEMPTS {
            let room = Room::random(random);

            if !rooms
                .iter()
                .any(|other| room.distance(*other) < ROOM_GAP as u32)
            {
                rooms.push(room);
            }
        }

        rooms
    }

    fn compute_awkward_points(rooms: &[Room]) -> HashSet<Coord> {
        let mut result = HashSet::new();

        let mut insert = |x: i32, y: i32| result.insert(coord(x, y));

        for room in rooms {
            insert(room.x - 1, room.y - 1);
            insert(room.x, room.y - 1);
            insert(room.x - 1, room.y);

            insert(room.x + room.width, room.y - 1);
            insert(room.x + room.width - 1, room.y - 1);
            insert(room.x + room.width, room.y);

            insert(room.x - 1, room.y + room.height);
            insert(room.x, room.y + room.height);
            insert(room.x - 1, room.y + room.height - 1);

            insert(room.x + room.width, room.y + room.height);
            insert(room.x + room.width - 1, room.y + room.height);
            insert(room.x + room.width, room.y + room.height - 1);
        }

        for i in 0..CHUNK_SIZE_I32 {
            insert(i, 0);
            insert(0, i);
            insert(i, CHUNK_SIZE_I32 - 1);
            insert(CHUNK_SIZE_I32 - 1, i);
        }

        result
    }

    fn connect_all_rooms(
        &mut self,
        random: &mut Random,
        rooms: &[Room],
        awkward_points: &HashSet<Coord>,
    ) {
        let vertices = rooms
            .iter()
            .map(|room| room.center().center())
            .collect::<Vec<_>>();

        let triangulation = triangulate(&vertices);

        let edges = triangulation
            .iter()
            .flat_map(|triangle| {
                [
                    (triangle.0, triangle.1),
                    (triangle.1, triangle.2),
                    (triangle.2, triangle.0),
                ]
            })
            .map(|(a, b)| {
                let dist = Vec2::distance(a, b);
                (a.coord(), b.coord(), FloatOrd(dist))
            })
            .collect::<Vec<_>>();

        let mst = kruskal(&edges);

        for (a, b, _) in mst {
            self.create_path(*a, *b, awkward_points);
        }

        for (a, b, _) in edges {
            if random.next_f64() < EXTRA_EDGE_CHANCE {
                self.create_path(a, b, awkward_points);
            }
        }
    }

    fn add_barriers(&mut self, rooms: &[Room], random: &mut Random) {
        let min_coord = coord(1, 1);
        let max_coord = coord(CHUNK_SIZE_I32, CHUNK_SIZE_I32) - 2;

        let coords = rooms
            .iter()
            .flat_map(|room| {
                let bottom_left = room.min_coord() - 1;
                let top_right = room.max_coord();
                let top_left = coord(bottom_left.x(), top_right.y());
                let bottom_right = coord(top_right.x(), bottom_left.y());

                let edges = [
                    (bottom_left, bottom_right),
                    (bottom_left, top_left),
                    (bottom_right, top_right),
                    (top_left, top_right),
                ];

                edges
                    .into_iter()
                    .flat_map(|(start, end)| Coord::between_inclusive(start, end))
            })
            .filter(|coord| coord.is_between_inclusive(min_coord, max_coord));

        let mut barrier_positions = Vec::<Coord>::new();

        for coord in coords {
            if self.at(coord) != Tile::Floor {
                continue;
            }

            if !self.is_barrier_location(coord) {
                continue;
            }

            let barrier_nearby = barrier_positions
                .iter()
                .any(|other| coord.distance(*other) <= 1);

            if barrier_nearby {
                continue;
            }

            barrier_positions.push(coord);

            if random.next_f64() < BARRIER_CHANCE || self.is_room_barrier(coord, &rooms[0]) {
                *self.at_mut(coord) = Tile::Barrier;
            }
        }
    }

    fn is_room_barrier(&self, coord: Coord, room: &Room) -> bool {
        let min = room.min_coord() - 1;
        let max = room.max_coord();
        coord.is_between_inclusive(min, max)
    }

    fn is_barrier_location(&self, coord: Coord) -> bool {
        let top = self.at(coord.top());
        let bottom = self.at(coord.bottom());
        let left = self.at(coord.left());
        let right = self.at(coord.right());

        let cases = [
            ([top, bottom], [left, right]),
            ([left, right], [top, bottom]),
        ];

        cases.into_iter().any(|(non_walls, walls)| {
            walls.into_iter().all(|wall| wall == Tile::Wall)
                && non_walls.into_iter().all(|non_wall| non_wall != Tile::Wall)
        })
    }

    fn add_features(&mut self, random: &mut Random, rooms: &[Room]) {
        for room in rooms {
            let mut feature_positions = Vec::new();

            self.add_chest(random, room, &mut feature_positions);
            self.add_spawners(random, room, &mut feature_positions);
            self.add_decorations(random, room, &mut feature_positions);
        }
    }

    fn available_feature_position(
        &self,
        random: &mut Random,
        room: &Room,
        feature_positions: &mut Vec<Vec2>,
    ) -> Option<Vec2> {
        const MAX_ATTEMPTS: usize = 10;

        let room_length = i32::max(room.width, room.height) as f64;
        let distance = room_length / (feature_positions.len() + 1) as f64 * 0.75;

        'attemptLoop: for _ in 0..MAX_ATTEMPTS {
            let position = room.feature_position(random);

            for other_position in feature_positions.iter() {
                if Vec2::taxicab_distance(position, *other_position) < distance {
                    continue 'attemptLoop;
                }
            }

            feature_positions.push(position);
            return Some(position);
        }

        None
    }

    fn add_chest(&mut self, random: &mut Random, room: &Room, feature_positions: &mut Vec<Vec2>) {
        if !self.can_add_chest(room) {
            return;
        }

        if let Some(position) = self.available_feature_position(random, room, feature_positions) {
            self.create_chest(position);
        }
    }

    fn can_add_chest(&self, room: &Room) -> bool {
        if room.is_starting {
            return false;
        }

        for chest in self.chests() {
            let position = room.center().start() + self.chunk_start();
            let chest_position = chest.position();

            if Vec2::distance_squared(position, chest_position) < CHEST_DISTANCE.powi(2) {
                return false;
            }
        }

        true
    }

    fn add_spawners(
        &mut self,
        random: &mut Random,
        room: &Room,
        feature_positions: &mut Vec<Vec2>,
    ) {
        if room.is_starting {
            let position = room.center().center() + vec2(STARTING_ROOM_SHIFT as f64, 0.0);
            feature_positions.push(position);
            self.create_spawner(position);
            return;
        }

        let count = i32::max(room.width, room.height) / SIDE_LENGTH_PER_SPAWNER;

        for _ in 0..count {
            if let Some(position) = self.available_feature_position(random, room, feature_positions)
            {
                self.create_spawner(position);
            }
        }
    }

    fn add_decorations(
        &mut self,
        random: &mut Random,
        room: &Room,
        feature_positions: &mut Vec<Vec2>,
    ) {
        let size = (room.width * room.height) as usize;
        let count = random.next_binomial(size, DECORATION_CHANCE);

        for _ in 0..count {
            if let Some(position) = self.available_feature_position(random, room, feature_positions)
            {
                self.create_decoration(position);
            }
        }
    }

    fn create_room(&mut self, room: Room) {
        for y in room.y..room.y + room.height {
            for x in room.x..room.x + room.width {
                *self.at_mut(coord(x, y)) = Tile::Floor;
            }
        }
    }

    fn fill_path(&mut self, nodes: Vec<Coord>) {
        for coord in nodes {
            let tile = self.at_mut(coord);

            if *tile == Tile::Wall {
                *tile = Tile::Floor;
            }
        }
    }

    fn create_path(&mut self, start: Coord, end: Coord, awkward_points: &HashSet<Coord>) {
        let successors = |coord: &Coord| self.pathfinding_successors(*coord, awkward_points);
        let heuristic = |coord: &Coord| coord.distance(end);
        let (nodes, _) = astar(&start, successors, heuristic, |coord| *coord == end).unwrap();
        self.fill_path(nodes);
    }

    fn create_chunk_entry(&mut self, start: Coord, awkward_points: &HashSet<Coord>) {
        let successors = |coord: &Coord| self.pathfinding_successors(*coord, awkward_points);
        let success = |coord: &Coord| self.at(*coord).is_walkable();
        let (nodes, _) = dijkstra(&start, successors, success).unwrap();
        self.fill_path(nodes);
    }

    fn pathfinding_successors<'a>(
        &'a self,
        coord: Coord,
        awkward_points: &'a HashSet<Coord>,
    ) -> impl Iterator<Item = (Coord, u32)> + 'a {
        let options = [coord.top(), coord.bottom(), coord.left(), coord.right()];

        options
            .into_iter()
            .filter(|coord| Self::in_bounds(*coord))
            .map(|coord| (coord, self.pathfinding_cost(coord, awkward_points)))
    }

    fn pathfinding_cost(&self, coord: Coord, awkward_points: &HashSet<Coord>) -> u32 {
        if self.at(coord) != Tile::Wall {
            1
        } else if awkward_points.contains(&coord) {
            PATHFINDING_TUNNEL_COST * 10
        } else {
            PATHFINDING_TUNNEL_COST
        }
    }

    fn in_bounds(coord: Coord) -> bool {
        coord.x() >= 0 && coord.y() > -0 && coord.x() < CHUNK_SIZE_I32 && coord.y() < CHUNK_SIZE_I32
    }

    fn chunk_entries(&self) -> impl Iterator<Item = Coord> {
        let left = Self::chunk_side_entries(self.coord(), false).map(|n| coord(0, n));

        let bottom = Self::chunk_side_entries(self.coord(), true).map(|n| coord(n, 0));

        let top = Self::chunk_side_entries(self.coord().top(), true)
            .map(|n| coord(n, CHUNK_SIZE_I32 - 1));

        let right = Self::chunk_side_entries(self.coord().right(), false)
            .map(|n| coord(CHUNK_SIZE_I32 - 1, n));

        left.chain(bottom).chain(top).chain(right)
    }

    fn chunk_side_entries(coord: Coord, vertical: bool) -> impl Iterator<Item = i32> {
        let mut hash = hash32(&coord);
        if vertical {
            hash = hash.wrapping_mul(31);
        }

        let mut random = Random::new(hash);

        let count = CHUNK_SIZE / TILES_PER_CHUNK_ENTRY;

        (0..count).map(move |i| {
            let base = (i * TILES_PER_CHUNK_ENTRY) as u32;
            let range = base + 1..base + TILES_PER_CHUNK_ENTRY as u32 - 1;
            random.next_u32_in(&range) as i32
        })
    }
}

#[derive(Copy, Clone)]
struct Room {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    is_starting: bool,
}

impl Room {
    pub fn starting() -> Self {
        let (width, height) = (7, 5);
        let x = CHUNK_SIZE_I32 / 2 - width / 2 + STARTING_ROOM_SHIFT;
        let y = CHUNK_SIZE_I32 / 2 - height / 2;

        Self {
            is_starting: true,
            width,
            height,
            x,
            y,
        }
    }

    pub fn random(random: &mut Random) -> Self {
        let (width, height) = Self::dimensions(random);

        let x_range = 0..(CHUNK_SIZE as u32 - width as u32 - 1);
        let y_range = 0..(CHUNK_SIZE as u32 - height as u32 - 1);

        let x = random.next_u32_in(&x_range) as i32;
        let y = random.next_u32_in(&y_range) as i32;

        Self {
            is_starting: false,
            x,
            y,
            width,
            height,
        }
    }

    fn dimensions(random: &mut Random) -> (i32, i32) {
        let x = random.next_binomial_between(MIN_ROOM_SIZE, AVERAGE_ROOM_SIZE, MAX_ROOM_SIZE);

        let min_y = x.saturating_sub(2).max(MIN_ROOM_SIZE);
        let max_y = (x + 2 + 1).min(MAX_ROOM_SIZE);

        let y = random.next_binomial_between(min_y, x as f64, max_y);

        (x as i32, y as i32)
    }

    pub fn feature_position(&self, random: &mut Random) -> Vec2 {
        let min = self.min_coord().start() + 0.75;
        let max = self.max_coord().start() - 0.75;

        let x = random.next_f64_in(min.x()..max.x());
        let y = random.next_f64_in(min.y()..max.y());

        vec2(x, y)
    }

    pub fn center(&self) -> Coord {
        coord(self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn min_coord(&self) -> Coord {
        coord(self.x, self.y)
    }

    pub fn max_coord(&self) -> Coord {
        coord(self.x + self.width, self.y + self.height)
    }

    pub fn distance(self, other: Room) -> u32 {
        let x_touches = self.max_coord().x() >= other.min_coord().x()
            && self.min_coord().x() <= other.max_coord().x();

        let y_touches = self.max_coord().y() >= other.min_coord().y()
            && self.min_coord().y() <= other.max_coord().y();

        let x_distance = if x_touches {
            0
        } else {
            u32::min(
                i32::abs_diff(self.max_coord().x(), other.min_coord().x()),
                i32::abs_diff(self.min_coord().x(), other.max_coord().x()),
            )
        };

        let y_distance = if y_touches {
            0
        } else {
            u32::min(
                i32::abs_diff(self.max_coord().y(), other.min_coord().y()),
                i32::abs_diff(self.min_coord().y(), other.max_coord().y()),
            )
        };

        x_distance + y_distance
    }
}
