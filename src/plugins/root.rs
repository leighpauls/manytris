use crate::consts;
use crate::game_state::{DownType, GameState, LockResult, TickMutation, TickResult};
use crate::plugins::assets;
use crate::plugins::input::{InputEvent, InputType};
use crate::plugins::system_sets::{StartupSystems, UpdateSystems};
use crate::shapes::Shape;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const LINES_PER_LEVEL: i32 = 10;

pub fn common_plugin(app: &mut App) {
    app.add_systems(Startup, setup_root.in_set(StartupSystems::Root))
        .add_systems(Update, update_root_tick.in_set(UpdateSystems::RootTick))
        .add_event::<TickEvent>()
        .add_event::<LockEvent>();
}

pub fn client_plugin(app: &mut App) {
    app.add_systems(
        Update,
        produce_tick_events.in_set(UpdateSystems::LocalEventProducers),
    );
}

#[derive(Component)]
pub struct GameRoot {
    pub game: GameState,
    pub lines_cleared: i32,
    pub level: i32,
    lines_to_next_level: i32,
    next_drop_time: Duration,
    lock_timer_target: Option<Duration>,
}

#[derive(Bundle)]
struct RootTransformBundle {
    transform: SpatialBundle,
    marker: GameRoot,
}

#[derive(Event, Deserialize, Serialize, Debug)]
pub struct TickEvent(pub TickMutation);

#[derive(Event, Deserialize, Serialize)]
pub struct LockEvent(pub LockResult);

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

fn produce_tick_events(
    mut input_events: EventReader<InputEvent>,
    time: Res<Time<Fixed>>,
    mut q_root: Query<&mut GameRoot>,
    mut tick_event_writer: EventWriter<TickEvent>,
) {
    let mut game_root = q_root.single_mut();

    let mut tick_events = vec![];

    use InputType::*;
    use TickMutation::*;

    tick_events.extend(input_events.read().map(|e| match e.input_type {
        ShiftEvent(s) => ShiftInput(s),
        RotateEvent(r) => RotateInput(r),
        DownEvent => DownInput(if e.is_repeat {
            DownType::HoldRepeat
        } else {
            DownType::FirstPress
        }),
        DropEvent => DropInput,
        HoldEvent => HoldInput,
    }));

    let cur_time = time.elapsed();
    while cur_time > game_root.next_drop_time {
        tick_events.push(DownInput(DownType::Gravity));
        let level = game_root.level;
        game_root.next_drop_time += time_to_drop(level);
    }

    if game_root
        .lock_timer_target
        .filter(|t| t <= &cur_time)
        .is_some()
    {
        tick_events.push(LockTimerExpired);
    }
    tick_event_writer.send_batch(tick_events.into_iter().map(|e| TickEvent(e)));
}

fn update_root_tick(
    mut q_root: Query<&mut GameRoot>,
    mut tick_event_reader: EventReader<TickEvent>,
    mut lock_event_writer: EventWriter<LockEvent>,
    time: Res<Time<Fixed>>,
) {
    let mut game_root = q_root.single_mut();
    let cur_time = time.elapsed();
    let events = tick_event_reader
        .read()
        .into_iter()
        .map(|e| e.0.clone())
        .collect();

    for tick_result in game_root.game.tick_mutation(events) {
        use TickResult::*;
        match tick_result {
            Lock(lr) => {
                lock_event_writer.send(LockEvent(lr.clone()));
                game_root.apply_lock_result(&lr);
            }
            RestartLockTimer => {
                game_root.lock_timer_target = Some(cur_time + consts::LOCK_TIMER_DURATION);
            }
            ClearLockTimer => {
                game_root.lock_timer_target = None;
            }
        }
    }
}

impl GameRoot {
    fn new(start_time: Duration) -> Self {
        let initial_states = enum_iterator::all::<Shape>()
            .chain(enum_iterator::all::<Shape>())
            .collect();
        Self {
            game: GameState::new(initial_states),
            level: 1,
            lines_cleared: 0,
            lines_to_next_level: LINES_PER_LEVEL,
            next_drop_time: start_time + time_to_drop(1),
            lock_timer_target: None,
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
    let micros = (seconds * 1_000_000.) as u64;
    Duration::from_micros(micros)
}
