use crate::plugins::root::{ControlEvent, TickMutationMessage};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum NetMessage {
    Tick(TickMutationMessage),
    Control(ControlEvent),
}
