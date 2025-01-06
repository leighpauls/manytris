use crate::states::PlayingState;
use anyhow;
use bevy::color::palettes::basic::BLACK;
use bevy::prelude::*;
use bevy_mod_reqwest::*;
use manytris_game_manager_proto::GetAddressResponse;
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
) {
    match state.request_state {
        RequestState::NotStarted => {
            let request = client
                .get("https://manytris-manager-265251374100.us-west1.run.app/server_address")
                .build()
                .unwrap();
            client
                .send(request)
                .on_response(response_handler)
                .on_error(error_handler);
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

fn response_handler(
    trigger: Trigger<ReqwestResponseEvent>,
    mut state: ResMut<ConnectingState>,
    mut text_q: Query<&mut Text, With<TextMarker>>,
    time: Res<Time<Fixed>>,
) {
    use GetAddressResponse::*;
    match extract_response::<GetAddressResponse>(trigger.event()) {
        Ok(NoServer) => {
            text_q.single_mut().0 = format!("No server available...");
            state.on_error(time.elapsed());
        }
        Ok(Ready { host, port }) => {
            text_q.single_mut().0 = format!("server at: {host}:{port}");
        }
        Err(e) => {
            text_q.single_mut().0 = format!("error: {e:?}. Retrying...");
            state.on_error(time.elapsed());
        }
    }
}

fn extract_response<T: DeserializeOwned>(rr: &ReqwestResponseEvent) -> anyhow::Result<T> {
    let status = rr.status();
    if !status.is_success() {
        return Err(anyhow::Error::msg(format!("Error: status code: {status}")));
    }

    Ok(serde_json::from_str::<T>(rr.as_str()?)?)
}

fn error_handler(
    trigger: Trigger<ReqwestErrorEvent>,
    mut state: ResMut<ConnectingState>,
    mut text_q: Query<&mut Text, With<TextMarker>>,
    time: Res<Time<Fixed>>,
) {
    let err = trigger.event();
    text_q.single_mut().0 = format!("Error: {err:?}");
    state.on_error(time.elapsed());
}

impl ConnectingState {
    fn on_error(&mut self, now: Duration) {
        self.attempts += 1;
        self.request_state = RequestState::BackoffDelay(now + Duration::from_secs(10));
    }
}
