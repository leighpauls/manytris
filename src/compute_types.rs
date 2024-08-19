use crate::consts;
use crate::consts::NUM_POSITIONS;
use crate::field::Pos;
use crate::shapes::{Rot, Shape};
use crate::tetromino::Tetromino;
use std::fmt::{Debug, Formatter};

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
    pub initial_field_idx: u32,
    pub dest_field_idx: u32,
    pub left_shifts: u8,
    pub right_shifts: u8,
}

impl TetrominoPositions {
    pub fn starting_rotations_for_shape(s: Shape) -> [TetrominoPositions; 4] {
        let mut t = Tetromino::new(s);
        (0..4)
            .map(|_| {
                let vec: Vec<[u8; 2]> = t
                    .get_blocks()
                    .into_iter()
                    .map(|p| [p.x as u8, p.y as u8])
                    .collect();
                t = t.rotation_options(Rot::Cw).get(0).unwrap().clone();
                TetrominoPositions {
                    pos: vec.try_into().unwrap(),
                }
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
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
        for y in (0..consts::H).rev() {
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
