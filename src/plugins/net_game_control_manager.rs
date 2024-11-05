use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game_state::GameState;
use crate::plugins::root::GameId;

#[derive(Deserialize, Serialize, Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConnectionId(Uuid);

#[derive(Clone, Deserialize, Serialize, Debug, Event)]
pub enum ClientControlEvent {
    JoinRequest,
}

#[derive(Clone, Deserialize, Serialize, Debug, Event)]
pub enum ServerControlEvent {
    AssignGameId(GameId),
    SnapshotResponse(GameState, GameId),
    DeliverGarbage{
        from_game_id: GameId,
        num_lines: usize,
    }
}

pub enum ConnectionTarget {
    All,
    To(ConnectionId),
}

#[derive(Event)]
pub struct SendControlEventToClient {
    pub event: ServerControlEvent,
    pub to_connection: ConnectionTarget,
}

#[derive(Event)]
pub struct ReceiveControlEventFromClient {
    pub event: ClientControlEvent,
    pub from_connection: ConnectionId,
}

pub fn server_plugin(app: &mut App) {
    app.add_event::<SendControlEventToClient>()
        .add_event::<ReceiveControlEventFromClient>();
}

pub fn client_plugin(app: &mut App) {
    app.add_event::<ClientControlEvent>()
        .add_event::<ServerControlEvent>();
}

impl ConnectionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
