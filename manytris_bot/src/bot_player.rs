use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::iter;

use ordered_float::OrderedFloat;

use crate::bot_shader::{BotShaderContext, UpcomingShapes};
use crate::bot_start_positions::StartPositions;
use crate::compute_types::MoveResultScore;
use manytris_core::bitmap_field::BitmapField;
use manytris_core::consts;
use manytris_core::field::Pos;
use manytris_core::game_state::{GameState, LockResult, TickMutation, TickResult};
use manytris_core::shapes::{Shape, Shift};

const VALIDATE_GPU_MOVES: bool = false;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MoveResult {
    pub moves: Vec<MovementDescriptor>,
    pub score: MoveResultScore,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovementDescriptor {
    pub shape: Shape,
    pub cw_rotations: usize,
    pub shifts_right: isize,
}

pub type ScoringKs = [f32; 4];

pub const BEST_BOT_KS: ScoringKs = [-2447.9722, 7782.121, -6099.498, -1970.1172];

impl MovementDescriptor {
    pub fn as_tick_mutations(&self, bsp: &StartPositions) -> Vec<TickMutation> {
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

pub fn select_next_move(
    gs: &GameState,
    ctx: &BotShaderContext,
    ks: &ScoringKs,
    mut search_depth: usize,
) -> Result<MoveResult, String> {
    search_depth -= 1;
    let mut usv = vec![gs.active_shape()];
    usv.extend_from_slice(&gs.upcoming_shapes());
    let us: UpcomingShapes = usv.try_into().unwrap();

    let results = ctx.compute_drop_search(search_depth, &us, &gs.make_bitmap_field(), |score| {
        OrderedFloat(weighted_result_score(score, ks))
    })?;

    let move_result = results.make_move_result();

    if VALIDATE_GPU_MOVES {
        let (_cpu_gs, cpu_score) = evaluate_moves_cpu(gs, &move_result.moves, &ctx.sp);
        assert_eq!(&cpu_score, &move_result.score);
    }

    Ok(move_result)
}

fn weighted_result_score(mrs: &MoveResultScore, ks: &ScoringKs) -> f32 {
    let game_over_f32 = if mrs.game_over { 1.0 } else { -1.0 };
    game_over_f32 * ks[0]
        + mrs.lines_cleared as f32 * ks[1]
        + mrs.height as f32 * ks[2]
        + mrs.covered as f32 * ks[3]
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
