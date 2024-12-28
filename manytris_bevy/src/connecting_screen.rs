use crate::states::PlayingState;
use bevy::color::palettes::basic::BLACK;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(PlayingState::Connecting), setup)
        .add_systems(OnExit(PlayingState::Connecting), teardown)
        .add_systems(Update, update.run_if(in_state(PlayingState::Connecting)));
}

#[derive(Component, Debug)]
pub struct ConnectingMarker;

#[derive(Component, Debug)]
pub struct TextMarker;

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
}

fn teardown(mut commands: Commands, marker_q: Query<Entity, With<ConnectingMarker>>) {
    commands.entity(marker_q.single()).despawn_recursive();
}

fn update(mut text_q: Query<&mut Text, With<TextMarker>>) {
    let mut text = text_q.single_mut();
    text.0 = "Retrieving server info...".into();
}
