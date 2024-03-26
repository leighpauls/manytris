use crate::game_state::{DownResult, DownType, GameState, LockResult};
use crate::plugins::assets;
use crate::plugins::input::{InputEvent, InputType};
use crate::plugins::system_sets::{StartupSystems, UpdateSystems};
use bevy::prelude::*;
use std::time::Duration;

const LINES_PER_LEVEL: i32 = 10;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_root.in_set(StartupSystems::Root))
        .add_systems(Update, update_root_tick.in_set(UpdateSystems::RootTick));
}

#[derive(Component)]
pub struct GameRoot {
    pub game: GameState,
    lines_cleared: i32,
    lines_to_next_level: i32,
    level: i32,
    next_drop_time: Duration,
}

#[derive(Bundle)]
struct RootTransformBundle {
    transform: SpatialBundle,
    marker: GameRoot,
}

fn setup_root(mut commands: Commands, time: Res<Time<Fixed>>) {
    commands.spawn(RootTransformBundle {
        transform: SpatialBundle::from_transform(Transform::from_xyz(
            -assets::BLOCK_SIZE * 8.,
            -assets::BLOCK_SIZE * 11.,
            0.,
        )),
        marker: GameRoot::new(time.elapsed()),
    });
}

fn update_root_tick(
    mut q_root: Query<&mut GameRoot>,
    mut input_events: EventReader<InputEvent>,
    time: Res<Time<Fixed>>,
) {
    let mut game_root = q_root.single_mut();

    for event in input_events.read() {
        use InputType::*;
        let gs = &mut game_root.game;
        let lock_result = match event.input_type {
            ShiftEvent(s) => {
                gs.shift(s);
                None
            }
            RotateEvent(d) => {
                gs.rotate(d);
                None
            }
            DownEvent => {
                let down_type = if event.is_repeat {
                    DownType::HoldRepeat
                } else {
                    DownType::FirstPress
                };
                match gs.down(down_type) {
                    DownResult::Locked(lr) => Some(lr),
                    DownResult::StillActive => None,
                }
            }
            DropEvent => Some(gs.drop()),
            HoldEvent => {
                gs.hold();
                None
            }
        };

        if let Some(lr) = lock_result {
            game_root.apply_lock_result(&lr);
        }
    }

    let cur_time = time.elapsed();
    while cur_time > game_root.next_drop_time {
        if let DownResult::Locked(lr) = game_root.game.down(DownType::Gravity) {
            game_root.apply_lock_result(&lr);
        }

        let level = game_root.level;
        game_root.next_drop_time += time_to_drop(level);
    }

    if let Some(lr) = game_root.game.tick(cur_time) {
        game_root.apply_lock_result(&lr);
    }
}

impl GameRoot {
    fn new(start_time: Duration) -> Self {
        Self {
            game: GameState::new(),
            level: 1,
            lines_cleared: 0,
            lines_to_next_level: LINES_PER_LEVEL,
            next_drop_time: start_time + time_to_drop(1),
        }
    }

    fn apply_lock_result(&mut self, lr: &LockResult) {
        match lr {
            LockResult::GameOver => panic!("Game Over!!!"),
            LockResult::Ok { lines_cleared } => {
                self.lines_cleared += lines_cleared;
                self.lines_to_next_level -= lines_cleared;
                if self.lines_to_next_level <= 0 {
                    self.level += 1;
                    self.lines_to_next_level = LINES_PER_LEVEL;
                }
            }
        }
    }
}

fn time_to_drop(mut level: i32) -> Duration {
    level = i32::min(level, 20);
    let l = level as f64;
    let seconds = (0.8 - ((l - 1.) * 0.007)).powf(l - 1.);
    let millis = seconds * 1000.;
    Duration::from_millis(millis as u64)
}
