use std::fmt::{Debug, Formatter};

use crate::bot::bot_player::MovementDescriptor;
use crate::bot::bot_start_positions::StartPositions;
use crate::consts;
use crate::consts::NUM_POSITIONS;
use crate::field::Pos;
use crate::tetromino::Tetromino;

pub const FIELD_BYTES: usize = NUM_POSITIONS / 8 + if (NUM_POSITIONS % 8) == 0 { 0 } else { 1 };

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TetrominoPositions {
    pos: [[u8; 2]; 4],
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct BitmapField {
    bytes: [u8; FIELD_BYTES],
}

#[repr(C)]
#[derive(Debug, Clone)]
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
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SearchParams {
    pub cur_search_depth: u8,
    pub upcoming_shape_idxs: UpcomingShapeIndexes,
}

#[repr(C)]
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ComputedDropConfig {
    pub shape_idx: u8,
    pub cw_rotations: u8,
    pub src_field_idx: u32,
    pub dest_field_idx: u32,
    pub left_shifts: u8,
    pub right_shifts: u8,
}

impl ComputedDropConfig {
    pub fn as_move_descriptor(&self, sp: &StartPositions) -> MovementDescriptor {
        MovementDescriptor {
            shape: sp.idx_to_shape.get(&self.shape_idx).unwrap().clone(),
            cw_rotations: self.cw_rotations as usize,
            shifts_right: self.right_shifts as isize - (self.left_shifts as isize),
        }
    }
}

impl From<Tetromino> for TetrominoPositions {
    fn from(value: Tetromino) -> Self {
        Self {
            pos: value.get_blocks().map(|p| [p.x as u8, p.y as u8]),
        }
    }
}

impl Default for BitmapField {
    fn default() -> Self {
        Self {
            bytes: [0; FIELD_BYTES],
        }
    }
}

impl Debug for BitmapField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("----------\n")?;
        for y in (0..consts::MAX_H).rev() {
            for x in 0..consts::W {
                f.write_str(if self.occupied(&Pos { x, y }) {
                    "X"
                } else if y < consts::H - consts::PREVIEW_H {
                    " "
                } else {
                    "_"
                })?;
            }
            f.write_str("\n")?;
        }
        f.write_str("----------\n")?;
        Ok(())
    }
}

impl BitmapField {
    pub fn set(&mut self, pos: &Pos) {
        let (byte, mask) = Self::byte_and_mask(pos);
        self.bytes[byte] |= mask;
    }

    pub fn occupied(&self, pos: &Pos) -> bool {
        let (byte, mask) = Self::byte_and_mask(pos);
        (self.bytes[byte] & mask) != 0
    }

    fn byte_and_mask(pos: &Pos) -> (usize, u8) {
        let bit_index = (pos.y * consts::W + pos.x) as usize;
        let byte = bit_index / 8;
        let mask_shift = bit_index - (byte * 8);
        (byte, 1 << mask_shift)
    }
}
