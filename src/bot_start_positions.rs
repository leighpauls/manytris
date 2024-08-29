use bevy::utils::HashMap;
use enum_iterator::all;
use enum_map::EnumMap;

use crate::compute_types::{ShapePositionConfig, ShapeStartingPositions, TetrominoPositions};
use crate::consts;
use crate::shapes::{Rot, Shape};
use crate::tetromino::Tetromino;

pub struct StartPositions {
    bot_positions: EnumMap<Shape, [Tetromino; 4]>,
    pub idx_to_shape: HashMap<u8, Shape>,
    pub shape_to_idx: EnumMap<Shape, u8>,
    pub bot_positions_as_tp: EnumMap<Shape, [TetrominoPositions; 4]>,
    pub player_positions: EnumMap<Shape, TetrominoPositions>,
    pub shape_position_config: ShapePositionConfig,
}

impl StartPositions {
    pub fn new() -> Self {
        let bot_positions_as_tp = EnumMap::from_fn(|s| {
            compute_bot_start_positions_for_shape(s).map(|t| TetrominoPositions::from(t))
        });
        let player_positions = EnumMap::from_fn(|s| TetrominoPositions::from(Tetromino::new(s)));

        let sp_vec = all::<Shape>()
            .map(|s| ShapeStartingPositions {
                bot_positions: bot_positions_as_tp[s].clone(),
                player_position: player_positions[s].clone(),
            })
            .collect::<Vec<_>>();

        let shape_position_config = ShapePositionConfig {
            starting_positions: sp_vec.try_into().unwrap(),
        };
        let idx_to_shape =
            HashMap::from_iter(all::<Shape>().enumerate().map(|(i, s)| (i as u8, s)));
        let shape_to_idx_hash =
            HashMap::from_iter(idx_to_shape.iter().map(|(i, s)| (s.clone(), i.clone())));
        let shape_to_idx = EnumMap::from_fn(|s: Shape| shape_to_idx_hash[&s]);
        Self {
            bot_positions: EnumMap::from_fn(|s| compute_bot_start_positions_for_shape(s)),
            bot_positions_as_tp,
            player_positions,
            shape_position_config,
            idx_to_shape,
            shape_to_idx,
        }
    }

    pub fn bot_start_position(&self, s: Shape, cw_rotations: usize) -> &Tetromino {
        &self.bot_positions[s][cw_rotations]
    }

    pub fn bot_start_tps(&self, s: Shape, cw_rotations: usize) -> &TetrominoPositions {
        &self.bot_positions_as_tp[s][cw_rotations]
    }

    pub fn player_start_tps(&self, s: Shape) -> &TetrominoPositions {
        &self.player_positions[s]
    }
}

fn bot_start_position(s: Shape, cw_rotations: usize) -> Tetromino {
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
