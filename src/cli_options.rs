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
    Client(HostConfig),
    StandAlone,
}

#[derive(Args, Clone, Debug, Serialize)]
pub struct HostConfig {
    #[arg(long, default_value = "localhost")]
    pub host: String,
    #[arg(long, default_value = "9989")]
    pub port: u16,
}
