use crate::k8s_commands::CreateResponse::AlreadyExists;
use crate::k8s_commands::DeleteResponse::{Deleting, NotFound};
use crate::k8s_commands::GetAddressResponse::NoServer;
use anyhow::{Context, Result};
use axum::http::Uri;
use gcp_auth;
use k8s_openapi::api::core::v1::{Container, ContainerPort, Node, Pod, PodSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{DeleteParams, PostParams};
use kube::{Api, Client, Config};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;

const NAMESPACE: &str = "manytris";
const GAME_POD_NAME: &str = "game-pod";
const SERVER_CONTAINER_NAME: &str = "server";
const IMAGE_NAME: &str = "registry.hub.docker.com/leighpauls/manytris:v0.5";

const SERVER_GAME_PORT_NAME: &str = "game-port";

#[derive(Serialize, Deserialize, Debug)]
pub enum GetAddressResponse {
    NoServer,
    Ok { host: String, port: u16 },
}

pub async fn read_state() -> Result<GetAddressResponse> {
    let client = get_client().await?;
    let pods: Api<Pod> = Api::namespaced(client.clone(), NAMESPACE);
    let nodes: Api<Node> = Api::all(client.clone());

    let Some(pod) = get_game_pod(&pods).await? else {
        return Ok(NoServer);
    };

    let (host, port) = get_server_address(&nodes, &pod).await?;
    Ok(GetAddressResponse::Ok { host, port })
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CreateResponse {
    AlreadyExists,
    Created,
}

pub async fn create() -> Result<CreateResponse> {
    let client = get_client().await?;
    let pods: Api<Pod> = Api::namespaced(client, NAMESPACE);

    if get_game_pod(&pods).await?.is_some() {
        return Ok(AlreadyExists);
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

    pods.create(&PostParams::default(), &pod_spec).await?;
    Ok(CreateResponse::Created)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeleteResponse {
    NotFound,
    Deleting,
}

pub async fn delete() -> Result<DeleteResponse> {
    let client = get_client().await?;
    let pods: Api<Pod> = Api::namespaced(client, NAMESPACE);
    match pods.delete(GAME_POD_NAME, &DeleteParams::default()).await {
        Ok(_) => Ok(Deleting),
        Err(kube::Error::Api(a)) if a.code == 404 => Ok(NotFound),
        Err(e) => Err(e.into()),
    }
}

async fn get_client() -> Result<Client> {
    if let Ok(api_server) = env::var("KUBE_API_SERVER") {
        println!("Use GCP auth");
        let token = gcp_auth::provider()
            .await?
            .token(&["https://www.googleapis.com/auth/cloud-platform"])
            .await?;

        let mut kube_config = Config::new(Uri::try_from(api_server)?);
        kube_config.headers.push((
            "Authorization".try_into()?,
            format!("Bearer {}", token.as_str()).try_into()?,
        ));
        Ok(Client::try_from(kube_config)?)
    } else {
        println!("Use default auth");
        Ok(Client::try_default().await?)
    }
}

async fn get_server_address(nodes: &Api<Node>, pod: &Pod) -> Result<(String, u16)> {
    let node_name = pod
        .spec
        .as_ref()
        .context("Pod spec not available")?
        .node_name
        .as_ref()
        .context("Missing node name")?;

    let node = nodes
        .get(node_name)
        .await
        .with_context(|| format!("Name not available: {node_name}"))?;

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

    Ok((host, port as u16))
}

async fn get_game_pod(pods: &Api<Pod>) -> Result<Option<Pod>> {
    match pods.get(GAME_POD_NAME).await {
        Ok(pod) => Ok(Some(pod)),
        Err(kube::Error::Api(e)) if e.code == 404 => Ok(None),
        Err(e) => Err(e.into()),
    }
}
