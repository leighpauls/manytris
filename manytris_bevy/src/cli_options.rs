use crate::states::{ExecType, MultiplayerType, PlayingState, StatesPlugin};
use bevy::prelude::*;
use clap::{ArgAction, Args, Parser, Subcommand};
use serde::Serialize;

// TODO: replace with "https://manytris-manager-265251374100.us-west1.run.app"
const DEFAULT_MANAGER_SERVER: &'static str = "http://localhost:3000";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct GameArgs {
    #[command(subcommand)]
    pub exec_command: ExecCommand,
}

#[derive(Subcommand, Clone, Debug)]
pub enum ExecCommand {
    Server(ServerConfig),
    Client(ClientConfig),
    Bot(BotConfig),
}

#[derive(Args, Clone, Debug, Serialize)]
pub struct ClientConfig {
    #[clap(flatten)]
    pub manager_server: ManagerServerConfig,
}

#[derive(Args, Clone, Debug, Serialize, Resource)]
pub struct ManagerServerConfig {
    #[arg(long, short = 'm', default_value = DEFAULT_MANAGER_SERVER)]
    pub manager_server: String,
}

#[derive(Args, Clone, Debug, Serialize)]
pub struct HostConfig {
    #[arg(long, default_value = "localhost")]
    pub host: String,
    #[arg(long, default_value = "9989")]
    pub port: u16,
}

#[derive(Args, Clone, Debug, Serialize)]
pub struct ServerConfig {
    #[clap(flatten)]
    pub server: HostConfig,
    #[clap(long, action=ArgAction::SetTrue)]
    pub headless: bool,
}

#[derive(Args, Clone, Debug, Serialize)]
pub struct BotConfig {
    #[clap(flatten)]
    pub server: HostConfig,

    #[arg(long, default_value = "1000")]
    pub bot_millis: u64,
}

pub fn web_client_args() -> GameArgs {
    GameArgs {
        exec_command: ExecCommand::Client(ClientConfig {
            manager_server: ManagerServerConfig {
                manager_server: DEFAULT_MANAGER_SERVER.into(),
            },
        }),
    }
}

impl ExecCommand {
    pub fn configure_states_plugin(&self) -> StatesPlugin {
        use ExecCommand::*;
        let initial_play_state = match self {
            Server(_) | Bot(_) => PlayingState::Playing,
            Client(_) => PlayingState::MainMenu,
        };

        let initial_exec_type = match self {
            Server(_) => ExecType::Server,
            Bot(_) => ExecType::MultiplayerClient(MultiplayerType::Bot),
            Client(_) => ExecType::StandAlone,
        };

        let headless = matches!(self, Server(ServerConfig { headless: true, .. }));

        StatesPlugin {
            initial_play_state,
            initial_exec_type,
            headless,
        }
    }
}
