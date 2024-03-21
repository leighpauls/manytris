use crate::assets::RenderAssets;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

mod assets;
mod entities;
mod game_state;
mod preview_entities;
mod root_entity;
mod shapes;
mod upcoming;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .init_resource::<RenderAssets>()
        .add_systems(
            Startup,
            (
                root_entity::setup_root,
                (entities::setup_field, preview_entities::setup_previews),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                entities::update_for_input,
                entities::update_block_colors,
                preview_entities::update_preview_window,
            ),
        );

    app.run();
}
