use crate::net_game_control_manager::{ClientControlEvent, ServerControlEvent};
use crate::root::TickMutationMessage;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum NetMessage {
    Tick(TickMutationMessage),
    ServerControl(ServerControlEvent),
    ClientControl(ClientControlEvent),
}
