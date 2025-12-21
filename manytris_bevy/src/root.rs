use crate::game_container::LocalGameRoot;
use crate::input::{InputEvent, InputType};
use crate::shape_producer::ShapeProducer;
use crate::states;
use crate::states::{is_paused, is_unpaused, PauseState, PlayingState};
use crate::system_sets::UpdateSystems;
use bevy::prelude::*;
use manytris_core::consts;
use manytris_core::game_state::{DownType, GameState, LockResult, TickMutation, TickResult};
use manytris_core::shapes::Shape;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;
use uuid::Uuid;

const LINES_PER_LEVEL: i32 = 10;

/// Resource to store timer state when paused
#[derive(Resource, Default)]
struct PauseTimerState {
    pause_time: Option<Duration>,
    remaining_drop_time: Option<Duration>,
    remaining_lock_time: Option<Duration>,
}

/// This plugin must be used for all executable variants.
pub fn common_plugin(app: &mut App) {
    app.init_resource::<PauseTimerState>()
        .add_event::<InputEvent>()
        .add_event::<TickEvent>()
        .add_event::<LockEvent>()
        .add_systems(
            Update,
            (
                save_timer_state_on_pause
                    .run_if(resource_changed::<PauseState>)
                    .run_if(is_paused)
                    .run_if(in_state(PlayingState::Playing))
                    .run_if(states::is_stand_alone),
                restore_timer_state_on_unpause
                    .run_if(resource_changed::<PauseState>)
                    .run_if(is_unpaused)
                    .run_if(in_state(PlayingState::Playing))
                    .run_if(states::is_stand_alone),
                produce_tick_events
                    .in_set(UpdateSystems::LocalEventProducers)
                    .run_if(in_state(PlayingState::Playing))
                    .run_if(states::is_client)
                    .run_if(is_unpaused),
                update_root_tick
                    .in_set(UpdateSystems::RootTick)
                    .run_if(in_state(PlayingState::Playing))
                    .run_if(is_unpaused),
            )
                .chain(),
        );
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct GameRoot {
    pub game_id: GameId,
    pub active_game: ActiveGame,
}

pub struct ActiveGame {
    pub game: GameState,
    pub lines_cleared: i32,
    pub level: i32,
    lines_to_next_level: i32,
    next_drop_time: Duration,
    lock_timer_target: Option<Duration>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TickMutationMessage {
    pub mutation: TickMutation,
    pub game_id: GameId,
}

#[derive(Clone, Event, Deserialize, Serialize, Debug)]
pub struct TickEvent {
    pub mutation: TickMutationMessage,
    pub local: bool,
}

impl TickEvent {
    pub fn new_local(mutation: TickMutationMessage) -> Self {
        Self {
            mutation,
            local: true,
        }
    }

    pub fn new_remote(mutation: TickMutationMessage) -> Self {
        Self {
            mutation,
            local: false,
        }
    }
}

#[derive(Deserialize, Serialize, Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GameId(Uuid);

impl GameId {
    pub fn new() -> Self {
        GameId(Uuid::new_v4())
    }
}

#[derive(Event, Deserialize, Serialize)]
pub struct LockEvent {
    pub game_id: GameId,
    pub lock_result: LockResult,
}

pub fn create_new_root(
    commands: &mut Commands,
    container_entity: Entity,
    transform: Transform,
    cur_time: Duration,
    shape_producer: &mut ShapeProducer,
) -> (GameState, GameId, Entity) {
    let game_id = GameId::new();
    let initial_shapes = (0..consts::NUM_PREVIEWS * 2)
        .into_iter()
        .map(|_| shape_producer.take(&game_id))
        .collect();

    let active_game = ActiveGame::new(cur_time, initial_shapes);
    let game_state = active_game.game.clone();
    let entity = spawn_root(commands, container_entity, transform, active_game, game_id);
    (game_state, game_id, entity)
}

pub fn create_root_from_snapshot(
    commands: &mut Commands,
    container_entity: Entity,
    transform: Transform,
    gs: GameState,
    cur_time: Duration,
    game_id: GameId,
) -> Entity {
    let active_game = ActiveGame::from_snapshot(gs, cur_time);
    spawn_root(commands, container_entity, transform, active_game, game_id)
}

fn spawn_root(
    commands: &mut Commands,
    container_entitiy: Entity,
    transform: Transform,
    active_game: ActiveGame,
    game_id: GameId,
) -> Entity {
    let root_entitiy = commands
        .spawn((
            transform,
            GameRoot {
                active_game,
                game_id: game_id,
            },
        ))
        .set_parent(container_entitiy)
        .id();

    root_entitiy
}

fn produce_tick_events(
    mut input_events: EventReader<InputEvent>,
    time: Res<Time<Fixed>>,
    mut q_root: Query<&mut GameRoot>,
    mut tick_event_writer: EventWriter<TickEvent>,
    local_game_root_res: Option<Res<LocalGameRoot>>,
) {
    let Some(local_game_root) = local_game_root_res else {
        return;
    };
    let Some(mut game_root) = q_root
        .iter_mut()
        .filter(|gr| gr.game_id == local_game_root.game_id)
        .next()
    else {
        return;
    };

    let game_id = local_game_root.game_id;
    let game = &mut game_root.active_game;

    let mut tick_events = vec![];
    use InputType::*;
    use TickMutation::*;

    tick_events.extend(
        input_events
            .read()
            .map(|e| match e.input_type {
                ShiftEvent(s) => vec![ShiftInput(s)],
                RotateEvent(r) => vec![RotateInput(r)],
                DownEvent => vec![DownInput(if e.is_repeat {
                    DownType::HoldRepeat
                } else {
                    DownType::FirstPress
                })],
                DropEvent => vec![DropInput],
                HoldEvent => vec![HoldInput],
                EnqueueGarbageEvent(lines) => vec![EnqueueGarbage(lines)],
                JumpToBotStartPositionEvent | PerformBotMoveEvent => vec![],
            })
            .flatten(),
    );

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
            .map(|mutation| TickEvent::new_local(TickMutationMessage { mutation, game_id })),
    );
}

fn update_root_tick(
    mut q_root: Query<&mut GameRoot>,
    mut tick_event_reader: EventReader<TickEvent>,
    mut lock_event_writer: EventWriter<LockEvent>,
    time: Res<Time<Fixed>>,
) {
    let cur_time = time.elapsed();

    // Group the incoming mutations by game.
    let mut mutations_by_game: BTreeMap<GameId, Vec<TickMutation>> = BTreeMap::new();
    for tick_event in tick_event_reader.read() {
        let game_id = tick_event.mutation.game_id;
        mutations_by_game
            .entry(game_id)
            .or_default()
            .push(tick_event.mutation.mutation.clone());
    }

    for mut game_root in q_root.iter_mut() {
        let game_id = game_root.game_id;
        let active_game = &mut game_root.active_game;
        let Some(mutations) = mutations_by_game.get(&game_id) else {
            continue;
        };

        // TODO: get game by game_id
        for tick_result in active_game.game.tick_mutation(mutations.clone()) {
            use manytris_core::consts;
            use TickResult::*;
            match tick_result {
                Lock(lr) => {
                    lock_event_writer.send(LockEvent {
                        game_id,
                        lock_result: lr.clone(),
                    });
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
}

impl ActiveGame {
    fn new(start_time: Duration, initial_shapes: Vec<Shape>) -> Self {
        Self::from_snapshot(GameState::new(initial_shapes), start_time)
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
            LockResult::GameOver => println!("Game Over!!!"),
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

fn save_timer_state_on_pause(
    time: Res<Time<Fixed>>,
    q_root: Query<&GameRoot>,
    local_game_root_res: Option<Res<LocalGameRoot>>,
    mut pause_timer_state: ResMut<PauseTimerState>,
) {
    let Some(local_game_root) = local_game_root_res else {
        return;
    };

    let Some(game_root) = q_root
        .iter()
        .find(|gr| gr.game_id == local_game_root.game_id)
    else {
        return;
    };

    let cur_time = time.elapsed();
    let game = &game_root.active_game;

    // Store pause time and calculate remaining times
    pause_timer_state.pause_time = Some(cur_time);
    pause_timer_state.remaining_drop_time = Some(game.next_drop_time.saturating_sub(cur_time));
    pause_timer_state.remaining_lock_time = game
        .lock_timer_target
        .map(|target| target.saturating_sub(cur_time));
}

fn restore_timer_state_on_unpause(
    time: Res<Time<Fixed>>,
    mut q_root: Query<&mut GameRoot>,
    local_game_root_res: Option<Res<LocalGameRoot>>,
    pause_timer_state: Res<PauseTimerState>,
) {
    let Some(local_game_root) = local_game_root_res else {
        return;
    };

    let Some(mut game_root) = q_root
        .iter_mut()
        .find(|gr| gr.game_id == local_game_root.game_id)
    else {
        return;
    };

    let cur_time = time.elapsed();
    let game = &mut game_root.active_game;

    // Restore timers by adding remaining time to current time
    if let Some(remaining) = pause_timer_state.remaining_drop_time {
        game.next_drop_time = cur_time + remaining;
    }

    if let Some(remaining) = pause_timer_state.remaining_lock_time {
        game.lock_timer_target = Some(cur_time + remaining);
    } else if pause_timer_state.pause_time.is_some() {
        // If there was no lock timer when we paused, keep it None
        game.lock_timer_target = None;
    }
}
