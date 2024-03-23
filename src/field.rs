use std::collections::HashMap;
use crate::consts;
use crate::tetromino::Tetromino;
use crate::shapes::Shape;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

pub struct Field {
    occupied: HashMap<Pos, Shape>,
}

impl Pos {
    pub fn out_of_bounds(&self) -> bool {
        self.x < 0 || self.x >= consts::W || self.y < 0
    }
}

impl Field {
    pub fn new() -> Field {
        Field {
            occupied: HashMap::new(),
        }
    }

    pub(crate) fn apply_tetrominio(&mut self, t: &Tetromino) {
        for block_pos in &t.get_blocks() {
            self.occupied.insert(block_pos.clone(), t.shape);
        }
    }

    pub(crate) fn find_shadow(&self, active: &Tetromino) -> Tetromino {
        let mut shadow = active.clone();
        while let Some(new_shadow) = shadow.down() {
            if !self.is_valid(&new_shadow) {
                break;
            }
            shadow = new_shadow;
        }
        shadow
    }

    pub(crate) fn get_occupied_block(&self, pos: &Pos) -> Option<Shape> {
        Some(self.occupied.get(pos)?.clone())
    }

    pub fn is_valid(&self, t: &Tetromino) -> bool {
        for p in t.get_blocks() {
            if self.get_occupied_block(&p).is_some() {
                return false;
            }
        }
        true
    }
}
