use crate::consts::NUM_POSITIONS;
use crate::tetromino::Tetromino;

pub const FIELD_BYTES: usize = NUM_POSITIONS / 8 + if (NUM_POSITIONS % 8) == 0 { 0 } else { 1 };

#[repr(C)]
#[derive(Debug)]
pub struct TetrominoPositions {
    pos: [[u8; 2]; 4],
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct BitmapField {
    bytes: [u8; FIELD_BYTES],
}

#[repr(C)]
pub struct DropConfig {
    pub tetromino_idx: u32,
    pub initial_field_idx: u32,
    pub dest_field_idx: u32,
}

impl From<Tetromino> for TetrominoPositions {
    fn from(value: Tetromino) -> Self {
        Self {
            pos: value.get_blocks().map(|p| [p.x as u8, p.y as u8])
        }
    }
}

