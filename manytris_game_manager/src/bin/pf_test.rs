use anyhow::Result;
use manytris_game_manager::{port_forward, CommandClient};

#[tokio::main]
async fn main() -> Result<()> {
    let cc = CommandClient::new().await?;
    let forwarder = port_forward::bind_port(cc.pods, "game-pod".to_string(), 9989).await?;

    let port = forwarder.listener_port();
    println!("Listening on port {port}");

    tokio::signal::ctrl_c().await?;
    forwarder.exit_join().await?;

    Ok(())
}
