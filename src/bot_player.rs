use crate::consts;
use crate::field::{CompactField, Pos};
use crate::game_state::{BlockDisplayState, GameState, LockResult, TickMutation, TickResult};
use crate::shapes::{Rot, Shift};
use rand::{random, thread_rng, Rng, RngCore};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::iter;

pub struct MoveResult {
    pub gs: GameState,
    pub moves: Vec<TickMutation>,
    pub score: MoveResultScore,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct MoveResultScore {
    pub game_over: bool,
    pub lines_cleared: i32,
    pub height: i32,
    pub covered: i32,
}

pub type ScoringKs = [f32; 4];

pub fn weighted_result_score(mrs: &MoveResultScore, ks: &ScoringKs) -> f32 {
    let game_over_f32 = if mrs.game_over { 1.0 } else { -1.0 };
    game_over_f32 * ks[0]
        + mrs.lines_cleared as f32 * ks[1]
        + mrs.height as f32 * ks[2]
        + mrs.covered as f32 * ks[3]
}

pub fn enumerate_moves(src_state: &GameState) -> impl Iterator<Item = MoveResult> + '_ {
    // Create our options of mutation lists
    let mut mutations: Vec<Vec<TickMutation>> = vec![];
    for turns in 0..4 {
        let turn_iter = iter::repeat(TickMutation::RotateInput(Rot::Cw)).take(turns);

        for left_shifts in 0..5 {
            let new_mutations = turn_iter
                .clone()
                .chain(iter::repeat(TickMutation::ShiftInput(Shift::Left)).take(left_shifts))
                .chain(iter::once(TickMutation::DropInput))
                .collect();
            mutations.push(new_mutations);
        }
        for right_shifts in 1..6 {
            let new_mutations = turn_iter
                .clone()
                .chain(iter::repeat(TickMutation::ShiftInput(Shift::Right)).take(right_shifts))
                .chain(iter::once(TickMutation::DropInput))
                .collect();
            mutations.push(new_mutations);
        }
    }

    // Run each mutation list
    mutations.into_iter().map(|moves| {
        let mut gs = src_state.clone();
        let results = gs.tick_mutation(moves.clone());
        let mut game_over = false;
        let mut lines_cleared = 0;
        for tr in results {
            match tr {
                TickResult::Lock(LockResult::GameOver) => {
                    game_over = true;
                }
                TickResult::Lock(LockResult::Ok { lines_cleared: lc }) => {
                    lines_cleared += lc;
                }
                _ => {}
            }
        }

        let cf = gs.make_compact_field();
        let height = find_height(&cf);
        let covered = find_covered(&cf, height);
        let score = MoveResultScore {
            game_over,
            lines_cleared,
            height,
            covered,
        };
        MoveResult { gs, moves, score }
    })
}

fn find_height(cf: &CompactField) -> i32 {
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

fn find_covered(cf: &CompactField, height: i32) -> i32 {
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
