use crate::map::chest::Chest;
use crate::map::Tile;
use crate::util::coord::Coord;
use crate::util::vector::Vec2;
use crate::vec2;

pub const CHUNK_SIZE: usize = 50;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;

pub struct Chunk {
    coord: Coord,
    data: Box<[[Tile; CHUNK_SIZE]; CHUNK_SIZE]>,
    spawners: Vec<Vec2>,
    chests: Vec<Chest>,
    decorations: Vec<Vec2>,
}

impl Chunk {
    pub fn new(coord: Coord, seed: u32) -> Self {
        let data = box [[Tile::Wall; CHUNK_SIZE]; CHUNK_SIZE];

        let mut result = Self {
            coord,
            data,
            spawners: Vec::new(),
            chests: Vec::new(),
            decorations: Vec::new(),
        };

        result.generate(seed);

        result
    }

    pub fn coord(&self) -> Coord {
        self.coord
    }

    pub fn at(&self, coord: Coord) -> Tile {
        self.data[coord.y() as usize][coord.x() as usize]
    }

    pub fn at_mut(&mut self, coord: Coord) -> &mut Tile {
        &mut self.data[coord.y() as usize][coord.x() as usize]
    }

    pub fn chunk_start(&self) -> Vec2 {
        vec2(
            (self.coord.x() * CHUNK_SIZE_I32 - CHUNK_SIZE_I32 / 2) as f64,
            (self.coord.y() * CHUNK_SIZE_I32 - CHUNK_SIZE_I32 / 2) as f64,
        )
    }

    pub fn create_spawner(&mut self, position: Vec2) {
        let offset = self.chunk_start();
        self.spawners.push(position + offset);
    }

    pub fn spawners(&self) -> impl Iterator<Item = Vec2> + '_ {
        self.spawners.iter().copied()
    }

    pub fn create_chest(&mut self, position: Vec2) {
        let offset = self.chunk_start();
        let chest = Chest::new(position + offset);
        self.chests.push(chest);
    }

    pub fn chests(&self) -> impl Iterator<Item = &Chest> {
        self.chests.iter()
    }

    pub fn chests_mut(&mut self) -> impl Iterator<Item = &mut Chest> {
        self.chests.iter_mut()
    }

    pub fn create_decoration(&mut self, position: Vec2) {
        let offset = self.chunk_start();
        self.decorations.push(position + offset);
    }

    pub fn decorations(&self) -> impl Iterator<Item = Vec2> + '_ {
        self.decorations.iter().copied()
    }
}
