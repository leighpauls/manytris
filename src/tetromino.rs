use serde::{Deserialize, Serialize};
use crate::field::Pos;
use crate::shapes;
use crate::shapes::{Orientation, Rot, Shape, Shift, TetrominoLocation};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Tetromino {
    pub shape: Shape,
    loc: TetrominoLocation,
    orientation: Orientation,
}

impl Tetromino {
    pub fn new(shape: Shape) -> Self {
        Self {
            loc: shape.starting_tetromino_location(),
            shape,
            orientation: Orientation::Up,
        }
    }

    pub fn for_preview(shape: Shape) -> Self {
        Self {
            loc: shape.preview_tetromino_location(),
            shape,
            orientation: Orientation::Up,
        }
    }

    pub fn get_blocks(&self) -> [Pos; 4] {
        let rels = self.shape.relative_positions(&self.orientation);
        rels.map(|rp| Pos {
            x: self.loc.0 + rp.0,
            y: self.loc.1 + rp.1,
        })
    }

    pub fn contains(&self, p: &Pos) -> bool {
        self.get_blocks().contains(p)
    }

    /// Returns a new Tetromino, dropped 1 space, if valid.
    pub fn down(&self) -> Option<Tetromino> {
        let mut t = self.clone();
        t.loc.1 -= 1;
        for p in &t.get_blocks() {
            if p.out_of_bounds() {
                return None;
            }
        }
        Some(t)
    }

    pub fn shift(&self, dir: Shift) -> Option<Tetromino> {
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
    pub fn rotation_options(&self, dir: Rot) -> Vec<Tetromino> {
        let new_orientation = self.orientation.rotate(dir);
        let kick_attempts = shapes::kick_offsets(self.shape, self.orientation, new_orientation);

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
