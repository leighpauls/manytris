use std::time::Duration;

pub const W: i32 = 10;
pub const H: i32 = 22;

pub const NUM_POSITIONS: usize = (W * H) as usize;

pub const PREVIEW_H: i32 = 2;

pub const NUM_PREVIEWS: usize = 6;

pub const LOCK_TIMER_DURATION: Duration = Duration::from_millis(500);
