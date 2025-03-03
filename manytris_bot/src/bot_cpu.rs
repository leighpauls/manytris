use std::cmp::max;

use anyhow::Result;
use manytris_core::{bitmap_field::BitmapField, game_state::GameState, shapes::Shape};

use crate::{
    bot_player::MovementDescriptor,
    bot_start_positions::START_POSITIONS,
    compute_types::{ComputedDropConfig, MoveResultScore, UpcomingShapes},
    evaluate_moves_cpu, BotContext, BotResults,
};

pub struct CpuBotContext;

#[derive(Default)]
pub struct CpuBotResults {
    configs: Vec<ComputedDropConfig>,
    scores: Vec<MoveResultScore>,
    fields: Vec<BitmapField>,
}

impl BotResults for CpuBotResults {
    fn configs(&self) -> &[ComputedDropConfig] {
        &self.configs
    }
    fn scores(&self) -> &[MoveResultScore] {
        &self.scores
    }
    fn fields(&self) -> &[BitmapField] {
        &self.fields
    }
}

impl BotContext for CpuBotContext {
    type ResultType = CpuBotResults;

    fn compute_drop_search(
        &self,
        search_depth: usize,
        upcoming_shapes: &UpcomingShapes,
        source_state: &GameState,
    ) -> Result<CpuBotResults> {
        let configs = make_drop_configs_cpu(&upcoming_shapes[0..search_depth]);

        let (fields, scores) = eval_configs(source_state, configs.as_slice());

        Ok(CpuBotResults {
            configs,
            fields,
            scores,
        })
    }
}

fn make_drop_configs_cpu(shapes: &[Shape]) -> Vec<ComputedDropConfig> {
    let mut res = vec![];
    let mut prev_gen_range = 0..1;

    for shape in shapes {
        let cur_start = res.len() as u32 + 1;
        for src_field_idx in prev_gen_range.clone() {
            for cw_rotations in 0..4 {
                for shifts in 0..10 {
                    let left_shifts = max(4 - shifts, 0) as u8;
                    let right_shifts = max(shifts - 4, 0) as u8;
                    let dest_field_idx = (res.len() + 1) as u32;
                    res.push(ComputedDropConfig {
                        shape_idx: START_POSITIONS.shape_to_idx[*shape],
                        cw_rotations,
                        left_shifts,
                        right_shifts,
                        src_field_idx,
                        dest_field_idx,
                    });
                }
            }
        }
        prev_gen_range = cur_start..(res.len() as u32 + 1);
    }
    res
}

fn eval_configs(
    initial_state: &GameState,
    configs: &[ComputedDropConfig],
) -> (Vec<BitmapField>, Vec<MoveResultScore>) {
    let mut fields = Vec::with_capacity(configs.len() + 1);
    fields.push(initial_state.make_bitmap_field());

    let mut scores = Vec::with_capacity(configs.len());

    for config in configs {
        let mut cur_config = config;
        let mut moves = vec![];
        loop {
            moves.push(MovementDescriptor::from_drop_config(cur_config));
            if cur_config.src_field_idx == 0 {
                break;
            } else {
                cur_config = &configs[cur_config.src_field_idx as usize - 1];
            }
        }

        moves.reverse();
        let (_, score, field) = evaluate_moves_cpu(initial_state, moves.as_slice());

        debug_assert_eq!(fields.len(), config.dest_field_idx as usize);
        debug_assert_eq!(scores.len(), config.dest_field_idx as usize - 1);

        scores.push(score);
        fields.push(field);
    }

    (fields, scores)
}
