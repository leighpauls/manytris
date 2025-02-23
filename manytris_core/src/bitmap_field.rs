use crate::consts;
use crate::consts::NUM_POSITIONS;
use crate::field::Pos;
use std::fmt::{Debug, Formatter};
use bytemuck::AnyBitPattern;

pub const FIELD_BYTES: usize = NUM_POSITIONS / 8 + if (NUM_POSITIONS % 8) == 0 { 0 } else { 1 };

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, AnyBitPattern)]
pub struct BitmapField {
    bytes: [u8; FIELD_BYTES],
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
