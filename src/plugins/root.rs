use std::time::Duration;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::bot_start_positions::StartPositions;
use crate::consts;
use crate::game_state::{DownType, GameState, LockResult, TickMutation, TickResult};
use crate::plugins::assets;
use crate::plugins::input::{InputEvent, InputType};
use crate::plugins::system_sets::{StartupSystems, UpdateSystems};
use crate::shapes::Shape;

const LINES_PER_LEVEL: i32 = 10;

pub fn common_plugin(app: &mut App) {
    app.add_systems(Startup, setup_root.in_set(StartupSystems::Root))
        .add_systems(Update, update_root_tick.in_set(UpdateSystems::RootTick))
        .add_event::<TickEvent>()
        .add_event::<LockEvent>()
        .add_event::<SendControlEvent>()
        .add_event::<ReceiveControlEvent>()
        .insert_resource(StartPositionRes(StartPositions::new()));
}

pub fn client_plugin(app: &mut App) {
    app.add_systems(
        Update,
        produce_tick_events.in_set(UpdateSystems::LocalEventProducers),
    );
}

pub fn stand_alone_plugin(app: &mut App) {
    app.add_systems(
        Startup,
        setup_start_standalone_game.in_set(StartupSystems::AfterRoot),
    )
    .add_systems(
        Update,
        produce_tick_events.in_set(UpdateSystems::LocalEventProducers),
    );
}

#[derive(Resource)]
pub struct StartPositionRes(pub StartPositions);

#[derive(Component)]
pub struct GameRoot {
    pub active_game: Option<ActiveGame>,
}

pub struct ActiveGame {
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

#[derive(Clone, Event, Deserialize, Serialize, Debug)]
pub struct TickEvent {
    pub mutation: TickMutation,
    pub local: bool,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum ControlEvent {
    JoinRequest,
    SnapshotResponse(GameState),
}

#[derive(Event)]
pub struct SendControlEvent(pub ControlEvent);

#[derive(Event)]
pub struct ReceiveControlEvent(pub ControlEvent);

impl TickEvent {
    pub fn new_local(mutation: TickMutation) -> Self {
        Self {
            mutation,
            local: true,
        }
    }

    pub fn new_remote(mutation: TickMutation) -> Self {
        Self {
            mutation,
            local: false,
        }
    }
}

#[derive(Event, Deserialize, Serialize)]
pub struct LockEvent(pub LockResult);

fn setup_root(mut commands: Commands) {
    commands.spawn(RootTransformBundle {
        transform: SpatialBundle::from_transform(Transform::from_xyz(
            -assets::BLOCK_SIZE * 8.,
            -assets::BLOCK_SIZE * 11.,
            0.,
        )),
        marker: GameRoot { active_game: None },
    });
}

fn setup_start_standalone_game(mut q_root: Query<&mut GameRoot>, time: Res<Time<Fixed>>) {
    let start_time = time.elapsed();
    q_root.single_mut().active_game = Some(ActiveGame::new(start_time));
}

fn produce_tick_events(
    mut input_events: EventReader<InputEvent>,
    time: Res<Time<Fixed>>,
    mut q_root: Query<&mut GameRoot>,
    mut tick_event_writer: EventWriter<TickEvent>,
    sp: Res<StartPositionRes>,
) {
    let mut game_root = q_root.single_mut();
    let Some(game) = &mut game_root.active_game else {
        return;
    };

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
        JumpToBotStartPositionEvent => {
            JumpToBotStartPosition(sp.0.bot_start_position(game.game.active_shape(), 0).clone())
        }
    }));

    let cur_time = time.elapsed();
    while cur_time > game.next_drop_time {
        tick_events.push(DownInput(DownType::Gravity));
        let level = game.level;
        game.next_drop_time += time_to_drop(level);
    }

    if game.lock_timer_target.filter(|t| t <= &cur_time).is_some() {
        tick_events.push(LockTimerExpired);
    }
    tick_event_writer.send_batch(
        tick_events
            .into_iter()
            .map(|mutation| TickEvent::new_local(mutation)),
    );
}

fn update_root_tick(
    mut q_root: Query<&mut GameRoot>,
    mut control_event_reader: EventReader<ReceiveControlEvent>,
    mut control_event_writer: EventWriter<SendControlEvent>,
    mut tick_event_reader: EventReader<TickEvent>,
    mut lock_event_writer: EventWriter<LockEvent>,
    time: Res<Time<Fixed>>,
) {
    let mut game_root = q_root.single_mut();
    let cur_time = time.elapsed();

    for rce in control_event_reader.read() {
        let ReceiveControlEvent(ce) = rce;
        match ce {
            ControlEvent::JoinRequest => {
                control_event_writer.send(game_root.handle_join_request(cur_time));
            }
            ControlEvent::SnapshotResponse(gs) => {
                game_root.handle_snapshot_response(gs.clone(), cur_time);
            }
        }
    }

    let Some(active_game) = &mut game_root.active_game else {
        return;
    };

    let events = tick_event_reader
        .read()
        .into_iter()
        .map(|e| e.mutation.clone())
        .collect();

    for tick_result in active_game.game.tick_mutation(events) {
        use TickResult::*;
        match tick_result {
            Lock(lr) => {
                println!("Lock result: {:?}", lr);
                lock_event_writer.send(LockEvent(lr.clone()));
                active_game.apply_lock_result(&lr);
            }
            RestartLockTimer => {
                active_game.lock_timer_target = Some(cur_time + consts::LOCK_TIMER_DURATION);
            }
            ClearLockTimer => {
                active_game.lock_timer_target = None;
            }
        }
    }
}

impl GameRoot {
    fn handle_join_request(&mut self, cur_time: Duration) -> SendControlEvent {
        if let None = self.active_game {
            self.active_game = Some(ActiveGame::new(cur_time));
        }
        SendControlEvent(ControlEvent::SnapshotResponse(
            self.active_game.as_ref().unwrap().game.clone(),
        ))
    }

    fn handle_snapshot_response(&mut self, game_state: GameState, cur_time: Duration) {
        if self.active_game.is_some() {
            eprintln!("Overwriting current game with snapshot from server!");
        }
        self.active_game = Some(ActiveGame::from_snapshot(game_state, cur_time));
    }
}

impl ActiveGame {
    fn new(start_time: Duration) -> Self {
        let initial_states = enum_iterator::all::<Shape>()
            .chain(enum_iterator::all::<Shape>())
            .collect();
        Self::from_snapshot(GameState::new(initial_states), start_time)
    }

    fn from_snapshot(gs: GameState, start_time: Duration) -> Self {
        Self {
            game: gs,
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
