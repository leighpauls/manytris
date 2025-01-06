use anyhow::Result;
use clap::{Parser, Subcommand};
use manytris_game_manager::k8s_commands::CommandClient;

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

    let cc = CommandClient::new().await?;

    match manager_args.cmd {
        ManagementCommand::Get => {
            let addr = cc.read_state().await?;
            println!("Game Address: {addr:?}");
        }
        ManagementCommand::Create => {
            let cr = cc.create().await?;
            println!("Created: {cr:?}");
        }
        ManagementCommand::Delete => {
            let dr = cc.delete().await?;
            println!("Deleted: {dr:?}");
        }
    }
    Ok(())
}
