use crate::plugins::states::PlayingState;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(PlayingState::MainMenu), setup)
        .add_systems(Update, update.run_if(in_state(PlayingState::MainMenu)))
        .add_systems(OnExit(PlayingState::MainMenu), tear_down);
}

#[derive(Component, Debug)]
pub enum MainMenuButtons {
    StartButton,
}

fn setup(mut commands: Commands) {
    let start_button = ButtonBundle {
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
    let start_button_text = TextBundle::from_section(
        "Start",
        TextStyle {
            font_size: 40.0,
            color: Color::BLACK,
            ..default()
        },
    );

    let button = commands
        .spawn(start_button)
        .insert(MainMenuButtons::StartButton)
        .id();
    commands.spawn(start_button_text).set_parent(button);
}

fn update(
    interation_q: Query<(&Interaction, &MainMenuButtons), Changed<Interaction>>,
    mut next_play_state: ResMut<NextState<PlayingState>>,
) {
    for (interaction, button) in &interation_q {
        println!("Updated interaction: {:?} for {:?}", interaction, button);
        if let (Interaction::Pressed, MainMenuButtons::StartButton) = (interaction, button) {
            next_play_state.set(PlayingState::Playing);
            return;
        }
    }
}

fn tear_down(mut commands: Commands, buttons_q: Query<Entity, With<MainMenuButtons>>) {
    for entity in &buttons_q {
        commands.entity(entity).despawn_recursive();
    }
}
