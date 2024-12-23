use crate::states::PlayingState;
use bevy::prelude::*;
use bevy_mod_reqwest::ReqwestPlugin;

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
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .insert(ConnectingMarker)
        .id();

    let text_style = TextStyle {
        font_size: 40.0,
        color: Color::BLACK,
        ..default()
    };

    let progress_text = commands
        .spawn(TextBundle::from_section(
            "...",
            text_style,
        ))
        .insert(TextMarker)
        .id();

    commands
        .entity(ui_container)
        .push_children(&[progress_text]);
}

fn teardown(mut commands: Commands, marker_q: Query<Entity, With<ConnectingMarker>>) {
    commands.entity(marker_q.single()).despawn_recursive();
}

fn update(mut text_q: Query<&mut Text, With<TextMarker>>) {
    let mut text = text_q.single_mut();
    text.sections[0].value = "Retrieving server info...".into();
    
}
