use crate::consts::{H, PREVIEW_H, W};
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

pub const KICK_ATTEMPTS: usize = 5;

#[derive(Copy, Clone, Debug, Sequence, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum Shape {
    S,
    Z,
    L,
    J,
    I,
    O,
    T,
}

#[derive(Copy, Clone)]
pub enum Orientation {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub enum Shift {
    Left,
    Right,
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub enum Rot {
    Cw,
    Ccw,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RelPos(pub i32, pub i32);

#[derive(Clone)]
pub struct TetrominoLocation(pub i32, pub i32);

impl RelPos {
    /// Rotate around a given block, like for T and Z
    fn rotate_cw_about_block(&self, center: &RelPos) -> RelPos {
        let (old_x, old_y) = (self.0 - center.0, self.1 - center.1);
        RelPos(old_y + center.0, -old_x + center.1)
    }
}

impl Shape {
    pub fn starting_tetromino_location(&self) -> TetrominoLocation {
        match self {
            Self::O => TetrominoLocation(W / 2 - 1, H - PREVIEW_H),
            Self::I => TetrominoLocation(W / 2 - 3, H - PREVIEW_H - 2),
            _ => TetrominoLocation(W / 2 - 2, H - PREVIEW_H - 1),
        }
    }

    pub fn preview_tetromino_location(&self) -> TetrominoLocation {
        match self {
            Self::O => TetrominoLocation(1, 1),
            Self::I => TetrominoLocation(-1, -1),
            _ => TetrominoLocation(0, 0),
        }
    }

    pub fn relative_positions(&self, o: &Orientation) -> [RelPos; 4] {
        let mut positions = self.up_positions();
        let rotate_fn = match self {
            Self::O => return positions,
            Self::I => |p: &RelPos| p.rotate_cw_about_block(&RelPos(2, 2)),
            _ => |p: &RelPos| p.rotate_cw_about_block(&RelPos(1, 1)),
        };

        for p in &mut positions {
            for _ in 0..o.cw_rotations() {
                *p = rotate_fn(p)
            }
        }
        positions
    }

    fn up_positions(&self) -> [RelPos; 4] {
        match self {
            Self::I => {
                let y = 2;
                [(1, y), (2, y), (3, y), (4, y)]
            }
            Self::J => [(0, 2), (0, 1), (1, 1), (2, 1)],
            Self::L => [(2, 2), (0, 1), (1, 1), (2, 1)],
            Self::O => [(0, 0), (0, 1), (1, 0), (1, 1)],
            Self::S => [(0, 1), (1, 1), (1, 2), (2, 2)],
            Self::T => [(0, 1), (1, 1), (1, 2), (2, 1)],
            Self::Z => [(0, 2), (1, 2), (1, 1), (2, 1)],
        }
        .map(|tup| RelPos(tup.0, tup.1))
    }
}

impl Orientation {
    pub fn rotate(&self, dir: Rot) -> Orientation {
        use Rot::{Ccw, Cw};
        match dir {
            Cw => match self {
                Self::Up => Self::Right,
                Self::Right => Self::Down,
                Self::Down => Self::Left,
                Self::Left => Self::Up,
            },
            Ccw => self.rotate(Cw).rotate(Cw).rotate(Cw),
        }
    }

    fn cw_rotations(&self) -> i32 {
        match self {
            Self::Up => 0,
            Self::Right => 1,
            Self::Down => 2,
            Self::Left => 3,
        }
    }
}

pub fn kick_offsets(
    shape: Shape,
    orig_orientation: Orientation,
    new_orientation: Orientation,
) -> [(i32, i32); KICK_ATTEMPTS] {
    let mut k_old = kick_consts(shape, orig_orientation);
    let k_new = kick_consts(shape, new_orientation);
    for i in 0..KICK_ATTEMPTS {
        k_old[i].0 -= k_new[i].0;
        k_old[i].1 -= k_new[i].1;
    }
    k_old
}

fn kick_consts(shape: Shape, orientation: Orientation) -> [(i32, i32); KICK_ATTEMPTS] {
    use Orientation::*;
    match (shape, orientation) {
        (Shape::O, _) => [(0, 0); 5],
        (Shape::I, Up) => [(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
        (Shape::I, Right) => [(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
        (Shape::I, Down) => [(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
        (Shape::I, Left) => [(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
        (_, Up | Down) => [(0, 0); 5],
        (_, Right) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        (_, Left) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cw_block() {
        assert_eq!(
            RelPos(0, 1).rotate_cw_about_block(&RelPos(1, 1)),
            RelPos(1, 2)
        );
    }
}
