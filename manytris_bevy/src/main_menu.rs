use crate::states::{ExecType, MultiplayerType, PlayingState};
use bevy::color::palettes::basic::*;
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
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .insert(MainMenu)
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

    let start_stand_alone_button = commands
        .spawn(button_template.clone())
        .insert(MainMenuButtons::StartStandAloneButton)
        .id();
    let start_stand_alone_text = commands
        .spawn((
            Text("Start Single Player".into()),
            button_text_font.clone(),
            button_text_color,
        ))
        .id();

    let start_multiplayer_button = commands
        .spawn(button_template)
        .insert(MainMenuButtons::StartMultiplayerButton)
        .id();
    let start_multiplayer_text = commands
        .spawn((
            Text("Start Multiplayer".into()),
            button_text_font,
            button_text_color,
        ))
        .id();

    commands
        .entity(main_menu_container)
        .add_children(&[start_stand_alone_button, start_multiplayer_button]);
    commands
        .entity(start_stand_alone_button)
        .add_children(&[start_stand_alone_text]);
    commands
        .entity(start_multiplayer_button)
        .add_children(&[start_multiplayer_text]);
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
