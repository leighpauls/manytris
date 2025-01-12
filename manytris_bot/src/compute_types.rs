use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::{Display, Formatter};

use manytris_core::consts;
use manytris_core::shapes::Shape;
use manytris_core::tetromino::Tetromino;
use bytemuck::AnyBitPattern;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TetrominoPositions {
    pos: [[u8; 2]; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Default, Copy, AnyBitPattern)]
pub struct DropConfig {
    pub tetromino_idx: u32,
    pub next_tetromino_idx: u32,
    pub initial_field_idx: u32,
    pub dest_field_idx: u32,
    pub left_shifts: u8,
    pub right_shifts: u8,
}

#[repr(C)]
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct MoveResultScore {
    pub game_over: bool,
    pub lines_cleared: u8,
    pub height: u8,
    pub covered: u16,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ShapeStartingPositions {
    pub bot_positions: [TetrominoPositions; 4],
    pub player_position: TetrominoPositions,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ShapePositionConfig {
    pub starting_positions: [ShapeStartingPositions; consts::NUM_SHAPES],
}

pub type UpcomingShapeIndexes = [u8; consts::MAX_SEARCH_DEPTH + 1];

#[repr(C)]
#[derive(Eq, PartialEq, Copy, Clone, Debug, AnyBitPattern)]
pub struct SearchParams {
    pub cur_search_depth: u8,
    pub upcoming_shape_idxs: UpcomingShapeIndexes,
}

#[repr(C)]
#[derive(Eq, PartialEq, Clone, Debug, Copy, AnyBitPattern)]
pub struct ComputedDropConfig {
    pub shape_idx: u8,
    pub cw_rotations: u8,
    pub src_field_idx: u32,
    pub dest_field_idx: u32,
    pub left_shifts: u8,
    pub right_shifts: u8,
}

pub type UpcomingShapes = [Shape; consts::MAX_SEARCH_DEPTH + 1];

impl From<Tetromino> for TetrominoPositions {
    fn from(value: Tetromino) -> Self {
        Self {
            pos: value.get_blocks().map(|p| [p.x as u8, p.y as u8]),
        }
    }
}

impl Display for MoveResultScore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Lost: {}, Cleared: {}, covered: {}, Height: {}",
            self.game_over, self.lines_cleared, self.covered, self.height
        ))
    }
}

impl PartialOrd<Self> for MoveResultScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MoveResultScore {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.game_over != other.game_over {
            // Not game over is better
            if self.game_over {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        } else if self.lines_cleared != other.lines_cleared {
            // More lines cleared is better
            self.lines_cleared.cmp(&other.lines_cleared)
        } else if self.covered != other.covered {
            // less coverage is better
            other.covered.cmp(&self.covered)
        } else {
            // less height is better
            other.height.cmp(&self.height)
        }
    }
}
