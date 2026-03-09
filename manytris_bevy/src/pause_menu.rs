use crate::states::{
    is_menu_closed, is_menu_open, is_stand_alone, ConnectionState, ExecType, MenuState, PauseState,
    PlayingState,
};
use bevy::color::palettes::basic::*;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            setup_pause_ui
                .run_if(resource_changed::<MenuState>)
                .run_if(is_menu_open)
                .run_if(states::is_human),
            refresh_pause_ui_on_connection_change
                .run_if(resource_changed::<ConnectionState>)
                .run_if(is_menu_open)
                .run_if(states::is_human),
            update_pause_buttons
                .run_if(in_state(PlayingState::Playing))
                .run_if(is_menu_open)
                .run_if(states::is_human),
            cleanup_pause_ui
                .run_if(resource_changed::<MenuState>)
                .run_if(is_menu_closed)
                .run_if(states::is_human),
        ),
    )
    .add_systems(
        OnEnter(PlayingState::Restarting),
        apply_restart_transition.run_if(is_stand_alone),
    );
}

use crate::states;

#[derive(Component, Debug)]
enum PauseButton {
    Resume,
    Restart,
    QuitToMainMenu,
}

#[derive(Component, Debug)]
struct PauseMenuMarker;

fn setup_pause_ui(
    mut commands: Commands,
    exec_type: Res<ExecType>,
    connection_state: Res<ConnectionState>,
) {
    spawn_pause_ui(&mut commands, &exec_type, &connection_state);
}

fn spawn_pause_ui(
    commands: &mut Commands,
    exec_type: &ExecType,
    connection_state: &ConnectionState,
) {
    // Create semi-transparent overlay
    let overlay_container = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseMenuMarker,
            ZIndex(100),
        ))
        .id();

    // Create button container
    let button_container = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(20.0),
            ..default()
        })
        .id();

    let button_template = (
        Button,
        Node {
            width: Val::Px(300.0),
            height: Val::Px(100.0),
            border: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(GRAY.into()),
        BackgroundColor(WHITE.into()),
    );

    let button_text_font = TextFont {
        font_size: 20.0,
        ..default()
    };
    let button_text_color = TextColor(BLACK.into());

    let mut children = vec![];

    if *connection_state == ConnectionState::Disconnected {
        // Title text for disconnection
        let title = commands
            .spawn((
                Text("Connection Lost".into()),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ))
            .id();
        children.push(title);
    } else {
        // Resume button
        let resume_button = commands
            .spawn(button_template.clone())
            .insert(PauseButton::Resume)
            .id();
        let resume_text = commands
            .spawn((
                Text("Resume".into()),
                button_text_font.clone(),
                button_text_color,
            ))
            .id();
        commands.entity(resume_button).add_children(&[resume_text]);
        children.push(resume_button);

        // Restart button (standalone only)
        if *exec_type == ExecType::StandAlone {
            let restart_button = commands
                .spawn(button_template.clone())
                .insert(PauseButton::Restart)
                .id();
            let restart_text = commands
                .spawn((
                    Text("Restart".into()),
                    button_text_font.clone(),
                    button_text_color,
                ))
                .id();
            commands
                .entity(restart_button)
                .add_children(&[restart_text]);
            children.push(restart_button);
        }
    }

    // Quit button (always shown)
    let quit_button = commands
        .spawn(button_template)
        .insert(PauseButton::QuitToMainMenu)
        .id();
    let quit_text = commands
        .spawn((
            Text("Quit to Main Menu".into()),
            button_text_font,
            button_text_color,
        ))
        .id();
    commands.entity(quit_button).add_children(&[quit_text]);
    children.push(quit_button);

    commands.entity(button_container).add_children(&children);
    commands
        .entity(overlay_container)
        .add_children(&[button_container]);
}

fn refresh_pause_ui_on_connection_change(
    mut commands: Commands,
    exec_type: Res<ExecType>,
    connection_state: Res<ConnectionState>,
    pause_menu_q: Query<Entity, With<PauseMenuMarker>>,
) {
    // Only refresh if there's existing UI to replace
    if pause_menu_q.is_empty() {
        return;
    }
    for entity in &pause_menu_q {
        commands.entity(entity).despawn_recursive();
    }
    spawn_pause_ui(&mut commands, &exec_type, &connection_state);
}

fn update_pause_buttons(
    interaction_q: Query<(&Interaction, &PauseButton), Changed<Interaction>>,
    mut pause_state: ResMut<PauseState>,
    mut menu_state: ResMut<MenuState>,
    mut next_play_state: ResMut<NextState<PlayingState>>,
) {
    for (interaction, button) in &interaction_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match button {
            PauseButton::Resume => {
                *menu_state = MenuState::Closed;
                *pause_state = PauseState::Unpaused;
            }
            PauseButton::Restart => {
                *menu_state = MenuState::Closed;
                *pause_state = PauseState::Unpaused;
                next_play_state.set(PlayingState::Restarting);
            }
            PauseButton::QuitToMainMenu => {
                *menu_state = MenuState::Closed;
                *pause_state = PauseState::Unpaused;
                next_play_state.set(PlayingState::MainMenu);
            }
        }
        return;
    }
}

fn cleanup_pause_ui(mut commands: Commands, pause_menu_q: Query<Entity, With<PauseMenuMarker>>) {
    for entity in &pause_menu_q {
        commands.entity(entity).despawn_recursive();
    }
}

// Placeholder to apply the fake transition from playing to not playing back to playing
fn apply_restart_transition(mut next_play_state: ResMut<NextState<PlayingState>>) {
    next_play_state.set(PlayingState::Playing);
}
