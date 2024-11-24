use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use manytris_core::game_state::GameState;
use crate::plugins::root::GameId;

#[derive(Deserialize, Serialize, Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConnectionId(Uuid);

#[derive(Clone, Deserialize, Serialize, Debug, Event)]
pub enum ClientControlEvent {
    JoinRequest,
    ReconnectRequest(GameId),
}

#[derive(Clone, Deserialize, Serialize, Debug, Event)]
pub enum ServerControlEvent {
    AssignGameId(GameId),
    SnapshotResponse(GameState, GameId),
    DeliverGarbage {
        from_game_id: GameId,
        num_lines: usize,
    },
    ClientGameOver(GameId),
    RejectConnectionRequest,
}

#[derive(Copy, Clone)]
pub enum ConnectionTarget {
    AllExcept(Option<ConnectionId>),
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

impl ConnectionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
