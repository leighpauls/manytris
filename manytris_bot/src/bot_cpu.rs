use std::cmp::max;

use anyhow::Result;
use manytris_core::{
    bitmap_field::BitmapField,
    consts,
    field::Pos,
    game_state::{GameState, LockResult, TickResult},
    shapes::Shape,
};

use crate::{
    bot_player::MovementDescriptor,
    bot_start_positions::START_POSITIONS,
    compute_types::{ComputedDropConfig, MoveResultScore, UpcomingShapes},
    BotContext, BotResults,
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
    fn compute_drop_search(
        &self,
        search_depth: usize,
        upcoming_shapes: &UpcomingShapes,
        source_state: &GameState,
    ) -> Result<impl BotResults> {
        let configs = make_drop_configs_cpu(&upcoming_shapes[0..search_depth]);

        let (fields, scores) = eval_configs(source_state, configs.as_slice());

        Ok(CpuBotResults {
            configs,
            fields,
            scores,
        })
    }
}

pub fn make_drop_configs_cpu(shapes: &[Shape]) -> Vec<ComputedDropConfig> {
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

pub fn evaluate_moves_cpu(
    src_state: &GameState,
    moves: &[MovementDescriptor],
) -> (GameState, MoveResultScore) {
    let mut gs = src_state.clone();
    let mut game_over = false;
    let mut lines_cleared = 0;

    moves.iter().for_each(|md| {
        let tick_results = gs.tick_mutation(md.as_tick_mutations());
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
    let score = MoveResultScore::init(game_over, lines_cleared, height, covered);

    (gs, score)
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
        let (gs, score) = evaluate_moves_cpu(initial_state, moves.as_slice());

        debug_assert_eq!(fields.len(), config.dest_field_idx as usize);
        debug_assert_eq!(scores.len(), config.dest_field_idx as usize - 1);

        scores.push(score);
        fields.push(gs.make_bitmap_field());
    }

    (fields, scores)
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
