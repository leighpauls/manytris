use crate::consts;
use crate::field::{Field, Pos};
use crate::shapes::{Rot, Shape, Shift};
use crate::tetromino::Tetromino;
use crate::upcoming::UpcomingTetrominios;
use std::time::Duration;

pub struct GameState {
    field: Field,
    active: Tetromino,
    upcoming: UpcomingTetrominios,

    held: Option<Shape>,
    hold_used: bool,

    lock_timer_reset_requested: bool,
    lock_timer_target: Option<Duration>,
}

pub enum BlockDisplayState {
    Empty,
    Occupied(Shape),
    Active(Shape),
    Shadow(Shape),
}

#[must_use]
pub enum DownResult {
    StillActive,
    Locked(LockResult),
}

#[must_use]
pub enum LockResult {
    GameOver,
    Ok { lines_cleared: i32 },
}

pub enum DownType {
    FirstPress,
    HoldRepeat,
    Gravity,
}

impl GameState {
    pub fn new() -> GameState {
        let mut upcoming = UpcomingTetrominios::new();

        return GameState {
            field: Field::new(),
            active: Tetromino::new(upcoming.take()),
            held: None,
            hold_used: false,
            upcoming,
            lock_timer_reset_requested: false,
            lock_timer_target: None,
        };
    }

    /// Drop the active tetromino, return True if it locks.
    pub fn down(&mut self, down_type: DownType) -> DownResult {
        match (self.active.down(), down_type) {
            (Some(new_t), _) if self.field.is_valid(&new_t) => {
                self.active = new_t;
                self.update_lock_timer_for_movement();
                DownResult::StillActive
            }
            (_, DownType::FirstPress) => DownResult::Locked(self.lock_active_tetromino()),
            // Don't from gravity or repeat.
            (_, DownType::Gravity | DownType::HoldRepeat) => DownResult::StillActive,
        }
    }

    pub fn drop(&mut self) -> LockResult {
        loop {
            match self.down(DownType::FirstPress) {
                DownResult::StillActive => (),
                DownResult::Locked(res) => return res,
            }
        }
    }

    pub fn shift(&mut self, dir: Shift) -> Option<()> {
        let new_t = self.active.shift(dir)?;
        if self.field.is_valid(&new_t) {
            self.active = new_t;
            self.update_lock_timer_for_movement();
            return Some(());
        }
        None
    }

    pub fn rotate(&mut self, dir: Rot) {
        for new_t in self.active.rotate(dir) {
            if self.field.is_valid(&new_t) {
                self.active = new_t;
                self.update_lock_timer_for_movement();
                return;
            }
        }
    }

    pub fn get_display_state(&self, p: &Pos) -> BlockDisplayState {
        if self.active.contains(p) {
            BlockDisplayState::Active(self.active.shape)
        } else if self.field.find_shadow(&self.active).contains(p) {
            BlockDisplayState::Shadow(self.active.shape)
        } else if let Some(shape) = self.field.get_occupied_block(p) {
            BlockDisplayState::Occupied(shape)
        } else {
            BlockDisplayState::Empty
        }
    }

    pub fn previews(&self) -> [Tetromino; consts::NUM_PREVIEWS] {
        self.upcoming
            .preview()
            .map(|shape| Tetromino::for_preview(shape))
    }

    pub fn held_tetromino(&self) -> Option<Tetromino> {
        Some(Tetromino::for_preview(self.held?))
    }

    pub fn tick(&mut self, cur_time: Duration) -> Option<LockResult> {
        if self.lock_timer_reset_requested {
            self.lock_timer_reset_requested = false;
            self.lock_timer_target = Some(cur_time + consts::LOCK_TIMER_DURATION);
        }

        match self.lock_timer_target {
            Some(target) if target <= cur_time => {
                self.lock_timer_target = None;
                Some(self.lock_active_tetromino())
            }
            _ => None,
        }
    }

    pub fn hold(&mut self) {
        if self.hold_used {
            return;
        }
        self.hold_used = true;

        let new_shape = if let Some(ref mut held_shape) = self.held {
            std::mem::replace(held_shape, self.active.shape)
        } else {
            self.held = Some(self.active.shape);
            self.upcoming.take()
        };
        self.replace_active_tetromino(new_shape);
        self.update_lock_timer_for_movement();
    }

    fn update_lock_timer_for_movement(&mut self) {
        if self.field.is_lockable(&self.active) {
            self.lock_timer_reset_requested = true;
        } else {
            self.lock_timer_reset_requested = false;
            self.lock_timer_target = None;
        }
    }

    fn lock_active_tetromino(&mut self) -> LockResult {
        self.hold_used = false;
        self.lock_timer_reset_requested = false;
        self.lock_timer_target = None;

        let lines_cleared = self.field.apply_tetrominio(&self.active);
        let next_shape = self.upcoming.take();
        if self.replace_active_tetromino(next_shape) {
            LockResult::Ok { lines_cleared }
        } else {
            LockResult::GameOver
        }
    }

    /// Place the new tetromino, return true if it has a valid placement.
    fn replace_active_tetromino(&mut self, shape: Shape) -> bool {
        self.active = Tetromino::new(shape);
        self.field.is_valid(&self.active)
    }
}
