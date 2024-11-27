use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use k8s_openapi::api::core::v1::{Container, ContainerPort, Node, Pod, PodSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{DeleteParams, PostParams};
use kube::{Api, Client, ResourceExt};
use std::collections::BTreeMap;
use tokio;

const NAMESPACE: &str = "manytris";
const GAME_POD_NAME: &str = "game-pod";
const SERVER_CONTAINER_NAME: &str = "server";
const IMAGE_NAME: &str = "registry.hub.docker.com/leighpauls/manytris:v0.4";
const SERVER_GAME_PORT_NAME: &str = "game-port";

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
    let client = Client::try_default().await?;

    match manager_args.cmd {
        ManagementCommand::Get => read_state(client).await,
        ManagementCommand::Create => create(client).await,
        ManagementCommand::Delete => delete(client).await,
    }
}

async fn read_state(client: Client) -> Result<()> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), NAMESPACE);
    let nodes: Api<Node> = Api::all(client.clone());

    if let Some(game_node) = get_game_pod(&pods).await? {
        let addr = get_server_address(&nodes, &game_node).await?;
        println!("Game address: {addr:?}");
    } else {
        println!("Game Pod not found");
    }

    Ok(())
}

async fn create(client: Client) -> Result<()> {
    let pods: Api<Pod> = Api::namespaced(client, NAMESPACE);

    if get_game_pod(&pods).await?.is_some() {
        println!("Game already exists");
        return Ok(());
    }

    let pod_spec = Pod {
        metadata: ObjectMeta {
            name: Some(GAME_POD_NAME.into()),
            annotations: Some(BTreeMap::from([(
                "autopilot.gke.io/host-port-assignment".to_string(),
                "{\"min\":8000,\"max\":20000}".to_string(),
            )])),
            ..Default::default()
        },
        spec: Some(PodSpec {
            containers: vec![Container {
                name: SERVER_CONTAINER_NAME.into(),
                image: Some(IMAGE_NAME.into()),
                ports: Some(vec![ContainerPort {
                    name: Some(SERVER_GAME_PORT_NAME.into()),
                    container_port: 9989,
                    host_port: Some(7001),
                    protocol: Some("TCP".into()),
                    host_ip: Some("0.0.0.0".into()),
                }]),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    };

    let new_pod_name = pods
        .create(&PostParams::default(), &pod_spec)
        .await?
        .name_any();
    println!("Created pod {new_pod_name}");
    Ok(())
}

async fn delete(client: Client) -> Result<()> {
    let pods: Api<Pod> = Api::namespaced(client, NAMESPACE);
    match pods.delete(GAME_POD_NAME, &DeleteParams::default()).await {
        Ok(_) => {
            println!("Started delete.");
            Ok(())
        }
        Err(kube::Error::Api(a)) if a.code == 404 => {
            println!("Pod not found");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

async fn get_server_address(nodes: &Api<Node>, pod: &Pod) -> Result<(String, i32)> {
    let node_name = pod
        .spec
        .as_ref()
        .context("Pod spec not available")?
        .node_name
        .as_ref()
        .context("Missing node name")?;

    let node = nodes.get(node_name).await.context("Name not available")?;

    let host = node
        .status
        .context("Status not available")?
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
        .context("No external ip or hostname address found")?;

    let port = pod
        .spec
        .as_ref()
        .context("Pod spec not available")?
        .containers
        .iter()
        .find_map(|c| {
            if c.name == "server" {
                Some(c.ports.as_ref())
            } else {
                None
            }
        })
        .context("Could not find server container")?
        .unwrap_or(&vec![])
        .iter()
        .find_map(|p| {
            if p.name.as_ref()? == "game-port" {
                p.host_port
            } else {
                None
            }
        })
        .context("Could not find host port for game-port")?;

    Ok((host, port))
}

async fn get_game_pod(pods: &Api<Pod>) -> Result<Option<Pod>> {
    match pods.get(GAME_POD_NAME).await {
        Ok(pod) => Ok(Some(pod)),
        Err(kube::Error::Api(e)) if e.code == 404 => Ok(None),
        Err(e) => Err(e.into()),
    }
}
