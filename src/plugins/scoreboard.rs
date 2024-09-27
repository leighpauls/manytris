use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::plugins::assets;
use crate::plugins::root::GameRoot;
use crate::plugins::system_sets::UpdateSystems;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update_scoreboard.in_set(UpdateSystems::Render));
}

#[derive(Component)]
struct ScoreboardComponent();

#[derive(Bundle)]
struct ScoreboardBundle {
    scoreboard: ScoreboardComponent,
    text_bundle: Text2dBundle,
}

pub fn spawn_scoreboard(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    root_entity: Entity,
) {
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
                transform: Transform::from_xyz(-assets::BLOCK_SIZE * 5., assets::BLOCK_SIZE, 0.),
                text_anchor: Anchor::BottomLeft,
                ..default()
            },
        })
        .set_parent(root_entity);
}

fn update_scoreboard(
    q_root: Query<&GameRoot>,
    mut q_scoreboard: Query<&mut Text, With<ScoreboardComponent>>,
) {
    let Some(game_root) = GameRoot::for_single(q_root.get_single()) else {
        return;
    };

    let mut score_text = q_scoreboard.single_mut();
    score_text.sections[0].value = get_score_text(
        game_root.active_game.level,
        game_root.active_game.lines_cleared,
    );
}

fn get_score_text(level: i32, lines_cleared: i32) -> String {
    format!("Level: {}\n\nLines: {}", level, lines_cleared)
}
