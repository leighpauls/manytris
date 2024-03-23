use crate::consts;
use crate::shapes::Shape;
use crate::tetromino::Tetromino;
use std::cmp::max;
use std::collections::HashMap;

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

    pub fn apply_tetrominio(&mut self, t: &Tetromino) {
        for block_pos in &t.get_blocks() {
            self.occupied.insert(block_pos.clone(), t.shape);
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
            return;
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

    pub fn get_occupied_block(&self, pos: &Pos) -> Option<Shape> {
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
