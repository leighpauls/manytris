use anyhow::Result;
use axum::http::StatusCode;
use axum::Json;
use axum::{response::IntoResponse, routing::get};
use bevy::prelude::*;
use bevy_defer::{AsyncAccess, AsyncWorld};
use bevy_webserver::{BevyWebServerPlugin, RouterAppExt, WebServerConfig};
use manytris_game_manager_proto::{StatsServerResponse, STATS_SERVER_PORT, STATS_SERVER_ROUTE};

use crate::game_container::GameContainer;
use crate::net_listener::ServerListenerComponent;

struct AppError(anyhow::Error);

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

pub fn plugin(app: &mut App) {
    app.add_plugins(BevyWebServerPlugin)
        .insert_resource(WebServerConfig {
            port: STATS_SERVER_PORT,
            ..default()
        })
        .route(STATS_SERVER_ROUTE, get(cur_players));
}

async fn cur_players() -> Result<Json<StatsServerResponse>, AppError> {
    let (num_connected_players, connectionless_time_secs) = AsyncWorld
        .query_single::<&mut ServerListenerComponent>()
        .get_mut(|listener| {
            (
                listener.get_num_players() as u16,
                listener.get_connectionless_time().as_secs() as u32,
            )
        })?;

    let num_active_games = AsyncWorld
        .query_single::<&mut GameContainer>()
        .get_mut(|gc| gc.get_num_active_games() as u16)?;

    Ok(Json::from(StatsServerResponse {
        num_connected_players,
        num_active_games,
        connectionless_time_secs,
    }))
}
