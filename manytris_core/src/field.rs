use crate::bitmap_field::BitmapField;
use crate::consts;
use crate::shapes::Shape;
use crate::tetromino::Tetromino;
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Hash, Debug, Deserialize, Serialize)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct Field {
    occupied: [[Option<OccupiedBlock>; consts::W_US]; consts::MAX_H_US],
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum OccupiedBlock {
    FromShape(Shape),
    FromGarbage,
}

impl Pos {
    pub fn out_of_bounds(&self) -> bool {
        self.x < 0 || self.x >= consts::W || self.y < 0
    }

    fn is_safe(&self) -> bool {
        (!self.out_of_bounds()) && self.y < consts::MAX_H
    }
}

impl Field {
    pub fn with_initial_occupied(occupied: impl IntoIterator<Item = Pos>) -> Self {
        let mut res = Self::default();
        for p in occupied.into_iter() {
            res.set_safe(&p, Some(OccupiedBlock::FromGarbage));
        }
        res
    }

    /// Apply the tetromino, return the number of lines cleared.
    pub fn apply_tetrominio(&mut self, t: &Tetromino) -> i32 {
        for block_pos in &t.get_blocks() {
            self.set_safe(block_pos, Some(OccupiedBlock::FromShape(t.shape)));
        }
        let mut num_to_drop = 0;

        for y in 0..consts::MAX_H_US {
            let num_occupied = self.occupied[y].iter().flatten().count();
            match num_occupied {
                consts::W_US => {
                    num_to_drop += 1;
                    self.occupied[y] = Default::default();
                }
                0 => break,
                _ => {
                    if num_to_drop > 0 {
                        self.occupied[y - num_to_drop] = self.occupied[y];
                        self.occupied[y] = Default::default();
                    }
                }
            }
        }

        return num_to_drop as i32;
    }

    pub fn find_shadow(&self, active: &Tetromino) -> Tetromino {
        let mut shadow = active.clone();
        while let Some(new_shadow) = shadow.down() {
            if !self.is_valid(&new_shadow) {
                break;
            }
            shadow = new_shadow;
        }
        shadow
    }

    pub fn get_occupied_block(&self, pos: &Pos) -> Option<OccupiedBlock> {
        if pos.is_safe() {
            self.occupied[pos.y as usize][pos.x as usize]
        } else {
            None
        }
    }

    pub fn is_valid(&self, t: &Tetromino) -> bool {
        for p in t.get_blocks() {
            if self.get_occupied_block(&p).is_some() {
                return false;
            }
        }
        true
    }

    pub fn is_lockable(&self, t: &Tetromino) -> bool {
        for p in t.get_blocks() {
            let test_pos = Pos { x: p.x, y: p.y - 1 };
            if test_pos.y < 0 || self.get_occupied_block(&test_pos).is_some() {
                return true;
            }
        }
        false
    }

    pub fn apply_garbage(&mut self) {
        for y in (1..consts::MAX_H_US).rev() {
            self.occupied[y] = self.occupied[y - 1]
        }

        // insert garbage at the bottom
        for x in 0..(consts::W_US - 1) {
            self.occupied[0][x] = Some(OccupiedBlock::FromGarbage);
        }
        self.occupied[0][consts::W_US - 1] = None;
    }

    pub fn make_bitmap_field(&self) -> BitmapField {
        let mut bf = BitmapField::default();
        for y in 0..consts::MAX_H_US {
            for x in 0..consts::W_US {
                if self.occupied[y][x].is_some() {
                    bf.set(&Pos {
                        x: x as i32,
                        y: y as i32,
                    });
                }
            }
        }
        bf
    }

    fn set_safe(&mut self, p: &Pos, v: Option<OccupiedBlock>) {
        if p.is_safe() {
            self.occupied[p.y as usize][p.x as usize] = v;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compact_field_creation() {
        let mut f = Field::default();
        let t = Tetromino::new(Shape::I);
        f.apply_tetrominio(&t);

        let cf = f.make_bitmap_field();

        for p in &t.get_blocks() {
            assert_eq!(cf.occupied(p), true)
        }
        assert_eq!(cf.occupied(&Pos { x: 0, y: 0 }), false);
    }
}
