use bevy::prelude::*;

pub struct StatesPlugin {
    pub initial_play_state: PlayingState,
    pub initial_exec_type: ExecType,
}

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(self.initial_play_state);
        app.insert_resource(self.initial_exec_type);
    }
}

#[derive(States, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum PlayingState {
    MainMenu,
    Playing,
}

#[derive(Resource, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExecType {
    StandAlone,
    Server,
    MultiplayerClient(MultiplayerType),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MultiplayerType {
    Human,
    Bot,
}

pub fn is_stand_alone(et: Res<ExecType>) -> bool {
    *et == ExecType::StandAlone
}

pub fn is_multiplayer_client(et: Res<ExecType>) -> bool {
    matches!(*et, ExecType::MultiplayerClient(_))
}

pub fn is_server(et: Res<ExecType>) -> bool {
    *et == ExecType::Server
}

pub fn is_client(et: Res<ExecType>) -> bool {
    matches!(*et, ExecType::StandAlone | ExecType::MultiplayerClient(_))
}

pub fn is_human(et: Res<ExecType>) -> bool {
    matches!(
        *et,
        ExecType::StandAlone | ExecType::MultiplayerClient(MultiplayerType::Human)
    )
}

pub fn is_bot(et: Res<ExecType>) -> bool {
    matches!(*et, ExecType::MultiplayerClient(MultiplayerType::Bot))
}

pub fn produces_shapes(et: Res<ExecType>) -> bool {
    matches!(*et, ExecType::StandAlone | ExecType::Server)
}
