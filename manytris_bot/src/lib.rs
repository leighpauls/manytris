pub mod compute_types;
pub mod bot_start_positions;
pub mod bot_player;

use bot_start_positions::StartPositions;
use compute_types::{ComputedDropConfig, MoveResultScore, UpcomingShapes};
use manytris_core::bitmap_field::BitmapField;


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
    ) -> Result<impl BotResults, String>;

    fn sp(&self) -> &StartPositions;
}

