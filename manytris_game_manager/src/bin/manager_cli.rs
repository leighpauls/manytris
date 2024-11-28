use anyhow::Result;
use clap::{Parser, Subcommand};
use manytris_game_manager::k8s_commands;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ManagerArgs {
    #[command(subcommand)]
    pub cmd: ManagementCommand,
}

#[derive(Subcommand, Clone, Debug)]
pub enum ManagementCommand {
    Get,
    Create,
    Delete,
}

#[tokio::main]
async fn main() -> Result<()> {
    let manager_args = ManagerArgs::parse();

    match manager_args.cmd {
        ManagementCommand::Get => {
            let addr = k8s_commands::read_state().await?;
            println!("Game Address: {addr:?}");
        }
        ManagementCommand::Create => {
            let cr = k8s_commands::create().await?;
            println!("Created: {cr:?}");
        }
        ManagementCommand::Delete => {
            let dr = k8s_commands::delete().await?;
            println!("Deleted: {dr:?}");
        }
    }
    Ok(())
}
