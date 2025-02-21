pub mod bot_player;
pub mod bot_start_positions;
pub mod compute_types;

use anyhow::Result;
use compute_types::{ComputedDropConfig, MoveResultScore, UpcomingShapes};
use manytris_core::{bitmap_field::BitmapField, consts};

pub trait BotResults {
    fn configs(&self) -> &[ComputedDropConfig];
    fn scores(&self) -> &[MoveResultScore];
}

pub trait BotContext {
    fn compute_drop_search(
        &self,
        search_depth: usize,
        upcoming_shapes: &UpcomingShapes,
        source_field: &BitmapField,
    ) -> Result<impl BotResults>;
}

pub fn num_outputs(search_depth: usize) -> usize {
    let mut total_outputs = 0;
    for i in 0..(search_depth + 1) {
        total_outputs += consts::OUTPUTS_PER_INPUT_FIELD.pow(i as u32 + 1);
    }
    total_outputs
}
