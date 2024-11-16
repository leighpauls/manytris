use crate::game_state::{GameState, LockResult};
use crate::plugins::assets::{RenderAssets, BLOCK_SIZE};
use crate::plugins::input::{InputEvent, InputType};
use crate::plugins::net_game_control_manager::{
    ClientControlEvent, ConnectionId, ConnectionTarget, ReceiveControlEventFromClient,
    SendControlEventToClient, ServerControlEvent,
};
use crate::plugins::root::{GameId, GameRoot, LockEvent};
use crate::plugins::shape_producer::ShapeProducer;
use crate::plugins::states::{ExecType, MultiplayerType, PlayingState};
use crate::plugins::{root, shape_producer, states};
use bevy::prelude::*;
use bevy::window::WindowResized;
use std::collections::BTreeMap;
use std::time::Duration;

const HEIGHT_IN_BLOCKS: f32 = 26.;
const PADDING_BLOCKS: f32 = 2.;
const WIDTH_IN_BLOCKS: f32 = 22.;

const HORIZONTAL_TILES: isize = 4;
const VERTICAL_TILES: isize = 3;

#[derive(Component)]
pub struct GameContainer {
    tiled_games: Vec<(GameId, Entity)>,
    connection_map: BTreeMap<GameId, ConnectionId>,
    container_type: ContainerType,
}

#[derive(Bundle)]
pub struct GameContainerBundle {
    transform: SpatialBundle,
    game_container: GameContainer,
}

#[derive(Resource)]
pub struct LocalGameRoot {
    pub game_id: GameId,
}

enum ContainerType {
    StandAlone,
    MultiplayerClient,
    ServerTiles,
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(PlayingState::Playing),
        (
            setup_stand_alone
                .after(shape_producer::setup)
                .run_if(states::is_stand_alone),
            setup_multiplayer_client.run_if(states::is_multiplayer_client),
            setup_server.run_if(states::is_server),
        ),
    )
    .add_systems(OnExit(PlayingState::Playing), tear_down_container)
    .add_systems(
        Update,
        (
            respond_to_resize.run_if(in_state(PlayingState::Playing)),
            accept_server_control_events
                .run_if(in_state(PlayingState::Playing))
                .run_if(states::is_multiplayer_client),
            (accept_client_control_events, accept_server_lock_events)
                .run_if(in_state(PlayingState::Playing))
                .run_if(states::is_server),
            accept_standalone_loss
                .run_if(in_state(PlayingState::Playing))
                .run_if(states::is_stand_alone),
        ),
    );
}

fn accept_standalone_loss(
    mut lock_events: EventReader<LockEvent>,
    mut play_state: ResMut<NextState<PlayingState>>,
) {
    let game_over_event = lock_events
        .read()
        .filter(|le| matches!(le.lock_result, LockResult::GameOver))
        .next();

    if game_over_event.is_some() {
        play_state.set(PlayingState::MainMenu);
    }
}

fn setup_stand_alone(
    mut commands: Commands,
    q_window: Query<&Window>,
    ra: Res<RenderAssets>,
    asset_server: Res<AssetServer>,
    time: Res<Time<Fixed>>,
    mut shape_producer: Query<&mut ShapeProducer>,
) {
    let container_entity =
        spawn_container(&mut commands, ContainerType::StandAlone, q_window.single());
    let start_time = time.elapsed();
    let (_, game_id, _) = root::create_new_root(
        &mut commands,
        container_entity,
        active_game_transform(),
        &ra,
        &asset_server,
        start_time,
        shape_producer.single_mut().as_mut(),
    );
    set_local_game_root(&mut commands, game_id);
}

fn tear_down_container(mut commands: Commands, container_q: Query<Entity, With<GameContainer>>) {
    commands.entity(container_q.single()).despawn_recursive();
}

fn setup_multiplayer_client(mut commands: Commands, q_window: Query<&Window>) {
    spawn_container(
        &mut commands,
        ContainerType::MultiplayerClient,
        q_window.single(),
    );
}

fn accept_server_control_events(
    mut commands: Commands,
    mut q_container: Query<(Entity, &mut GameContainer)>,
    mut events: EventReader<ServerControlEvent>,
    mut input_writer: EventWriter<InputEvent>,
    ra: Res<RenderAssets>,
    asset_server: Res<AssetServer>,
    time: Res<Time<Fixed>>,
    local_game_root_res: Option<Res<LocalGameRoot>>,
    mut play_state: ResMut<NextState<PlayingState>>,
    exec_type: Res<ExecType>,
    mut root_xforms_q: Query<&mut Transform>,
) {
    let mut local_game_id = local_game_root_res.map(|lgr| lgr.game_id);
    let (container_entity, mut game_container) = q_container.single_mut();

    for event in events.read() {
        match event {
            ServerControlEvent::AssignGameId(game_id) => {
                set_local_game_root(&mut commands, game_id.clone());
                local_game_id = Some(game_id.clone());
                println!("Assigned gameid {game_id:?}");
            }
            ServerControlEvent::SnapshotResponse(gs, game_id) => {
                println!("Received snapshot for gameid {game_id:?}");
                // TODO: better define multiplayer tiling
                let transform = if Some(game_id) == local_game_id.as_ref() {
                    active_game_transform()
                } else {
                    client_opponent_game_transform(game_container.tiled_games.len())
                };

                println!("New transform: {transform:?}");

                let entity = root::create_root_from_snapshot(
                    &mut commands,
                    container_entity,
                    transform,
                    &ra,
                    &asset_server,
                    gs.clone(),
                    time.elapsed(),
                    game_id.clone(),
                );

                if Some(game_id) != local_game_id.as_ref() {
                    game_container.tiled_games.push((*game_id, entity));
                }
            }
            ServerControlEvent::DeliverGarbage {
                from_game_id,
                num_lines,
            } => {
                if local_game_id.is_some() && Some(from_game_id) != local_game_id.as_ref() {
                    input_writer.send(InputEvent {
                        input_type: InputType::EnqueueGarbageEvent(*num_lines),
                        is_repeat: false,
                    });
                }
            }
            ServerControlEvent::ClientGameOver(game_id) => {
                if local_game_id == Some(*game_id) {
                    println!("Game over!");
                    match *exec_type {
                        ExecType::MultiplayerClient(MultiplayerType::Bot) => {
                            panic!("TODO: exit safely");
                        }
                        ExecType::MultiplayerClient(MultiplayerType::Human) => {
                            play_state.set(PlayingState::MainMenu);
                        }
                        _ => {
                            panic!("Unexpected");
                        }
                    }
                } else {
                    game_container.remove_game(
                        &mut commands,
                        *game_id,
                        &mut root_xforms_q,
                        client_opponent_game_transform,
                    );
                }
            }
        }
    }
}

fn setup_server(mut commands: Commands, q_window: Query<&Window>) {
    spawn_container(&mut commands, ContainerType::ServerTiles, q_window.single());
}

fn accept_client_control_events(
    mut commands: Commands,
    mut q_container: Query<(Entity, &mut GameContainer)>,
    ra: Res<RenderAssets>,
    asset_server: Res<AssetServer>,
    mut control_event_reader: EventReader<ReceiveControlEventFromClient>,
    mut control_event_writer: EventWriter<SendControlEventToClient>,
    time: Res<Time<Fixed>>,
    mut q_shape_producer: Query<&mut ShapeProducer>,
    q_roots: Query<&GameRoot>,
) {
    for rce in control_event_reader.read() {
        match rce {
            ReceiveControlEventFromClient {
                event: ClientControlEvent::JoinRequest,
                from_connection,
            } => {
                let (container_entity, mut container) = q_container.single_mut();
                let (game_state, game_id) = container.create_server_game(
                    &mut commands,
                    container_entity,
                    &ra,
                    &asset_server,
                    time.elapsed(),
                    q_shape_producer.single_mut().as_mut(),
                    *from_connection,
                );

                control_event_writer.send(SendControlEventToClient {
                    event: ServerControlEvent::AssignGameId(game_id),
                    to_connection: ConnectionTarget::To(from_connection.clone()),
                });

                // Send existing game snapshots to the current connection.
                control_event_writer.send_batch(q_roots.iter().map(|gr| {
                    SendControlEventToClient {
                        event: ServerControlEvent::SnapshotResponse(
                            gr.active_game.game.clone(),
                            gr.game_id,
                        ),
                        to_connection: ConnectionTarget::To(*from_connection),
                    }
                }));

                // Inform all clients about the new game snapshot.
                control_event_writer.send(SendControlEventToClient {
                    event: ServerControlEvent::SnapshotResponse(game_state, game_id),
                    to_connection: ConnectionTarget::AllExcept(None),
                });
            }
        }
    }
}

fn accept_server_lock_events(
    mut commands: Commands,
    mut lock_events: EventReader<LockEvent>,
    mut control_event_writer: EventWriter<SendControlEventToClient>,
    mut q_game_container: Query<&mut GameContainer>,
    mut root_xform_q: Query<&mut Transform>,
) {
    let mut game_container = q_game_container.single_mut();
    for LockEvent {
        game_id,
        lock_result,
    } in lock_events.read()
    {
        match lock_result {
            LockResult::Ok { lines_cleared } => {
                if *lines_cleared <= 1 {
                    continue;
                }
                let num_lines: usize = match *lines_cleared {
                    n if n <= 1 => continue,
                    2 => 1,
                    3 => 2,
                    n => n as usize,
                };
                control_event_writer.send(SendControlEventToClient {
                    event: ServerControlEvent::DeliverGarbage {
                        from_game_id: *game_id,
                        num_lines,
                    },
                    to_connection: ConnectionTarget::AllExcept(Some(
                        game_container.connection_for_game(game_id),
                    )),
                });
            }
            LockResult::GameOver => {
                control_event_writer.send(SendControlEventToClient {
                    event: ServerControlEvent::ClientGameOver(*game_id),
                    to_connection: ConnectionTarget::AllExcept(None),
                });

                // Remove the game locally
                game_container.remove_game(
                    &mut commands,
                    *game_id,
                    &mut root_xform_q,
                    tiled_game_transform,
                );
            }
        }
    }
}

fn respond_to_resize(
    mut q_container_xform: Query<(&mut Transform, &GameContainer)>,
    mut resize_reader: EventReader<WindowResized>,
) {
    for e in resize_reader.read() {
        let (mut xform, container) = q_container_xform.single_mut();
        *xform = container.get_transform(e.width, e.height);
    }
}

fn active_game_transform() -> Transform {
    Transform::from_translation(
        (Vec3::Y * PADDING_BLOCKS - 0.5 * Vec3::new(WIDTH_IN_BLOCKS, HEIGHT_IN_BLOCKS, 0.0))
            * BLOCK_SIZE,
    )
}

fn tiled_game_transform(game_index: usize) -> Transform {
    let game_index = game_index as isize;
    let tile_x = (game_index % HORIZONTAL_TILES) as f32;
    let tile_y = (game_index / HORIZONTAL_TILES) as f32;

    Transform::from_translation(
        ((Vec3::new(tile_x, tile_y, 0.)
            - 0.5 * Vec3::new(HORIZONTAL_TILES as f32, VERTICAL_TILES as f32, 0.))
            * Vec3::new(WIDTH_IN_BLOCKS, HEIGHT_IN_BLOCKS, 0.)
            + Vec3::Y * PADDING_BLOCKS)
            * BLOCK_SIZE,
    )
}

fn client_opponent_game_transform(opponent_index: usize) -> Transform {
    let scale = 0.3;
    let active = active_game_transform();
    active.with_scale(Vec3::splat(scale)).with_translation(
        active.translation
            + Vec3::X * WIDTH_IN_BLOCKS * BLOCK_SIZE
            + Vec3::Y * HEIGHT_IN_BLOCKS * BLOCK_SIZE * scale * (opponent_index as f32),
    )
}

fn spawn_container(
    commands: &mut Commands,
    container_type: ContainerType,
    window: &Window,
) -> Entity {
    let game_container = GameContainer {
        tiled_games: default(),
        connection_map: default(),
        container_type,
    };
    let transform =
        game_container.get_transform(window.resolution.width(), window.resolution.height());
    commands
        .spawn(GameContainerBundle {
            transform: SpatialBundle::from_transform(transform),
            game_container,
        })
        .id()
}

fn set_local_game_root(commands: &mut Commands, game_id: GameId) {
    commands.insert_resource(LocalGameRoot { game_id });
}

impl GameContainer {
    fn get_transform(&self, width_pixels: f32, height_pixels: f32) -> Transform {
        let x_scale = width_pixels / (WIDTH_IN_BLOCKS * BLOCK_SIZE);
        let y_scale = height_pixels / (HEIGHT_IN_BLOCKS * BLOCK_SIZE);

        let scale = match self.container_type {
            ContainerType::StandAlone | ContainerType::MultiplayerClient => x_scale.min(y_scale),
            ContainerType::ServerTiles => {
                (x_scale / HORIZONTAL_TILES as f32).min(y_scale / VERTICAL_TILES as f32)
            }
        };

        Transform::from_scale(Vec3::splat(scale))
    }

    fn create_server_game(
        &mut self,
        commands: &mut Commands,
        container_entity: Entity,
        ra: &Res<RenderAssets>,
        asset_server: &Res<AssetServer>,
        cur_time: Duration,
        shape_producer: &mut ShapeProducer,
        connection_id: ConnectionId,
    ) -> (GameState, GameId) {
        let new_idx = self.tiled_games.len();
        let (game_state, game_id, root_entity) = root::create_new_root(
            commands,
            container_entity,
            tiled_game_transform(new_idx),
            ra,
            asset_server,
            cur_time,
            shape_producer,
        );
        self.tiled_games.push((game_id, root_entity));
        self.connection_map.insert(game_id, connection_id);
        (game_state, game_id)
    }

    pub fn connection_for_game(&self, game_id: &GameId) -> ConnectionId {
        *self.connection_map.get(game_id).unwrap()
    }

    pub fn remove_game(
        &mut self,
        commands: &mut Commands,
        game_id: GameId,
        root_xform_q: &mut Query<&mut Transform>,
        xform_function: fn(usize) -> Transform,
    ) {
        // find the entity
        let idx = self
            .tiled_games
            .iter()
            .position(|(gid, _)| gid == &game_id)
            .unwrap();

        // Remove and despawn
        let (_, entity) = self.tiled_games.remove(idx);
        commands.entity(entity).despawn_recursive();

        // re-tile the remaining games
        for (i, (_, entity)) in self.tiled_games.iter().enumerate().skip(idx) {
            let mut xform = root_xform_q.get_mut(*entity).unwrap();
            *xform = xform_function(i);
        }
    }
}
