use crate::plugins::assets::RenderAssets;
use crate::plugins::net_game_control_manager::{
    ClientControlEvent, ReceiveControlEventFromClient, SendControlEventToClient, ServerControlEvent,
};
use crate::plugins::root;
use crate::plugins::root::{GameId, GameRoot};
use bevy::prelude::*;
use std::collections::BTreeMap;

#[derive(Component)]
pub struct GameContainer {}

#[derive(Bundle)]
pub struct GameContainerBundle {
    transform: SpatialBundle,
    game_container: GameContainer,
}

#[derive(Resource)]
pub struct LocalGameRoot {
    pub game_id: GameId,
    pub root_entity: Entity,
}

pub fn stand_alone_plugin(app: &mut App) {
    app.add_systems(Startup, setup_stand_alone);
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
    ra: Res<RenderAssets>,
    asset_server: Res<AssetServer>,
    time: Res<Time<Fixed>>,
) {
    let container_entity = spawn_container(&mut commands, 1.0);
    let start_time = time.elapsed();
    let (_, game_id, root_entity) = root::create_new_root(
        &mut commands,
        container_entity,
        &ra,
        &asset_server,
        start_time,
    );
    set_local_game_root(&mut commands, game_id, root_entity);
}

fn setup_multiplayer_client(mut commands: Commands) {
    spawn_container(&mut commands, 1.0);
}

fn accept_server_control_events(
    mut commands: Commands,
    q_container: Query<Entity, With<GameContainer>>,
    mut events: EventReader<ServerControlEvent>,
    ra: Res<RenderAssets>,
    asset_server: Res<AssetServer>,
    time: Res<Time<Fixed>>,
) {
    for event in events.read() {
        match event {
            ServerControlEvent::SnapshotResponse(gs, game_id) => {
                let container_entity = q_container.single();
                let root_entity = root::create_root_from_snapshot(
                    &mut commands,
                    container_entity,
                    &ra,
                    &asset_server,
                    gs.clone(),
                    time.elapsed(),
                    game_id.clone(),
                );
                set_local_game_root(&mut commands, game_id.clone(), root_entity);
            }
        }
    }
}

fn setup_server(mut commands: Commands) {
    spawn_container(&mut commands, 0.5);
}

fn accept_client_control_events(
    mut commands: Commands,
    q_container: Query<Entity, With<GameContainer>>,
    ra: Res<RenderAssets>,
    asset_server: Res<AssetServer>,
    mut control_event_reader: EventReader<ReceiveControlEventFromClient>,
    mut control_event_writer: EventWriter<SendControlEventToClient>,
    time: Res<Time<Fixed>>,
) {
    for rce in control_event_reader.read() {
        match rce {
            ReceiveControlEventFromClient {
                event: ClientControlEvent::JoinRequest,
                from_connection,
            } => {
                let container_entity = q_container.single();
                let (game_state, game_id, _) = root::create_new_root(
                    &mut commands,
                    container_entity,
                    &ra,
                    &asset_server,
                    time.elapsed(),
                );

                control_event_writer.send(SendControlEventToClient {
                    event: ServerControlEvent::SnapshotResponse(game_state, game_id),
                    to_connection: from_connection.clone(),
                });
            }
        }
    }
}

fn spawn_container(commands: &mut Commands, scaling_factor: f32) -> Entity {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(GameContainerBundle {
            transform: SpatialBundle::from_transform(Transform::from_scale(Vec3::splat(
                scaling_factor,
            ))),
            game_container: GameContainer {},
        })
        .id()
}

fn set_local_game_root(commands: &mut Commands, game_id: GameId, root_entity: Entity) {
    commands.insert_resource(LocalGameRoot {
        game_id,
        root_entity,
    });
}
