pub mod bot_cpu;
pub mod bot_player;
pub mod bot_start_positions;
pub mod compute_types;

use std::fmt::Debug;

use anyhow::Result;
use bot_player::MovementDescriptor;
use compute_types::{ComputedDropConfig, MoveResultScore, UpcomingShapes};
use manytris_core::{
    bitmap_field::BitmapField,
    consts,
    field::Pos,
    game_state::{GameState, LockResult, TickResult},
};

pub trait BotResults {
    fn configs(&self) -> &[ComputedDropConfig];
    fn scores(&self) -> &[MoveResultScore];
    fn fields(&self) -> &[BitmapField];
}

pub trait BotContext {
    type ResultType: BotResults;

    fn compute_drop_search(
        &self,
        search_depth: usize,
        upcoming_shapes: &UpcomingShapes,
        source_state: &GameState,
    ) -> Result<Self::ResultType>;
}

pub fn num_outputs(search_depth: usize) -> usize {
    let mut total_outputs = 0;
    for i in 0..(search_depth) {
        total_outputs += consts::OUTPUTS_PER_INPUT_FIELD.pow(i as u32 + 1);
    }
    total_outputs
}

pub struct BotResultsWraper<'a, T: BotResults>(&'a T);

impl<'a, T, U> PartialEq<BotResultsWraper<'a, U>> for BotResultsWraper<'a, T>
where
    T: BotResults,
    U: BotResults,
{
    fn eq(&self, other: &BotResultsWraper<U>) -> bool {
        self.0.configs() == other.0.configs()
            && self.0.scores() == other.0.scores()
            && self.0.fields() == other.0.fields()
    }
}

pub fn evaluate_moves_cpu(
    src_state: &GameState,
    moves: &[MovementDescriptor],
) -> (GameState, MoveResultScore, BitmapField) {
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

    (gs, score, cpu_field)
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
