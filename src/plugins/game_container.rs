use bevy::prelude::*;
use bevy::window::WindowResized;

use crate::plugins::assets::{RenderAssets, BLOCK_SIZE};
use crate::plugins::net_game_control_manager::{
    ClientControlEvent, ConnectionTarget, ReceiveControlEventFromClient, SendControlEventToClient,
    ServerControlEvent,
};
use crate::plugins::root::{GameId, GameRoot};
use crate::plugins::shape_producer::ShapeProducer;
use crate::plugins::{root, shape_producer};

const HEIGHT_IN_BLOCKS: f32 = 26.;
const PADDING_BLOCKS: f32 = 2.;
const WIDTH_IN_BLOCKS: f32 = 22.;

const HORIZONTAL_TILES: isize = 4;
const VERTICAL_TILES: isize = 3;

#[derive(Component)]
pub struct GameContainer {
    tiled_games: Vec<GameId>,
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
        .add_systems(Update, accept_client_control_events);
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
    let (_, game_id, root_entity) = root::create_new_root(
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
    q_container: Query<Entity, With<GameContainer>>,
    mut events: EventReader<ServerControlEvent>,
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
                let container_entity = q_container.single();
                println!("Received snapshot for gameid {game_id:?}");
                // TODO: better define multiplayer tiling
                let transform = if Some(game_id) == local_game_id.as_ref() {
                    active_game_transform()
                } else {
                    tiled_game_transform(1)
                };

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
                let new_idx = container.tiled_games.len();
                let (game_state, game_id, _) = root::create_new_root(
                    &mut commands,
                    container_entity,
                    tiled_game_transform(new_idx),
                    &ra,
                    &asset_server,
                    time.elapsed(),
                    q_shape_producer.single_mut().as_mut(),
                );
                container.tiled_games.push(game_id);

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
                        to_connection: ConnectionTarget::To(from_connection.clone()),
                    }
                }));

                // Inform all clients about the new game snapshot.
                control_event_writer.send(SendControlEventToClient {
                    event: ServerControlEvent::SnapshotResponse(game_state, game_id),
                    to_connection: ConnectionTarget::All,
                });
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

fn spawn_container(
    commands: &mut Commands,
    container_type: ContainerType,
    window: &Window,
) -> Entity {
    let game_container = GameContainer {
        tiled_games: default(),
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
}
