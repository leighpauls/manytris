use bevy::prelude::*;

pub struct StatesPlugin {
    pub initial_play_state: PlayingState,
    pub initial_exec_type: ExecType,
    pub headless: bool
}

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(self.initial_play_state);
        app.insert_resource(self.initial_exec_type);
        app.init_resource::<PauseState>();
        if self.headless {
            app.insert_resource(Headless);
        }
    }
}

#[derive(States, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum PlayingState {
    MainMenu,
    Connecting,
    Playing,
    Restarting,
}

#[derive(Resource, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExecType {
    StandAlone,
    Server,
    MultiplayerClient(MultiplayerType),
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseState {
    #[default]
    Unpaused,
    Paused,
}

#[derive(Resource, Debug, Copy, Clone)]
pub struct Headless;

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

pub fn headed(headless: Option<Res<Headless>>) -> bool {
    headless.is_none()
}

pub fn headless(headless: Option<Res<Headless>>) -> bool {
    headless.is_some()
}

pub fn is_paused(pause_state: Option<Res<PauseState>>) -> bool {
    pause_state.map(|ps| *ps == PauseState::Paused).unwrap_or(false)
}

pub fn is_unpaused(pause_state: Option<Res<PauseState>>) -> bool {
    !is_paused(pause_state)
}
