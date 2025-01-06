use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use manytris_game_manager::CommandClient;
use manytris_game_manager_proto::{CreateResponse, DeleteResponse, GetAddressResponse};

use std::env;

struct AppError(anyhow::Error);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = env::var("PORT")
        .unwrap_or("3000".to_string())
        .parse::<u16>()?;

    // build our application with a single route
    let app = Router::new()
        .route("/server_address", get(get_address))
        .route("/create_server", post(create))
        .route("/delete_server", post(delete));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_address() -> Result<Json<GetAddressResponse>, AppError> {
    Ok(Json(CommandClient::new().await?.read_state().await?))
}

async fn create() -> Result<Json<CreateResponse>, AppError> {
    Ok(Json(CommandClient::new().await?.create().await?))
}

async fn delete() -> Result<Json<DeleteResponse>, AppError> {
    Ok(Json(CommandClient::new().await?.delete().await?))
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
