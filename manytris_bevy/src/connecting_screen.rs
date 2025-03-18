use crate::cli_options::HostConfig;
use crate::states::PlayingState;
use crate::{cli_options::ManagerServerConfig, net_client::NetClientConfig};
use anyhow;
use bevy::color::palettes::basic::BLACK;
use bevy::prelude::*;
use bevy_mod_reqwest::*;
use manytris_game_manager_proto::{CreateResponse, GetAddressResponse};
use serde::de::DeserializeOwned;
use serde_json;
use std::time::Duration;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(PlayingState::Connecting), setup)
        .add_systems(OnExit(PlayingState::Connecting), teardown)
        .add_systems(Update, update.run_if(in_state(PlayingState::Connecting)));
}

#[derive(Component, Debug)]
pub struct ConnectingMarker;

#[derive(Component, Debug)]
pub struct TextMarker;

#[derive(Resource, Debug, Default)]
pub struct ConnectingState {
    attempts: usize,
    request_state: RequestState,
}

#[derive(Default, Debug)]
enum RequestState {
    #[default]
    NotStarted,
    WaitingForResponse,
    BackoffDelay(Duration),
}

fn setup(mut commands: Commands) {
    let ui_container = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ConnectingMarker,
        ))
        .id();

    let text_font = TextFont {
        font_size: 40.0,
        ..default()
    };
    let text_color = TextColor(BLACK.into());

    let progress_text = commands
        .spawn(Text("...".into()))
        .insert(text_font)
        .insert(text_color)
        .insert(TextMarker)
        .id();

    commands.entity(ui_container).add_children(&[progress_text]);

    commands.init_resource::<ConnectingState>();
    commands.remove_resource::<NetClientConfig>();
}

fn teardown(mut commands: Commands, marker_q: Query<Entity, With<ConnectingMarker>>) {
    commands.entity(marker_q.single()).despawn_recursive();
    commands.remove_resource::<ConnectingState>();
}

fn update(
    mut text_q: Query<&mut Text, With<TextMarker>>,
    mut state: ResMut<ConnectingState>,
    mut client: BevyReqwest,
    time: Res<Time<Fixed>>,
    manager_cfg: Res<ManagerServerConfig>,
) {
    match state.request_state {
        RequestState::NotStarted => {
            let request = client
                .get(format!("{}/server_address", manager_cfg.manager_server))
                .build()
                .unwrap();
            client
                .send(request)
                .on_response(get_address_response_handler)
                .on_error(get_address_error_handler);
            state.request_state = RequestState::WaitingForResponse;
            text_q.single_mut().0 = "Retrieving server info...".into();
        }
        RequestState::WaitingForResponse => {}
        RequestState::BackoffDelay(target_time) => {
            if time.elapsed() >= target_time {
                state.request_state = RequestState::NotStarted;
            }
        }
    }
}

fn get_address_response_handler(
    trigger: Trigger<ReqwestResponseEvent>,
    mut state: ResMut<ConnectingState>,
    mut text_q: Query<&mut Text, With<TextMarker>>,
    time: Res<Time<Fixed>>,
    mut client: BevyReqwest,
    manager_cfg: Res<ManagerServerConfig>,
    mut commands: Commands,
    mut next_play_state: ResMut<NextState<PlayingState>>,
) {
    use GetAddressResponse::*;
    let msg = match extract_response::<GetAddressResponse>(trigger.event()) {
        Ok(NoServer) => {
            let create_server_request = client
                .post(format!("{}/create_server", manager_cfg.manager_server))
                .build()
                .unwrap();
            client
                .send(create_server_request)
                .on_response(create_server_response_handler)
                .on_error(create_server_error_handler);
            state.on_error(time.elapsed());

            "Requesting New Server...".into()
        }
        Ok(Ready { host, host_port, .. }) => {
            commands.insert_resource(NetClientConfig(HostConfig {
                host: host.clone(),
                port: host_port,
            }));
            next_play_state.set(PlayingState::Playing);
            format!("server at: {host}:{host_port}")
        }
        Err(e) => {
            state.on_error(time.elapsed());
            format!("error: {e:?}. Retrying...")
        }
    };

    text_q.single_mut().0 = msg.clone();
    eprintln!("{msg}");
}

fn get_address_error_handler(
    trigger: Trigger<ReqwestErrorEvent>,
    mut state: ResMut<ConnectingState>,
    mut text_q: Query<&mut Text, With<TextMarker>>,
    time: Res<Time<Fixed>>,
) {
    let err = trigger.event();
    let msg = format!("Get Server Request Error: {err:?}");
    text_q.single_mut().0 = msg.clone();
    eprintln!("{msg}");
    state.on_error(time.elapsed());
}

fn create_server_response_handler(
    trigger: Trigger<ReqwestResponseEvent>,
    mut text_q: Query<&mut Text, With<TextMarker>>,
) {
    let message: String = match extract_response::<CreateResponse>(trigger.event()) {
        Ok(resp) => format!("Sever Creation: {resp:?}"),
        Err(e) => format!("Request error: {e:?}"),
    };

    println!("Create Server Response: {message}");

    if let Ok(mut text) = text_q.get_single_mut() {
        text.0 = message;
    };
}

fn create_server_error_handler(
    trigger: Trigger<ReqwestErrorEvent>,
    mut text_q: Query<&mut Text, With<TextMarker>>,
) {
    let err = trigger.event();
    let msg = format!("Create Server Request Error: {err:?}");
    text_q.single_mut().0 = msg.clone();
    eprintln!("{msg}");
}

fn extract_response<T: DeserializeOwned>(rr: &ReqwestResponseEvent) -> anyhow::Result<T> {
    let status = rr.status();
    if !status.is_success() {
        return Err(anyhow::Error::msg(format!("Error: status code: {status}")));
    }

    Ok(serde_json::from_str::<T>(rr.as_str()?)?)
}

impl ConnectingState {
    fn on_error(&mut self, now: Duration) {
        self.attempts += 1;
        self.request_state = RequestState::BackoffDelay(now + Duration::from_secs(10));
    }
}
