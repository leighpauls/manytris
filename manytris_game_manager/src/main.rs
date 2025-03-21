use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use k8s_openapi::api::core::v1::Pod;
use kube::Api;
use manytris_game_manager::port_forward::{self, Forwarder};
use manytris_game_manager::CommandClient;
use manytris_game_manager_proto::{CreateResponse, DeleteResponse, GetAddressResponse};
use tokio::sync::Mutex;

use std::env;
use std::sync::Arc;

struct AppError(anyhow::Error);

type FwdState = Arc<Mutex<Option<Forwarder>>>;

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
    if let Some(forwarder) = fwd_lock.take() {
        forwarder.exit_join().await?;
    }

    Ok(())
}

async fn get_address(forwarder: State<FwdState>) -> Result<Json<GetAddressResponse>, AppError> {
    let cc = CommandClient::new().await?;
    let result = cc.read_state().await?;

    println!("Get response: {result:?}");

    if let GetAddressResponse::Ready {
        host,
        container_port,
        ..
    } = &result
    {
        if host.as_str() == "docker-desktop" {
            let local_host_port =
                ensure_forwarding(forwarder.0.clone(), &cc.pods, *container_port).await?;
            return Ok(Json(GetAddressResponse::Ready {
                host: "localhost".to_string(),
                host_port: local_host_port,
                container_port: *container_port,
            }));
        }
    }
    Ok(Json(result))
}

async fn create() -> Result<Json<CreateResponse>, AppError> {
    let resp = CommandClient::new().await?.create().await?;
    println!("Create response: {resp:?}");
    Ok(Json(resp))
}

async fn delete() -> Result<Json<DeleteResponse>, AppError> {
    Ok(Json(CommandClient::new().await?.delete().await?))
}

async fn ensure_forwarding(
    forwarder_state: FwdState,
    pods: &Api<Pod>,
    pod_port: u16,
) -> Result<u16> {
    let mut fwd_lock = forwarder_state.lock().await;

    if let Some(ref forwarder) = *fwd_lock {
        println!("forwarder already running");
        Ok(forwarder.listener_port())
    } else {
        println!("start new forwarder");

        let new_forwarder =
            port_forward::bind_port(pods.clone(), "game-pod".to_string(), pod_port).await?;

        let res = new_forwarder.listener_port();
        *fwd_lock = Some(new_forwarder);

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
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}
