use crate::game_state::TickMutation;
use crate::plugins::root::ControlEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum NetMessage {
    Tick(TickMutation),
    Control(ControlEvent),
}
