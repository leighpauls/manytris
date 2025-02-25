pub mod bot_cpu;
pub mod bot_player;
pub mod bot_start_positions;
pub mod compute_types;

use anyhow::Result;
use compute_types::{ComputedDropConfig, MoveResultScore, UpcomingShapes};
use manytris_core::{bitmap_field::BitmapField, consts, game_state::GameState};

pub trait BotResults {
    fn configs(&self) -> &[ComputedDropConfig];
    fn scores(&self) -> &[MoveResultScore];
    fn fields(&self) -> &[BitmapField];
}

pub trait BotContext {
    fn compute_drop_search(
        &self,
        search_depth: usize,
        upcoming_shapes: &UpcomingShapes,
        source_state: &GameState,
    ) -> Result<impl BotResults>;
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
