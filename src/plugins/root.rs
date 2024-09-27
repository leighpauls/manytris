use bevy::ecs::query::QuerySingleError;
use std::collections::BTreeMap;
use std::time::Duration;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::bot::bot_player;
use crate::bot::bot_shader::BotShaderContext;
use crate::bot::bot_start_positions::StartPositions;
use crate::consts;
use crate::game_state::{DownType, GameState, LockResult, TickMutation, TickResult};
use crate::plugins::assets::RenderAssets;
use crate::plugins::input::{InputEvent, InputType};
use crate::plugins::system_sets::UpdateSystems;
use crate::plugins::{assets, field_blocks, scoreboard, window_blocks};
use crate::shapes::Shape;

const LINES_PER_LEVEL: i32 = 10;

/// This plugin must be used for all executable variants.
pub fn common_plugin(app: &mut App) {
    app.add_systems(Update, update_root_tick.in_set(UpdateSystems::RootTick))
        .add_event::<TickEvent>()
        .add_event::<LockEvent>()
        .insert_resource(StartPositionRes(StartPositions::new()));
}

/// Use this plugin for the client of a multiplayer game.
pub fn client_plugin(app: &mut App) {
    app.add_systems(
        Update,
        produce_tick_events.in_set(UpdateSystems::LocalEventProducers),
    );
}

/// Use this plugin for clients of single-player games.
pub fn stand_alone_plugin(app: &mut App) {
    app.add_systems(Startup, setup_start_standalone_game)
        .add_systems(
            Update,
            produce_tick_events.in_set(UpdateSystems::LocalEventProducers),
        );
}

#[derive(Resource)]
pub struct StartPositionRes(pub StartPositions);

#[derive(Component)]
pub struct GameRoot {
    pub game_id: Uuid,
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

#[derive(Bundle)]
struct RootTransformBundle {
    transform: SpatialBundle,
    marker: GameRoot,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TickMutationMessage {
    pub mutation: TickMutation,
    pub game_id: Uuid,
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

#[derive(Event, Deserialize, Serialize)]
pub struct LockEvent {
    pub game_id: Uuid,
    pub lock_result: LockResult,
}

fn setup_start_standalone_game(
    mut commands: Commands,
    ra: Res<RenderAssets>,
    asset_server: Res<AssetServer>,
    time: Res<Time<Fixed>>,
) {
    let start_time = time.elapsed();
    create_new_root(&mut commands, &ra, &asset_server, start_time);
}

pub fn create_new_root(
    commands: &mut Commands,
    ra: &Res<RenderAssets>,
    asset_server: &Res<AssetServer>,
    cur_time: Duration,
) -> (GameState, Uuid) {
    let active_game = ActiveGame::new(cur_time);
    let game_state = active_game.game.clone();
    let game_id = Uuid::new_v4();
    spawn_root(commands, ra, asset_server, active_game, game_id);
    (game_state, game_id)
}

pub fn create_root_from_snapshot(
    commands: &mut Commands,
    ra: &Res<RenderAssets>,
    asset_server: &Res<AssetServer>,
    gs: GameState,
    cur_time: Duration,
    game_id: Uuid,
) {
    let active_game = ActiveGame::from_snapshot(gs, cur_time);
    spawn_root(commands, ra, asset_server, active_game, game_id);
}

fn spawn_root(
    commands: &mut Commands,
    ra: &Res<RenderAssets>,
    asset_server: &Res<AssetServer>,
    active_game: ActiveGame,
    game_id: Uuid,
) {
    let root_entitiy = commands
        .spawn(RootTransformBundle {
            transform: SpatialBundle::from_transform(Transform::from_xyz(
                -assets::BLOCK_SIZE * 8.,
                -assets::BLOCK_SIZE * 11.,
                0.,
            )),
            marker: GameRoot {
                active_game,
                game_id: game_id,
            },
        })
        .id();

    field_blocks::spawn_field(commands, ra, root_entitiy);
    scoreboard::spawn_scoreboard(commands, asset_server, root_entitiy);
    window_blocks::spawn_windows(commands, ra, root_entitiy);
}

fn produce_tick_events(
    mut input_events: EventReader<InputEvent>,
    time: Res<Time<Fixed>>,
    mut q_root: Query<&mut GameRoot>,
    mut tick_event_writer: EventWriter<TickEvent>,
    sp: Res<StartPositionRes>,
) {
    let Some(mut game_root) = GameRoot::for_single_mut(q_root.get_single_mut()) else {
        return;
    };
    let game_id = game_root.game_id;
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
                JumpToBotStartPositionEvent => {
                    vec![JumpToBotStartPosition(
                        sp.0.bot_start_position(game.game.active_shape(), 0).clone(),
                    )]
                }
                PerformBotMoveEvent => make_bot_move_events(game, &sp.0),
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

fn make_bot_move_events(game: &ActiveGame, sp: &StartPositions) -> Vec<TickMutation> {
    let bot_context = BotShaderContext::new().unwrap();
    let mr =
        bot_player::select_next_move(&game.game, &bot_context, &consts::BEST_BOT_KS, 3).unwrap();
    mr.moves[0].as_tick_mutations(sp)
}

fn update_root_tick(
    mut q_root: Query<&mut GameRoot>,
    mut tick_event_reader: EventReader<TickEvent>,
    mut lock_event_writer: EventWriter<LockEvent>,
    time: Res<Time<Fixed>>,
) {
    let cur_time = time.elapsed();

    let Some(mut game_root) = GameRoot::for_single_mut(q_root.get_single_mut()) else {
        return;
    };
    let active_game = &mut game_root.active_game;

    let mut mutations_by_game: BTreeMap<Uuid, Vec<TickMutation>> = BTreeMap::new();
    for tick_event in tick_event_reader.read() {
        let game_id = tick_event.mutation.game_id;
        mutations_by_game
            .entry(game_id)
            .or_default()
            .push(tick_event.mutation.mutation.clone());
    }

    for (game_id, mutations) in mutations_by_game {
        // TODO: get game by game_id
        for tick_result in active_game.game.tick_mutation(mutations) {
            use TickResult::*;
            match tick_result {
                Lock(lr) => {
                    println!("Lock result: {:?}", lr);
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

impl GameRoot {
    pub fn for_single(res: Result<&GameRoot, QuerySingleError>) -> Option<&GameRoot> {
        match res {
            Ok(r) => Some(r),
            Err(QuerySingleError::NoEntities(_)) => None,
            Err(QuerySingleError::MultipleEntities(_)) => panic!("Unexpected multiple roots found"),
        }
    }
    pub fn for_single_mut(res: Result<Mut<GameRoot>, QuerySingleError>) -> Option<Mut<GameRoot>> {
        match res {
            Ok(r) => Some(r),
            Err(QuerySingleError::NoEntities(_)) => None,
            Err(QuerySingleError::MultipleEntities(_)) => panic!("Unexpected multiple roots found"),
        }
    }
}

impl ActiveGame {
    fn new(start_time: Duration) -> Self {
        // TODO: make this random
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
