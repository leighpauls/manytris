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
    if let ExecType::MultiplayerClient(_) = et.as_ref() {
        true
    } else {
        false
    }
}

pub fn is_server(et: Res<ExecType>) -> bool {
    *et == ExecType::Server
}

pub fn is_client(et: Res<ExecType>) -> bool {
    match et.as_ref() {
        ExecType::StandAlone | ExecType::MultiplayerClient(_) => true,
        ExecType::Server => false,
    }
}

pub fn is_human(et: Res<ExecType>) -> bool {
    match et.as_ref() {
        ExecType::StandAlone => true,
        ExecType::Server => false,
        ExecType::MultiplayerClient(ccfg) => match ccfg {
            MultiplayerType::Human => true,
            MultiplayerType::Bot => false,
        },
    }
}

pub fn is_bot(et: Res<ExecType>) -> bool {
    match et.as_ref() {
        ExecType::StandAlone | ExecType::Server => false,
        ExecType::MultiplayerClient(ccfg) => match ccfg {
            MultiplayerType::Human => false,
            MultiplayerType::Bot => true,
        },
    }
}
pub fn produces_shapes(et: Res<ExecType>) -> bool {
    match et.as_ref() {
        ExecType::StandAlone | ExecType::Server => true,
        ExecType::MultiplayerClient(_) => false,
    }
}
