use crate::states::{ExecType, MultiplayerType, PlayingState};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(PlayingState::MainMenu), setup)
        .add_systems(Update, update.run_if(in_state(PlayingState::MainMenu)))
        .add_systems(OnExit(PlayingState::MainMenu), tear_down);
}

#[derive(Component, Debug)]
pub enum MainMenuButtons {
    StartStandAloneButton,
    StartMultiplayerButton,
}

#[derive(Component, Debug)]
pub struct MainMenu;

fn setup(mut commands: Commands) {
    let main_menu_container = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .insert(MainMenu)
        .id();

    let button_template = ButtonBundle {
        style: Style {
            width: Val::Px(300.0),
            height: Val::Px(100.0),
            border: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        border_color: Color::GRAY.into(),
        background_color: Color::WHITE.into(),
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 40.0,
        color: Color::BLACK,
        ..default()
    };

    let start_stand_alone_button = commands
        .spawn(button_template.clone())
        .insert(MainMenuButtons::StartStandAloneButton)
        .id();
    let start_stand_alone_text = commands
        .spawn(TextBundle::from_section(
            "Start Single Player",
            button_text_style.clone(),
        ))
        .id();

    let start_multiplayer_button = commands
        .spawn(button_template)
        .insert(MainMenuButtons::StartMultiplayerButton)
        .id();
    let start_multiplayer_text = commands
        .spawn(TextBundle::from_section(
            "Start Multiplayer",
            button_text_style,
        ))
        .id();

    commands
        .entity(main_menu_container)
        .push_children(&[start_stand_alone_button, start_multiplayer_button]);
    commands
        .entity(start_stand_alone_button)
        .push_children(&[start_stand_alone_text]);
    commands
        .entity(start_multiplayer_button)
        .push_children(&[start_multiplayer_text]);
}

fn update(
    interaction_q: Query<(&Interaction, &MainMenuButtons), Changed<Interaction>>,
    mut next_play_state: ResMut<NextState<PlayingState>>,
    mut exec_type: ResMut<ExecType>,
) {
    for (interaction, button) in &interaction_q {
        match interaction {
            Interaction::Pressed => {}
            _ => {
                continue;
            }
        }
        match button {
            MainMenuButtons::StartStandAloneButton => {
                *exec_type = ExecType::StandAlone;
                next_play_state.set(PlayingState::Playing);
            }
            MainMenuButtons::StartMultiplayerButton => {
                *exec_type = ExecType::MultiplayerClient(MultiplayerType::Human);
                next_play_state.set(PlayingState::Connecting);
            }
        }
        return;
    }
}

fn tear_down(mut commands: Commands, main_menu_q: Query<Entity, With<MainMenu>>) {
    for entity in &main_menu_q {
        commands.entity(entity).despawn_recursive();
    }
}
