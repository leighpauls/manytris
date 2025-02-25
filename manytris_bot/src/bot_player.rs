use std::iter;

use crate::bot_start_positions::START_POSITIONS;
use crate::compute_types::{ComputedDropConfig, MoveResultScore, UpcomingShapes};
use crate::{bot_cpu, BotContext, BotResults};
use anyhow::Result;
use manytris_core::consts;
use manytris_core::game_state::{GameState, TickMutation};
use manytris_core::shapes::{Shape, Shift};
use ordered_float::OrderedFloat;

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

pub struct ComputedDropSearchResults {
    pub search_depth: usize,
    pub upcoming_shapes: UpcomingShapes,
    pub drops: Vec<MovementDescriptor>,
    pub score: MoveResultScore,
}

pub type ScoringKs = [f32; 4];

pub const BEST_BOT_KS: ScoringKs = [-2447.9722, 7782.121, -6099.498, -1970.1172];

impl MovementDescriptor {
    pub fn as_tick_mutations(&self) -> Vec<TickMutation> {
        let (dir, num_shifts) = if self.shifts_right >= 0 {
            (Shift::Right, self.shifts_right as usize)
        } else {
            (Shift::Left, (-self.shifts_right) as usize)
        };
        iter::once(TickMutation::JumpToBotStartPosition(
            START_POSITIONS
                .bot_start_position(self.shape, self.cw_rotations)
                .clone(),
        ))
        .chain(iter::repeat(TickMutation::ShiftInput(dir)).take(num_shifts))
        .chain(iter::once(TickMutation::DropInput))
        .collect()
    }

    pub fn from_drop_config(drop_config: &ComputedDropConfig) -> Self {
        Self {
            shape: *START_POSITIONS
                .idx_to_shape
                .get(&drop_config.shape_idx)
                .unwrap(),
            cw_rotations: drop_config.cw_rotations as usize,
            shifts_right: drop_config.right_shifts as isize - (drop_config.left_shifts as isize),
        }
    }
}

impl ComputedDropSearchResults {
    // Select the best results from the score
    pub fn find_results<F: Fn(&MoveResultScore) -> OrderedFloat<f32>>(
        search_depth: usize,
        upcoming_shapes: UpcomingShapes,
        bot_results: &impl BotResults,
        scoring_fn: F,
    ) -> Self {
        let (start_idx, end_idx) = Self::idx_range(search_depth);
        let scores = bot_results.scores();
        assert_eq!(end_idx, scores.len());

        // Find the best score
        let (best_idx, best_score) = scores[start_idx..end_idx]
            .into_iter()
            .enumerate()
            .max_by_key(|(_i, s)| scoring_fn(s))
            .unwrap();

        let mut next_config_idx = start_idx + best_idx;
        let mut moves = vec![];
        let configs = bot_results.configs();
        loop {
            let cfg = &configs[next_config_idx];
            moves.insert(0, MovementDescriptor::from_drop_config(cfg));

            if cfg.src_field_idx == 0 {
                break;
            }
            next_config_idx = cfg.src_field_idx as usize - 1;
        }

        ComputedDropSearchResults {
            search_depth,
            upcoming_shapes: upcoming_shapes.clone(),
            drops: moves,
            score: best_score.clone(),
        }
    }

    pub fn make_move_result(&self) -> MoveResult {
        MoveResult {
            moves: self.drops.clone(),
            score: self.score.clone(),
        }
    }

    fn idx_range(search_depth: usize) -> (usize, usize) {
        let mut start_idx = 0;
        let mut end_idx = 0;
        for i in 0..search_depth {
            start_idx = end_idx;
            end_idx += consts::OUTPUTS_PER_INPUT_FIELD.pow(i as u32 + 1);
        }
        (start_idx, end_idx)
    }
}

pub fn select_next_move(
    gs: &GameState,
    ctx: &impl BotContext,
    ks: &ScoringKs,
    mut search_depth: usize,
) -> Result<MoveResult> {
    let mut usv = vec![gs.active_shape()];
    usv.extend_from_slice(&gs.upcoming_shapes());
    let us: UpcomingShapes = usv.try_into().unwrap();

    let bot_results = ctx.compute_drop_search(search_depth, &us, gs)?;

    let results =
        ComputedDropSearchResults::find_results(search_depth, us, &bot_results, |score| {
            OrderedFloat(weighted_result_score(score, ks))
        });

    let move_result = results.make_move_result();

    if VALIDATE_GPU_MOVES {
        let (_cpu_gs, cpu_score) = bot_cpu::evaluate_moves_cpu(gs, &move_result.moves);
        assert_eq!(&cpu_score, &move_result.score);
    }

    Ok(move_result)
}

fn weighted_result_score(mrs: &MoveResultScore, ks: &ScoringKs) -> f32 {
    let game_over_f32 = if mrs.is_game_over() { 1.0 } else { -1.0 };
    game_over_f32 * ks[0]
        + mrs.lines_cleared as f32 * ks[1]
        + mrs.height as f32 * ks[2]
        + mrs.covered as f32 * ks[3]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering() {
        assert!(MoveResultScore::init(false, 0, 0, 0) > MoveResultScore::init(true, 0, 0, 0));

        assert!(MoveResultScore::init(false, 1, 0, 0) > MoveResultScore::init(false, 0, 0, 0));

        assert!(MoveResultScore::init(false, 0, 0, 0) > MoveResultScore::init(false, 0, 1, 0));

        assert!(MoveResultScore::init(false, 0, 0, 0) > MoveResultScore::init(false, 0, 0, 1));
    }
}
