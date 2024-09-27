use serde::{Deserialize, Serialize};

use crate::plugins::net_game_control_manager::{ClientControlEvent, ServerControlEvent};
use crate::plugins::root::TickMutationMessage;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum NetMessage {
    Tick(TickMutationMessage),
    ServerControl(ServerControlEvent),
    ClientControl(ClientControlEvent),
}
