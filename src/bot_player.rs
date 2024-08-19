use crate::compute_types::{BitmapField, DropConfig, MoveResultScore, TetrominoPositions};
use bevy::prelude::KeyCode::ShiftLeft;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::iter;

use crate::bot_start_positions::bot_start_position;
use crate::field::Pos;
use crate::game_state::{GameState, LockResult, TickMutation, TickResult};
use crate::shapes::{Rot, Shape, Shift};
use crate::{bot_shader, consts};

const VALIDATE_GPU_MOVES: bool = false;

#[derive(Clone)]
pub struct MoveResult {
    pub gs: GameState,
    pub moves: Vec<TickMutation>,
    pub score: MoveResultScore,
}

pub struct MovementDescriptor {
    pub shape: Shape,
    pub next_shape: Shape,
    pub cw_rotations: usize,
    pub shifts_right: isize,
}

impl MovementDescriptor {
    fn as_tick_mutations(&self) -> Vec<TickMutation> {
        let (dir, num_shifts) = if self.shifts_right >= 0 {
            (Shift::Right, self.shifts_right as usize)
        } else {
            (Shift::Left, (-self.shifts_right) as usize)
        };
        iter::once(TickMutation::JumpToBotStartPosition(bot_start_position(
            self.shape,
            self.cw_rotations,
        )))
        .chain(iter::repeat(TickMutation::ShiftInput(dir)).take(num_shifts))
        .chain(iter::once(TickMutation::DropInput))
        .collect()
    }
}

pub type ScoringKs = [f32; 4];

pub fn weighted_result_score(mrs: &MoveResultScore, ks: &ScoringKs) -> f32 {
    let game_over_f32 = if mrs.game_over { 1.0 } else { -1.0 };
    game_over_f32 * ks[0]
        + mrs.lines_cleared as f32 * ks[1]
        + mrs.height as f32 * ks[2]
        + mrs.covered as f32 * ks[3]
}

pub fn enumerate_moves(src_state: &GameState, depth: usize) -> Vec<MoveResult> {
    // Create our options of mutation lists
    let mut all_moves = vec![];
    for cw_rotations in 0..4 {
        for shifts_right in -5..5 {
            all_moves.push(MovementDescriptor {
                shape: src_state.active_shape(),
                next_shape: src_state.next_shape(),
                cw_rotations,
                shifts_right,
            });
        }
    }

    let gpu_results =
        bot_shader::evaluate_moves(&src_state.make_bitmap_field(), &all_moves).unwrap();

    // Run each mutation list
    all_moves
        .into_iter()
        .enumerate()
        .map(|(i, cur_move)| {
            // CPU evaluation
            let mut gs = src_state.clone();
            let mutations = cur_move.as_tick_mutations();
            let results = gs.tick_mutation(mutations.clone());
            let mut game_over = false;
            let mut lines_cleared = 0;
            for tr in results {
                match tr {
                    TickResult::Lock(LockResult::GameOver) => {
                        game_over = true;
                    }
                    TickResult::Lock(LockResult::Ok { lines_cleared: lc }) => {
                        lines_cleared += lc as u8;
                    }
                    _ => {}
                }
            }

            let (gpu_field, gpu_score) = gpu_results.get(i).unwrap();
            // Compare against the GPU evaluation.
            assert_eq!(gpu_field, &gs.make_bitmap_field());

            let result_list: Vec<MoveResult> = if game_over || depth == 0 {
                let cpu_field = gs.make_bitmap_field();
                let height = find_height(&cpu_field) as u8;
                let covered = find_covered(&cpu_field, height as i32) as u16;
                let score = MoveResultScore {
                    game_over,
                    lines_cleared,
                    height,
                    covered,
                };
                assert_eq!(
                    (
                        gpu_field,
                        gpu_score.lines_cleared,
                        gpu_score.height,
                        gpu_score.game_over
                    ),
                    (&cpu_field, lines_cleared, height, game_over)
                );

                vec![MoveResult {
                    gs,
                    moves: mutations,
                    score,
                }]
            } else {
                let next_turns = enumerate_moves(&gs, depth - 1);
                next_turns
                    .into_iter()
                    .map(|mut mr| {
                        // Use the gamestate and move list of only the first move in the tree.
                        mr.gs = gs.clone();
                        mr.moves = mutations.clone();
                        mr.score.lines_cleared += lines_cleared;
                        mr
                    })
                    .collect()
            };
            result_list
        })
        .flatten()
        .collect()
}

fn find_height(cf: &BitmapField) -> i32 {
    for y in 0..consts::H {
        let mut empty_row = true;
        for x in 0..consts::W {
            if cf.occupied(&Pos { x, y }) {
                empty_row = false;
                break;
            }
        }
        if empty_row {
            return y;
        }
    }
    consts::H
}

fn find_covered(cf: &BitmapField, height: i32) -> i32 {
    let mut count = 0;
    for x in 0..consts::W {
        let mut y = height;
        // Find the top of this column
        while y > 0 {
            if cf.occupied(&Pos { x, y }) {
                break;
            }
            y -= 1;
        }

        // Count the holes
        y -= 1;
        while y >= 0 {
            if !cf.occupied(&Pos { x, y }) {
                count += 1;
            }
            y -= 1;
        }
    }
    count
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering() {
        assert!(
            MoveResultScore {
                game_over: false,
                lines_cleared: 0,
                height: 0,
                covered: 0
            } > MoveResultScore {
                game_over: true,
                lines_cleared: 0,
                height: 0,
                covered: 0
            }
        );

        assert!(
            MoveResultScore {
                game_over: false,
                lines_cleared: 1,
                height: 0,
                covered: 0
            } > MoveResultScore {
                game_over: false,
                lines_cleared: 0,
                height: 0,
                covered: 0
            }
        );

        assert!(
            MoveResultScore {
                game_over: false,
                lines_cleared: 0,
                height: 0,
                covered: 0
            } > MoveResultScore {
                game_over: false,
                lines_cleared: 0,
                height: 1,
                covered: 0
            }
        );

        assert!(
            MoveResultScore {
                game_over: false,
                lines_cleared: 0,
                height: 0,
                covered: 0
            } > MoveResultScore {
                game_over: false,
                lines_cleared: 0,
                height: 0,
                covered: 1
            }
        );
    }
}
