use crate::shapes;
use crate::shapes::{Orientation, Rot, Shape, Shift, TetrominoLocation};
use crate::upcoming::UpcomingTetrominios;
use std::collections::HashSet;

pub const W: i32 = 10;
pub const H: i32 = 22;

pub const PREVIEW_H: i32 = 2;

const TOTAL_BLOCKS: usize = (W * H) as usize;

pub struct GameState {
    field: Field,
    active: Tetromino,
    upcoming: UpcomingTetrominios,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone)]
pub struct Tetromino {
    shape: Shape,
    loc: TetrominoLocation,
    orientation: Orientation,
}

struct Field {
    occupied: HashSet<Pos>,
}

pub enum BlockState {
    Empty,
    Occupied,
    Active,
}

impl Pos {
    fn to_buffer_idx(&self) -> usize {
        (self.y * W + self.x) as usize
    }

    fn out_of_bounds(&self) -> bool {
        self.x < 0 || self.x >= W || self.y < 0
    }
}

impl Tetromino {
    pub fn new(shape: Shape) -> Tetromino {
        Tetromino {
            loc: shape.starting_tetromino_location(),
            shape,
            orientation: Orientation::Up,
        }
    }

    fn get_blocks(&self) -> [Pos; 4] {
        let rels = self.shape.relative_positions(&self.orientation);
        rels.map(|rp| Pos {
            x: self.loc.0 + rp.0,
            y: self.loc.1 + rp.1,
        })
    }

    fn contains(&self, p: &Pos) -> bool {
        self.get_blocks().contains(p)
    }

    /// Returns a new Tetromino, dropped 1 space, if valid.
    fn down(&self) -> Option<Tetromino> {
        let mut t = self.clone();
        t.loc.1 -= 1;
        for p in &t.get_blocks() {
            if p.out_of_bounds() {
                return None;
            }
        }
        Some(t)
    }

    fn shift(&self, dir: Shift) -> Option<Tetromino> {
        let mut new_t = self.clone();
        new_t.loc.0 += match dir {
            Shift::Left => -1,
            Shift::Right => 1,
        };

        if new_t.out_of_bounds() {
            None
        } else {
            Some(new_t)
        }
    }

    /// Return the list of possible tetromino kick attempts
    fn rotate(&self, dir: Rot) -> Vec<Tetromino> {
        let new_orientation = self.orientation.rotate(dir);
        let kick_attempts = shapes::kick_offsets(
            self.shape,
            self.orientation,
            new_orientation,
        );

        let mut result = vec![];
        for (dx, dy) in kick_attempts {
            let new_t = Tetromino {
                shape: self.shape,
                orientation: new_orientation,
                loc: TetrominoLocation(self.loc.0 + dx, self.loc.1 + dy),
            };
            if !new_t.out_of_bounds() {
                result.push(new_t);
            }
        }
        result
    }

    fn out_of_bounds(&self) -> bool {
        for p in self.get_blocks() {
            if p.out_of_bounds() {
                return true;
            }
        }
        false
    }
}

impl Field {
    fn new() -> Field {
        Field {
            occupied: HashSet::new(),
        }
    }

    fn apply_tetrominio(&mut self, t: &Tetromino) {
        for block_pos in &t.get_blocks() {
            self.occupied.insert(block_pos.clone());
        }
    }

    fn is_occupied(&self, pos: &Pos) -> bool {
        self.occupied.contains(pos)
    }

    fn is_valid(&self, t: &Tetromino) -> bool {
        for p in t.get_blocks() {
            if self.is_occupied(&p) {
                return false;
            }
        }
        true
    }
}

impl GameState {
    pub fn new() -> GameState {
        let mut upcoming = UpcomingTetrominios::new();
        return GameState {
            field: Field::new(),
            active: Tetromino::new(upcoming.take()),
            upcoming,
        };
    }

    fn lock_active_tetromino(&mut self) {
        self.field.apply_tetrominio(&self.active);
        self.active = Tetromino::new(self.upcoming.take());
        if !self.field.is_valid(&self.active) {
            panic!("Game over!");
        }
    }

    /// Drop the active tetromino, return True if it locks.
    pub fn down(&mut self) -> bool {
        match self.active.down() {
            Some(new_t) if self.field.is_valid(&new_t) => {
                self.active = new_t;
                false
            }
            _ => {
                self.lock_active_tetromino();
                true
            }
        }
    }

    pub fn drop(&mut self) {
        while !self.down() {}
    }

    pub fn shift(&mut self, dir: Shift) -> Option<()> {
        let new_t = self.active.shift(dir)?;
        if self.field.is_valid(&new_t) {
            self.active = new_t;
            return Some(());
        }
        None
    }

    pub fn rotate(&mut self, dir: Rot) {
        for new_t in self.active.rotate(dir) {
            if self.field.is_valid(&new_t) {
                self.active = new_t;
                return;
            }
        }
    }

    pub fn check_block(&self, p: &Pos) -> BlockState {
        if self.active.contains(p) {
            BlockState::Active
        } else if self.field.is_occupied(p) {
            BlockState::Occupied
        } else {
            BlockState::Empty
        }
    }
}
