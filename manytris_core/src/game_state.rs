use crate::bitmap_field::BitmapField;
use crate::consts;
use crate::field::{Field, OccupiedBlock, Pos};
use crate::shapes::{Rot, Shape, Shift};
use crate::tetromino::Tetromino;
use crate::upcoming::UpcomingTetrominios;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct GameState {
    field: Field,
    active: Tetromino,
    upcoming: UpcomingTetrominios,
    garbage_queue: VecDeque<usize>,

    held: Option<Shape>,
    hold_used: bool,
}

pub enum BlockDisplayState {
    Empty,
    Occupied(OccupiedBlock),
    Active(Shape),
    Shadow(Shape),
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum LockResult {
    GameOver, // TODO: GameOver can occur during hold too
    Ok { lines_cleared: i32 },
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum DownType {
    FirstPress,
    HoldRepeat,
    Gravity,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum TickMutation {
    LockTimerExpired,
    DownInput(DownType),
    ShiftInput(Shift),
    RotateInput(Rot),
    DropInput,
    HoldInput,
    EnqueueTetromino(Shape),
    JumpToBotStartPosition(Tetromino),
    EnqueueGarbage(usize),
}

#[must_use]
pub enum TickResult {
    Lock(LockResult),
    RestartLockTimer,
    ClearLockTimer,
}

impl GameState {
    pub fn new(inital_shapes: Vec<Shape>) -> GameState {
        let mut upcoming = UpcomingTetrominios::new(inital_shapes);

        GameState {
            field: Field::new(),
            active: Tetromino::new(upcoming.take()),
            garbage_queue: VecDeque::default(),
            held: None,
            hold_used: false,
            upcoming,
        }
    }

    pub fn tick_mutation(&mut self, mutations: Vec<TickMutation>) -> Vec<TickResult> {
        use TickMutation::*;
        let mut result = vec![];

        for mutation in mutations {
            result.extend(match mutation {
                LockTimerExpired => self.lock_active_tetromino(),
                DownInput(dt) => self.down(dt),
                ShiftInput(shift) => self.shift(shift).into_iter().collect(),
                RotateInput(rot) => self.rotate(rot).into_iter().collect(),
                DropInput => self.drop(),
                HoldInput => self.hold(),
                EnqueueTetromino(shape) => {
                    self.upcoming.enqueue(shape);
                    vec![]
                }
                JumpToBotStartPosition(new_tet) => {
                    self.active = new_tet;
                    vec![]
                }
                EnqueueGarbage(lines) => {
                    self.enqueue_garbage(lines);
                    vec![]
                }
            });
        }
        result
    }

    /// Drop the active tetromino
    fn down(&mut self, down_type: DownType) -> Vec<TickResult> {
        match (self.active.down(), down_type) {
            (Some(new_t), _) if self.field.is_valid(&new_t) => {
                self.active = new_t;
                vec![self.update_lock_timer_for_movement()]
            }
            // Can't drop any further on the first press, lock it.
            (_, DownType::FirstPress) => self.lock_active_tetromino(),
            // Don't lock from gravity or repeat.
            (_, DownType::Gravity | DownType::HoldRepeat) => vec![],
        }
    }

    fn drop(&mut self) -> Vec<TickResult> {
        loop {
            match self.active.down() {
                Some(new_t) if self.field.is_valid(&new_t) => self.active = new_t,
                _ => break,
            };
        }
        self.lock_active_tetromino()
    }

    fn shift(&mut self, dir: Shift) -> Option<TickResult> {
        self.active = self
            .active
            .shift(dir)
            .filter(|new_t| self.field.is_valid(&new_t))?;
        Some(self.update_lock_timer_for_movement())
    }

    fn rotate(&mut self, dir: Rot) -> Option<TickResult> {
        self.active = self
            .active
            .rotation_options(dir)
            .into_iter()
            .filter(|t| self.field.is_valid(t))
            .next()?;
        Some(self.update_lock_timer_for_movement())
    }

    fn enqueue_garbage(&mut self, num_lines: usize) {
        for _ in 0..num_lines {
            self.garbage_queue.push_back(consts::GARBAGE_TURN_COUNT)
        }
    }

    pub fn get_garbage_element_countdown(&self, index: usize) -> Option<usize> {
        return self.garbage_queue.get(index).copied();
    }

    pub fn get_display_state(&self, p: &Pos) -> BlockDisplayState {
        if self.active.contains(p) {
            BlockDisplayState::Active(self.active.shape)
        } else if self.field.find_shadow(&self.active).contains(p) {
            BlockDisplayState::Shadow(self.active.shape)
        } else if let Some(color) = self.field.get_occupied_block(p) {
            BlockDisplayState::Occupied(color)
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

    pub fn make_bitmap_field(&self) -> BitmapField {
        self.field.make_bitmap_field()
    }

    pub fn active_shape(&self) -> Shape {
        self.active.shape
    }

    pub fn upcoming_shapes(&self) -> [Shape; consts::NUM_PREVIEWS] {
        self.upcoming.preview()
    }

    fn hold(&mut self) -> Vec<TickResult> {
        if self.hold_used {
            return vec![];
        }
        self.hold_used = true;

        let new_shape = if let Some(ref mut held_shape) = self.held {
            std::mem::replace(held_shape, self.active.shape)
        } else {
            self.held = Some(self.active.shape);
            self.upcoming.take()
        };

        let mut result = vec![];
        if !self.replace_active_tetromino(new_shape) {
            result.push(TickResult::Lock(LockResult::GameOver));
        }
        result.push(self.update_lock_timer_for_movement());
        result
    }

    fn update_lock_timer_for_movement(&mut self) -> TickResult {
        if self.field.is_lockable(&self.active) {
            TickResult::RestartLockTimer
        } else {
            TickResult::ClearLockTimer
        }
    }

    fn lock_active_tetromino(&mut self) -> Vec<TickResult> {
        self.hold_used = false;
        let mut result = vec![TickResult::ClearLockTimer];

        let lines_cleared = self.field.apply_tetrominio(&self.active);
        let next_shape = self.upcoming.take();

        while (!self.garbage_queue.is_empty()) && self.garbage_queue[0] == 1 {
            self.field.apply_garbage();
            self.garbage_queue.pop_front();
        }

        self.garbage_queue.iter_mut().for_each(|cnt| {
            *cnt -= 1;
        });

        result.push(TickResult::Lock(
            if self.replace_active_tetromino(next_shape) {
                LockResult::Ok { lines_cleared }
            } else {
                LockResult::GameOver
            },
        ));
        result
    }

    /// Place the new tetromino, return true if it has a valid placement.
    fn replace_active_tetromino(&mut self, shape: Shape) -> bool {
        self.active = Tetromino::new(shape);
        self.field.is_valid(&self.active)
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("----------\n")?;
        for y in (0..consts::H).rev() {
            for x in 0..consts::W {
                let ch = match self.get_display_state(&Pos { x, y }) {
                    BlockDisplayState::Empty => " ",
                    BlockDisplayState::Occupied(_) => "X",
                    BlockDisplayState::Active(_) => "O",
                    BlockDisplayState::Shadow(_) => " ",
                };
                f.write_str(ch)?;
            }
            f.write_str("\n")?;
        }
        f.write_str("----------\n")?;
        Ok(())
    }
}
