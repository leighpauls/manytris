use anyhow::{Context, Result};
use axum::http::Uri;
use gcp_auth;
use k8s_openapi::api::core::v1::{Container, ContainerPort, Node, Pod, PodSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{DeleteParams, PostParams};
use kube::{Api, Client, Config};
use manytris_game_manager_proto::{CreateResponse, DeleteResponse, GetAddressResponse};
use std::collections::BTreeMap;
use std::env;

const NAMESPACE: &str = "manytris";
const GAME_POD_NAME: &str = "game-pod";
const SERVER_CONTAINER_NAME: &str = "server";

const VERSION_STRING_RAW: &str = include_str!("../../docker/version.txt");

const SERVER_GAME_PORT_NAME: &str = "game-port";

pub struct CommandClient {
    pub pods: Api<Pod>,
    nodes: Api<Node>,
}

impl CommandClient {
    pub async fn new() -> Result<Self> {
        let client = get_client().await?;
        let pods: Api<Pod> = Api::namespaced(client.clone(), NAMESPACE);
        let nodes: Api<Node> = Api::all(client.clone());
        Ok(Self { pods, nodes })
    }

    pub async fn read_state(&self) -> Result<GetAddressResponse> {
        let Some(pod) = self.get_game_pod().await? else {
            return Ok(GetAddressResponse::NoServer);
        };

        let (host, host_port, container_port) = self.get_server_address(&pod).await?;
        Ok(GetAddressResponse::Ready {
            host,
            host_port,
            container_port,
        })
    }

    pub async fn create(&self) -> Result<CreateResponse> {
        if let Some(existing_pod) = self.get_game_pod().await? {
            return Ok(CreateResponse::AlreadyExists);
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
                    image: Some(dev_image_name()),
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

        self.pods.create(&PostParams::default(), &pod_spec).await?;
        Ok(CreateResponse::Created)
    }

    pub async fn delete(&self) -> Result<DeleteResponse> {
        match self
            .pods
            .delete(GAME_POD_NAME, &DeleteParams::default())
            .await
        {
            Ok(_) => Ok(DeleteResponse::Deleting),
            Err(kube::Error::Api(a)) if a.code == 404 => Ok(DeleteResponse::NotFound),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_server_address(&self, pod: &Pod) -> Result<(String, u16, u16)> {
        let node_name = pod
            .spec
            .as_ref()
            .context("Pod spec not available")?
            .node_name
            .as_ref()
            .context("Missing node name")?;

        let node = self
            .nodes
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

        let (host_port, container_port) = pod
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
                    p.host_port.map(|hp| (hp, p.container_port))
                } else {
                    None
                }
            })
            .context("Could not find host port for game-port")?;

        Ok((host, host_port as u16, container_port as u16))
    }

    async fn get_game_pod(&self) -> Result<Option<Pod>> {
        match self.pods.get(GAME_POD_NAME).await {
            Ok(pod) => Ok(Some(pod)),
            Err(kube::Error::Api(e)) if e.code == 404 => Ok(None),
            Err(e) => Err(e.into()),
        }
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

fn prod_image_name() -> String {
    let vs = version_string();
    format!("registry.hub.docker.com/leighpauls/manytris:{vs}-prod")
}

fn dev_image_name() -> String {
    let vs = version_string();
    format!("registry.hub.docker.com/leighpauls/manytris:{vs}-dev")
}

fn version_string() -> String {
    VERSION_STRING_RAW.trim().to_string()
}
