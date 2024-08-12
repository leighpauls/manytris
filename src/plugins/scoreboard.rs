use crate::plugins::assets;
use crate::plugins::root::GameRoot;
use crate::plugins::system_sets::{StartupSystems, UpdateSystems};
use bevy::prelude::*;
use bevy::sprite::Anchor;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_scoreboard.in_set(StartupSystems::AfterRoot))
        .add_systems(Update, update_scoreboard.in_set(UpdateSystems::Render));
}

#[derive(Component)]
struct ScoreboardComponent();

#[derive(Bundle)]
struct ScoreboardBundle {
    scoreboard: ScoreboardComponent,
    text_bundle: Text2dBundle,
}

fn setup_scoreboard(
    mut commands: Commands,
    q_root: Query<Entity, With<GameRoot>>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/white-rabbit.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 15.,
        color: Color::WHITE,
    };

    let root = q_root.single();
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
        .set_parent(root);
}

fn update_scoreboard(
    q_root: Query<&GameRoot>,
    mut q_scoreboard: Query<&mut Text, With<ScoreboardComponent>>,
) {
    let game_root = q_root.single();
    let (level, lines_cleared) = if let Some(active_game) = &game_root.active_game {
        (active_game.level, active_game.lines_cleared)
    } else {
        (0, 0)
    };

    let mut score_text = q_scoreboard.single_mut();
    score_text.sections[0].value = get_score_text(level, lines_cleared);
}

fn get_score_text(level: i32, lines_cleared: i32) -> String {
    format!("Level: {}\n\nLines: {}", level, lines_cleared)
}
