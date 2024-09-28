use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game_state::GameState;
use crate::plugins::assets::RenderAssets;
use crate::plugins::root;
use crate::plugins::root::{GameId, LocalGameRoot};
use crate::plugins::system_sets::UpdateSystems;

#[derive(Deserialize, Serialize, Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConnectionId(Uuid);

#[derive(Clone, Deserialize, Serialize, Debug, Event)]
pub enum ClientControlEvent {
    JoinRequest,
}

#[derive(Clone, Deserialize, Serialize, Debug, Event)]
pub enum ServerControlEvent {
    SnapshotResponse(GameState, GameId),
}

#[derive(Event)]
pub struct SendControlEventToClient {
    pub event: ServerControlEvent,
    pub to_connection: ConnectionId,
}

#[derive(Event)]
pub struct ReceiveControlEventFromClient {
    pub event: ClientControlEvent,
    pub from_connection: ConnectionId,
}

pub fn server_plugin(app: &mut App) {
    app.add_systems(
        Update,
        update_server_for_control_events.in_set(UpdateSystems::LocalEventProducers),
    )
    .add_event::<SendControlEventToClient>()
    .add_event::<ReceiveControlEventFromClient>();
}

pub fn client_plugin(app: &mut App) {
    app.add_systems(
        Update,
        update_client_for_control_events.in_set(UpdateSystems::LocalEventProducers),
    )
    .add_event::<ClientControlEvent>()
    .add_event::<ServerControlEvent>();
}

pub fn update_server_for_control_events(
    mut commands: Commands,
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
                let (game_state, game_id, _) =
                    root::create_new_root(&mut commands, &ra, &asset_server, time.elapsed());

                control_event_writer.send(SendControlEventToClient {
                    event: ServerControlEvent::SnapshotResponse(game_state, game_id),
                    to_connection: from_connection.clone(),
                });
            }
        }
    }
}

pub fn update_client_for_control_events(
    mut commands: Commands,
    ra: Res<RenderAssets>,
    asset_server: Res<AssetServer>,
    mut control_event_reader: EventReader<ServerControlEvent>,
    time: Res<Time<Fixed>>,
) {
    for sce in control_event_reader.read() {
        match sce {
            ServerControlEvent::SnapshotResponse(gs, game_id) => {
                let root_entity = root::create_root_from_snapshot(
                    &mut commands,
                    &ra,
                    &asset_server,
                    gs.clone(),
                    time.elapsed(),
                    game_id.clone(),
                );
                commands.insert_resource(LocalGameRoot {
                    game_id: game_id.clone(),
                    root_entity,
                });
            }
        }
    }
}

impl ConnectionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
