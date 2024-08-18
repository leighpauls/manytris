use std::time::Duration;

pub const W: i32 = 10;
/// Height of the visible game
pub const H: i32 = 22;
/// Max height considerable, including invisible positions "above" the game.
pub const MAX_H: i32 = 26;

pub const NUM_POSITIONS: usize = (W * MAX_H) as usize;

pub const PREVIEW_H: i32 = 2;

pub const NUM_PREVIEWS: usize = 6;

pub const LOCK_TIMER_DURATION: Duration = Duration::from_millis(500);
