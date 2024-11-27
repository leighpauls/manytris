use crate::root::GameRoot;
use crate::states::PlayingState;
use crate::system_sets::UpdateSystems;
use crate::{assets, states};
use bevy::prelude::*;
use bevy::sprite::Anchor;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            add_scoreboard_to_root.in_set(UpdateSystems::PreRender),
            update_scoreboard.in_set(UpdateSystems::Render),
        )
            .run_if(in_state(PlayingState::Playing))
            .run_if(states::headed),
    );
}

#[derive(Component)]
struct ScoreboardComponent();

#[derive(Bundle)]
struct ScoreboardBundle {
    scoreboard: ScoreboardComponent,
    text_bundle: Text2dBundle,
}

fn add_scoreboard_to_root(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    root_entity_q: Query<Entity, Added<GameRoot>>,
) {
    for root_entity in &root_entity_q {
        let font = asset_server.load("fonts/white-rabbit.ttf");
        let text_style = TextStyle {
            font: font.clone(),
            font_size: 15.,
            color: Color::WHITE,
        };

        commands
            .spawn(ScoreboardBundle {
                scoreboard: ScoreboardComponent(),
                text_bundle: Text2dBundle {
                    text: Text::from_section(get_score_text(0, 0), text_style),
                    transform: Transform::from_xyz(
                        -assets::BLOCK_SIZE * 5.,
                        assets::BLOCK_SIZE,
                        0.,
                    ),
                    text_anchor: Anchor::BottomLeft,
                    ..default()
                },
            })
            .set_parent(root_entity);
    }
}

fn update_scoreboard(
    q_root: Query<&GameRoot>,
    mut q_scoreboard: Query<(&mut Text, &Parent), With<ScoreboardComponent>>,
) {
    for (mut score_text, parent_entity) in q_scoreboard.iter_mut() {
        let game_root = q_root.get(parent_entity.get()).unwrap();
        score_text.sections[0].value = get_score_text(
            game_root.active_game.level,
            game_root.active_game.lines_cleared,
        );
    }
}

fn get_score_text(level: i32, lines_cleared: i32) -> String {
    format!("Level: {}\n\nLines: {}", level, lines_cleared)
}