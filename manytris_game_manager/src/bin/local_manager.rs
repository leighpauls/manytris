use axum::http::{HeaderValue, Method};
use axum::routing::{get, post};
use axum::{Json, Router};
use clap::Parser;
use manytris_game_manager_proto::{CreateResponse, GetAddressResponse};
use tower_http::cors::CorsLayer;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "3000")]
    port: u16,

    #[arg(long, default_value = "localhost")]
    game_host: String,

    #[arg(long, default_value = "9989")]
    game_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let game_host = args.game_host.clone();
    let game_port = args.game_port;

    let cors = CorsLayer::new()
        .allow_origin("http://127.0.0.1:1334".parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS]);

    let app = Router::new()
        .route(
            "/server_address",
            get(move || async move {
                Json(GetAddressResponse::Ready {
                    host: game_host,
                    host_port: game_port,
                    container_port: game_port,
                    host_stats_port: 9990,
                    container_stats_port: 9990,
                })
            }),
        )
        .route(
            "/create_server",
            post(|| async { Json(CreateResponse::AlreadyExists) }),
        )
        .layer(cors);

    println!("Local manager listening on port {}", args.port);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", args.port)).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
        .await?;

    Ok(())
}
