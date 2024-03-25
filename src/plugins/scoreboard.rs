use crate::plugins::assets;
use crate::plugins::root::GameRoot;
use crate::plugins::system_sets::StartupSystems;
use bevy::prelude::*;
use bevy::sprite::Anchor;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_scoreboard.in_set(StartupSystems::AfterRoot));
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
                text: Text::from_section("Level: 99\n\nLines: 12345", text_style),
                transform: Transform::from_xyz(-assets::BLOCK_SIZE * 5., assets::BLOCK_SIZE, 0.),
                text_anchor: Anchor::BottomLeft,
                ..default()
            },
        })
        .set_parent(root);
}
