use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use k8s_openapi::api::core::v1::{Node, Pod};
use kube::api::ListParams;
use kube::{Api, Client, ResourceExt};
use tokio;

const NAMESPACE: &str = "manytris";

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

fn main() -> Result<()> {
    let manager_args = ManagerArgs::parse();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(handle_cmd(manager_args))
}

async fn handle_cmd(manager_args: ManagerArgs) -> Result<()> {
    match manager_args.cmd {
        ManagementCommand::Get => read_state().await,
        ManagementCommand::Create => create().await,
        ManagementCommand::Delete => delete().await,
    }
}

async fn read_state() -> Result<()> {
    println!("Hello, world!");

    let client = Client::try_default().await?;

    let pods: Api<Pod> = Api::namespaced(client.clone(), NAMESPACE);
    let nodes: Api<Node> = Api::all(client.clone());
    for p in pods.list(&ListParams::default()).await? {
        let pod_name = p.name_any();
        println!("Found pod: {pod_name}");
        match get_server_address(&nodes, &p).await {
            Ok(address) => {
                println!("Server address: {address:?}")
            }
            Err(e) => {
                println!("Server address not found: {e:?}")
            }
        }
    }

    println!("Done!");

    Ok(())
}

async fn get_server_address(nodes: &Api<Node>, pod: &Pod) -> Result<(String, i32)> {
    let node_name = pod
        .spec
        .as_ref()
        .with_context(|| "Pod spec not available")?
        .node_name
        .as_ref()
        .with_context(|| "Missing node name")?;

    let node = nodes
        .get(node_name)
        .await
        .with_context(|| "Name not available")?;

    let host = node
        .status
        .with_context(|| "Status not available")?
        .addresses
        .unwrap_or_default()
        .iter()
        .find_map(|addr| {
            if addr.type_ == "ExternalIP" || addr.type_ == "Hostname" {
                Some(addr.address.clone())
            } else {
                None
            }
        })
        .with_context(|| "No external ip or hostname address found")?;

    let port = pod
        .spec
        .as_ref()
        .with_context(|| "Pod spec not available")?
        .containers
        .iter()
        .find_map(|c| {
            if c.name == "server" {
                Some(c.ports.as_ref())
            } else {
                None
            }
        })
        .with_context(|| "Could not find server container")?
        .unwrap_or(&vec![])
        .iter()
        .find_map(|p| {
            if p.name.as_ref()? == "game-port" {
                p.host_port
            } else {
                None
            }
        })
        .with_context(|| "Could not find host port for game-port")?;

    Ok((host, port))
}

async fn create() -> Result<()> {
    Err(anyhow::anyhow!("Not implemented"))
}

async fn delete() -> Result<()> {
    Err(anyhow::anyhow!("Not implemented"))
}
