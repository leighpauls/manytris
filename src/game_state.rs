use crate::consts;
use crate::field::{Field, Pos};
use crate::shapes::{Rot, Shape, Shift};
use crate::tetromino::Tetromino;
use crate::upcoming::UpcomingTetrominios;

pub struct GameState {
    field: Field,
    active: Tetromino,
    upcoming: UpcomingTetrominios,
    held: Option<Shape>,
    hold_used: bool,
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

impl GameState {
    pub fn new() -> GameState {
        let mut upcoming = UpcomingTetrominios::new();

        return GameState {
            field: Field::new(),
            active: Tetromino::new(upcoming.take()),
            held: None,
            hold_used: false,
            upcoming,
        };
    }

    /// Drop the active tetromino, return True if it locks.
    pub fn down(&mut self, is_repeat: bool) -> DownResult {
        match (self.active.down(), is_repeat) {
            (Some(new_t), _) if self.field.is_valid(&new_t) => {
                self.active = new_t;
                DownResult::StillActive
            }
            (_, false) => DownResult::Locked(self.lock_active_tetromino()),
            // Don't lock when repeating the down input.
            (_, true) => DownResult::StillActive,
        }
    }

    pub fn drop(&mut self) -> LockResult {
        loop {
            match self.down(false) {
                DownResult::StillActive => (),
                DownResult::Locked(res) => return res,
            }
        }
    }

    pub fn shift(&mut self, dir: Shift) -> Option<()> {
        let new_t = self.active.shift(dir)?;
        if self.field.is_valid(&new_t) {
            self.active = new_t;
            return Some(());
        }
        None
    }

    pub fn rotate(&mut self, dir: Rot) {
        for new_t in self.active.rotate(dir) {
            if self.field.is_valid(&new_t) {
                self.active = new_t;
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
    }

    fn lock_active_tetromino(&mut self) -> LockResult {
        self.hold_used = false;
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
