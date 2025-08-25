use anyhow::{bail, Context, Result};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::future;
use k8s_openapi::api::core::v1::Pod;
use kube::Api;
use manytris_game_manager::port_forward::{self, Forwarder};
use manytris_game_manager::CommandClient;
use manytris_game_manager_proto::{CreateResponse, DeleteResponse, GetAddressResponse};
use tokio::sync::Mutex;

use std::env;
use std::sync::Arc;

struct AppError(anyhow::Error);

type FwdState = Arc<Mutex<Option<(Forwarder, Forwarder)>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = env::var("PORT")
        .unwrap_or("3000".to_string())
        .parse::<u16>()?;

    let forwarder_arc: FwdState = Arc::new(Mutex::new(None));

    // build our application with a single route
    let app = Router::new()
        .route(
            "/server_address",
            get(get_address).with_state(forwarder_arc.clone()),
        )
        .route("/create_server", post(create))
        .route("/delete_server", post(delete));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
        .await?;

    println!("Closing forwarder...");

    let mut fwd_lock = forwarder_arc.lock().await;
    if let Some((game_fwd, stats_fwd)) = fwd_lock.take() {
        future::join_all(vec![game_fwd.exit_join(), stats_fwd.exit_join()])
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
    }

    Ok(())
}

async fn get_address(forwarder: State<FwdState>) -> Result<Json<GetAddressResponse>, AppError> {
    let cc = CommandClient::new().await?;
    let result = cc.read_state().await?;

    println!("Get response: {result:?}");

    Ok(Json(match result {
        GetAddressResponse::Ready {
            host,
            container_port,
            container_stats_port,
            ..
        } if host.as_str() == "docker-desktop" => {
            let (host_game_port, host_stats_port) = ensure_forwarding(
                forwarder.0.clone(),
                &cc.pods,
                container_port,
                container_stats_port,
            )
            .await?;
            GetAddressResponse::Ready {
                host: "localhost".to_string(),
                host_port: host_game_port,
                container_port,
                host_stats_port,
                container_stats_port,
            }
        }
        r => r,
    }))
}

async fn create() -> Result<Json<CreateResponse>, AppError> {
    let resp = CommandClient::new().await?.create().await;
    println!("Create response: {resp:?}");
    Ok(Json(resp?))
}

async fn delete() -> Result<Json<DeleteResponse>, AppError> {
    Ok(Json(CommandClient::new().await?.delete().await?))
}

async fn ensure_forwarding(
    forwarder_state: FwdState,
    pods: &Api<Pod>,
    pod_port: u16,
    pod_stats_port: u16,
) -> Result<(u16, u16)> {
    let mut fwd_lock = forwarder_state.lock().await;

    if let Some(ref forwarders) = *fwd_lock {
        println!("forwarder already running");
        Ok((forwarders.0.listener_port(), forwarders.1.listener_port()))
    } else {
        println!("start new forwarder");

        let port_vec = vec![pod_port, pod_stats_port];
        let mut forwarders =
            port_forward::bind_ports(pods.clone(), "game-pod".to_string(), &port_vec)
                .await?
                .into_iter();

        let game_forwarder = forwarders.next().context("Expected game forwarder")?;
        let stats_forwarder = forwarders.next().context("Expected stats forwarder")?;
        if forwarders.next().is_some() {
            bail!("Expected only 2 forwarders");
        }

        let res = (
            game_forwarder.listener_port(),
            stats_forwarder.listener_port(),
        );
        *fwd_lock = Some((game_forwarder, stats_forwarder));

        Ok(res)
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(e: E) -> Self {
        Self(e.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        println!("Error response: {}", self.0.to_string());
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}
