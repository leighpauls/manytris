use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::iter;

use bevy::render::render_resource::encase::private::RuntimeSizedArray;

use crate::bot_shader::BotShaderContext;
use crate::bot_start_positions::StartPositions;
use crate::compute_types::{BitmapField, MoveResultScore};
use crate::field::Pos;
use crate::game_state::{GameState, LockResult, TickMutation, TickResult};
use crate::shapes::{Shape, Shift};
use crate::{bot_shader, consts};

const VALIDATE_GPU_MOVES: bool = false;

#[derive(Clone)]
pub struct MoveResult {
    pub moves: Vec<MovementDescriptor>,
    pub score: MoveResultScore,
}

#[derive(Clone)]
pub struct MovementDescriptor {
    pub shape: Shape,
    pub next_shape: Shape,
    pub cw_rotations: usize,
    pub shifts_right: isize,
}

struct MovePassResult {
    moves: Vec<MovementDescriptor>,
    field: BitmapField,
    score: MoveResultScore,
}

impl MovementDescriptor {
    fn as_tick_mutations(&self, bsp: &StartPositions) -> Vec<TickMutation> {
        let (dir, num_shifts) = if self.shifts_right >= 0 {
            (Shift::Right, self.shifts_right as usize)
        } else {
            (Shift::Left, (-self.shifts_right) as usize)
        };
        iter::once(TickMutation::JumpToBotStartPosition(
            bsp.bot_start_position(self.shape, self.cw_rotations)
                .clone(),
        ))
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

fn create_movement_descriptor_passes(
    cur_shape: Shape,
    upcoming_shapes: &[Shape],
    depth: usize,
) -> Vec<Vec<MovementDescriptor>> {
    assert!(upcoming_shapes.len() > 0);
    assert!(depth > 0);

    let mut cur_pass = vec![];
    for cw_rotations in 0..4 {
        for shifts_right in -5..5 {
            cur_pass.push(MovementDescriptor {
                shape: cur_shape,
                next_shape: upcoming_shapes[0],
                cw_rotations,
                shifts_right,
            });
        }
    }

    let mut all_passes = vec![cur_pass];
    if depth > 1 {
        let later_moves =
            create_movement_descriptor_passes(upcoming_shapes[0], &upcoming_shapes[1..], depth - 1);
        all_passes.extend(later_moves);
    }

    return all_passes;
}

pub fn enumerate_moves(
    bot_context: &BotShaderContext,
    src_state: &GameState,
    depth: usize,
) -> Vec<MoveResult> {
    let passes = create_movement_descriptor_passes(
        src_state.active_shape(),
        &src_state.upcoming_shapes(),
        depth,
    );

    let mut layer_results = vec![MovePassResult {
        moves: vec![],
        field: src_state.make_bitmap_field(),
        score: MoveResultScore {
            game_over: false,
            lines_cleared: 0,
            height: 0,
            covered: 0,
        },
    }];
    for pass in passes {
        let list_of_results = layer_results
            .into_iter()
            .map(|mpr| {
                if mpr.score.game_over {
                    return vec![mpr];
                }
                let gpu_results = bot_context.evaluate_moves(&mpr.field, &pass).unwrap();
                gpu_results
                    .into_iter()
                    .zip(&pass)
                    .map(|((field, mut score), movement)| {
                        let mut moves = mpr.moves.clone();
                        moves.push(movement.clone());
                        score.lines_cleared += mpr.score.lines_cleared;
                        MovePassResult {
                            moves,
                            field,
                            score,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        layer_results = list_of_results.into_iter().flatten().collect();
    }

    if VALIDATE_GPU_MOVES {
        layer_results.iter().for_each(|mpr| {
            let (cpu_gs, cpu_score) = evaluate_moves_cpu(src_state, &mpr.moves, &bot_context.sp);
            let cpu_field = cpu_gs.make_bitmap_field();
            assert_eq!((&mpr.field, &mpr.score), (&cpu_field, &cpu_score));
        })
    }

    layer_results
        .into_iter()
        .map(|mpr| MoveResult {
            moves: mpr.moves,
            score: mpr.score,
        })
        .collect()
}

pub fn evaluate_moves_cpu(
    src_state: &GameState,
    moves: &[MovementDescriptor],
    sp: &StartPositions,
) -> (GameState, MoveResultScore) {
    let mut gs = src_state.clone();
    let mut game_over = false;
    let mut lines_cleared = 0;

    moves.iter().for_each(|md| {
        let tick_results = gs.tick_mutation(md.as_tick_mutations(sp));
        for tr in tick_results {
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
    });
    let cpu_field = gs.make_bitmap_field();
    let height = find_height(&cpu_field) as u8;
    let covered = find_covered(&cpu_field, height as i32) as u16;
    let score = MoveResultScore {
        game_over,
        lines_cleared,
        height,
        covered,
    };

    (gs, score)
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
        let mut y = height - 1;
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
