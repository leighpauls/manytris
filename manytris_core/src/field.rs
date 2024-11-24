use crate::bitmap_field::BitmapField;
use crate::consts;
use crate::shapes::Shape;
use crate::tetromino::Tetromino;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use std::mem;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Deserialize, Serialize)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Field {
    occupied: HashMap<Pos, OccupiedBlock>,
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

    fn up(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
        }
    }
}

impl Field {
    pub fn new() -> Field {
        Field {
            occupied: HashMap::new(),
        }
    }

    /// Apply the tetromino, return the number of lines cleared.
    pub fn apply_tetrominio(&mut self, t: &Tetromino) -> i32 {
        for block_pos in &t.get_blocks() {
            self.occupied
                .insert(block_pos.clone(), OccupiedBlock::FromShape(t.shape));
        }

        let mut blocks_by_line = HashMap::<i32, i32>::new();
        let mut lines_to_remove = vec![];
        let mut max_y = 0;
        for (pos, _) in &mut self.occupied {
            let y = pos.y;
            max_y = max(y, max_y);

            let count = blocks_by_line.get(&y);
            let new_count = count.unwrap_or(&0) + 1;
            blocks_by_line.insert(y, new_count);
            if new_count == 10 {
                lines_to_remove.push(y);
            }
        }

        if lines_to_remove.is_empty() {
            return 0;
        }

        let mut drop_dist = 0;
        for y in 0..=max_y {
            let replace = if lines_to_remove.contains(&y) {
                drop_dist += 1;
                false
            } else {
                true
            };

            for x in 0..consts::W {
                if let (Some(s), true) = (self.occupied.remove(&Pos { x, y }), replace) {
                    let new_pos = Pos {
                        x,
                        y: y - drop_dist,
                    };
                    self.occupied.insert(new_pos, s);
                }
            }
        }

        return lines_to_remove.len() as i32;
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

    pub fn is_lockable(&self, t: &Tetromino) -> bool {
        for p in t.get_blocks() {
            let test_pos = Pos { x: p.x, y: p.y - 1 };
            if test_pos.y < 0 || self.occupied.contains_key(&test_pos) {
                return true;
            }
        }
        false
    }

    pub fn apply_garbage(&mut self) {
        // move all blocks up 1
        self.occupied = mem::take(&mut self.occupied)
            .into_iter()
            .map(|(p, s)| (p.up(), s))
            .collect();

        // insert garbage at the bottom
        for x in 0..(consts::W - 1) {
            self.occupied
                .insert(Pos { x, y: 0 }, OccupiedBlock::FromGarbage);
        }
    }

    #[cfg(target_os = "macos")]
    pub fn make_bitmap_field(&self) -> BitmapField {
        let mut bf = BitmapField::default();
        for p in self.occupied.keys() {
            bf.set(p);
        }
        bf
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compact_field_creation() {
        let mut f = Field::new();
        let t = Tetromino::new(Shape::I);
        f.apply_tetrominio(&t);

        let cf = f.make_bitmap_field();

        for p in &t.get_blocks() {
            assert_eq!(cf.occupied(p), true)
        }
        assert_eq!(cf.occupied(&Pos { x: 0, y: 0 }), false);
    }
}
