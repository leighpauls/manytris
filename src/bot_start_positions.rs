use bevy::utils::HashMap;
use enum_iterator::all;
use crate::consts;
use crate::shapes::{Rot, Shape};
use crate::tetromino::Tetromino;

pub fn bot_start_position(s: Shape, cw_rotations: usize) -> Tetromino {
    compute_bot_start_positions_for_shape(s)[cw_rotations].clone()
}
fn compute_bot_start_positions_for_shape(s: Shape) -> [Tetromino; 4] {
    let mut result = vec![];
    for rotations in 0..4 {
        // Rotate to the appropiate height
        let mut t = Tetromino::new(s);
        (0..rotations).for_each(|_| t = t.rotation_options(Rot::Cw).get(0).unwrap().clone());
        // raise above the main field
        let lowest_y = t.get_blocks().into_iter().map(|p| p.y).min().unwrap();

        t.raise(consts::H - lowest_y);
        result.push(t);
    }
    result.try_into().unwrap()
}
