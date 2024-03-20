use crate::game_state::{H, PREVIEW_H, W};
use enum_iterator::Sequence;

#[derive(Clone, Sequence)]
pub enum Shape {
    S,
    Z,
    L,
    J,
    I,
    O,
    T,
}

#[derive(Clone)]
pub enum Orientation {
    Up,
    Right,
    Down,
    Left,
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

    fn rotate_cw_about_corner(&self, center: &RelPos) -> RelPos {
        let (mut old_x, mut old_y) = (self.0 - center.0, self.1 - center.1);
        if old_x >= 0 {
            old_x += 1;
        }
        if old_y >= 0 {
            old_y += 1;
        }
        let (mut new_x, mut new_y) = (old_y, -old_x);
        if new_x > 0 {
            new_x -= 1;
        }
        if new_y > 0 {
            new_y -= 1;
        }
        RelPos(new_x + center.0, new_y + center.1)
    }
}

impl Shape {
    pub fn starting_tetromino_location(&self) -> TetrominoLocation {
        match self {
            Self::O => TetrominoLocation(W / 2 - 1, H - PREVIEW_H),
            Self::I => TetrominoLocation(W / 2 - 2, H - PREVIEW_H - 2),
            _ => TetrominoLocation(W / 2 - 1, H - PREVIEW_H - 1),
        }
    }
    pub fn relative_positions(&self, o: &Orientation) -> [RelPos; 4] {
        let mut positions = self.up_positions();
        let rotate_fn = match self {
            Self::O => return positions,
            Self::I => |p: &RelPos| p.rotate_cw_about_corner(&RelPos(2, 2)),
            _ => |p: &RelPos| p.rotate_cw_about_block(&RelPos(1, 1)),
        };

        for p in &mut positions {
            for _ in 0..o.cw_rotations() {
                *p = rotate_fn(p as &RelPos)
            }
        }
        positions
    }

    fn up_positions(&self) -> [RelPos; 4] {
        let y = 2;
        [RelPos(0, y), RelPos(1, y), RelPos(2, y), RelPos(3, y)]
    }
}

impl Orientation {
    pub fn cw(&self) -> Orientation {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }

    pub fn ccw(&self) -> Orientation {
        self.cw().cw().cw()
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
