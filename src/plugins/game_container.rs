use crate::game_state::{GameState, LockResult};
use crate::plugins::assets::{RenderAssets, BLOCK_SIZE};
use crate::plugins::input::{InputEvent, InputType};
use crate::plugins::net_game_control_manager::{
    ClientControlEvent, ConnectionId, ConnectionTarget, ReceiveControlEventFromClient,
    SendControlEventToClient, ServerControlEvent,
};
use crate::plugins::root::{GameId, GameRoot, LockEvent};
use crate::plugins::shape_producer::ShapeProducer;
use crate::plugins::{root, shape_producer};
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
    tiled_games: Vec<GameId>,
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

pub fn common_plugin(app: &mut App) {
    app.add_systems(Update, respond_to_resize);
}

pub fn stand_alone_plugin(app: &mut App) {
    app.add_systems(Startup, setup_stand_alone.after(shape_producer::setup));
}

pub fn multiplayer_client_plugin(app: &mut App) {
    app.add_systems(Startup, setup_multiplayer_client)
        .add_systems(Update, accept_server_control_events);
}

pub fn server_plugin(app: &mut App) {
    app.add_systems(Startup, setup_server)
        .add_systems(Update, accept_client_control_events)
        .add_systems(Update, deliver_garbage);
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
) {
    let mut local_game_id = local_game_root_res.map(|lgr| lgr.game_id);

    for event in events.read() {
        match event {
            ServerControlEvent::AssignGameId(game_id) => {
                set_local_game_root(&mut commands, game_id.clone());
                local_game_id = Some(game_id.clone());
                println!("Assigned gameid {game_id:?}");
            }
            ServerControlEvent::SnapshotResponse(gs, game_id) => {
                let (container_entity, mut game_container) = q_container.single_mut();
                println!("Received snapshot for gameid {game_id:?}");
                // TODO: better define multiplayer tiling
                let transform = if Some(game_id) == local_game_id.as_ref() {
                    active_game_transform()
                } else {
                    client_opponent_game_transform(game_container.tiled_games.len())
                };

                game_container.tiled_games.push(game_id.clone());

                println!("New transform: {transform:?}");

                root::create_root_from_snapshot(
                    &mut commands,
                    container_entity,
                    transform,
                    &ra,
                    &asset_server,
                    gs.clone(),
                    time.elapsed(),
                    game_id.clone(),
                );
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

fn deliver_garbage(
    mut lock_events: EventReader<LockEvent>,
    mut control_event_writer: EventWriter<SendControlEventToClient>,
    q_game_container: Query<&GameContainer>,
) {
    let game_container = q_game_container.single();
    for lock_event in lock_events.read() {
        if let LockEvent {
            game_id,
            lock_result: LockResult::Ok { lines_cleared },
        } = lock_event
        {
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
    commands.spawn(Camera2dBundle::default());
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
        let (game_state, game_id, _) = root::create_new_root(
            commands,
            container_entity,
            tiled_game_transform(new_idx),
            ra,
            asset_server,
            cur_time,
            shape_producer,
        );
        self.tiled_games.push(game_id);
        self.connection_map.insert(game_id, connection_id);
        (game_state, game_id)
    }

    pub fn connection_for_game(&self, game_id: &GameId) -> ConnectionId {
        *self.connection_map.get(game_id).unwrap()
    }
}
