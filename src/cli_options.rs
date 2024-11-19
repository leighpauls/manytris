use crate::plugins::states::{ExecType, MultiplayerType, PlayingState, StatesPlugin};
use clap::{ArgAction, Args, Parser, Subcommand};
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct GameArgs {
    #[command(subcommand)]
    pub exec_command: ExecCommand,
}

#[derive(Subcommand, Clone, Debug)]
pub enum ExecCommand {
    Server(ServerConfig),
    Client(HostConfig),
    Bot(BotConfig),
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
        exec_command: ExecCommand::Client(HostConfig {
            host: String::from("localhost"),
            port: 9989,
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
