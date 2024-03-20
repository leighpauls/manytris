use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

mod entities;
mod game_state;
mod shapes;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_systems(
            Startup,
            (entities::setup_assets, entities::setup_field).chain(),
        )
        .add_systems(Update, (entities::update_for_input, entities::update_block_colors));

    app.run();
}
