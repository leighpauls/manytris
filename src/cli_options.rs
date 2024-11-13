use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct GameArgs {
    #[command(subcommand)]
    pub exec_command: ExecCommand,
}

#[derive(Subcommand, Clone, Debug)]
pub enum ExecCommand {
    Server(HostConfig),
    Client(ClientConfig),
    StandAlone,
}

#[derive(Args, Clone, Debug, Serialize)]
pub struct HostConfig {
    #[arg(long, default_value = "localhost")]
    pub host: String,
    #[arg(long, default_value = "9989")]
    pub port: u16,
}

#[derive(Args, Clone, Debug, Serialize)]
pub struct ClientConfig {
    #[clap(flatten)]
    pub server: HostConfig,
    #[arg(long, default_value = "human")]
    pub client_type: ClientType,
    #[arg(long, default_value = "700")]
    pub bot_millis: u64,
}

#[derive(ValueEnum, Clone, Debug, Serialize)]
pub enum ClientType {
    Bot,
    Human,
}

pub fn web_client_args() -> GameArgs {
    GameArgs {
        exec_command: ExecCommand::Client(ClientConfig {
            server: HostConfig {
                host: String::from("localhost"),
                port: 9989,
            },
            client_type: ClientType::Human,
            bot_millis: 0,
        }),
    }
}
