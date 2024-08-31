use std::time::Duration;
use crate::bot_player::ScoringKs;

pub const W: i32 = 10;
/// Height of the visible game
pub const H: i32 = 22;
/// Max height considerable, including invisible positions "above" the game.
pub const MAX_H: i32 = 26;

pub const NUM_POSITIONS: usize = (W * MAX_H) as usize;

pub const PREVIEW_H: i32 = 2;

pub const NUM_PREVIEWS: usize = 6;

pub const NUM_SHAPES: usize = 7;

pub const MAX_SEARCH_DEPTH: usize = NUM_PREVIEWS;

pub const ROTATIONS_PER_SHAPE: usize = 4;
pub const SHIFTS_PER_ROTATION: usize = 10;
pub const OUTPUTS_PER_INPUT_FIELD: usize = ROTATIONS_PER_SHAPE * SHIFTS_PER_ROTATION;

pub const LOCK_TIMER_DURATION: Duration = Duration::from_millis(500);

pub const BEST_BOT_KS: ScoringKs = [-2447.9722, 7782.121, -6099.498, -1970.1172];