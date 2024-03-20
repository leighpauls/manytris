use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

mod field_rendering;
mod game_state;
mod shapes;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, field_rendering::setup)
        .add_systems(Update, field_rendering::update_field);

    app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    app.add_plugins(LogDiagnosticsPlugin::default());

    app.run();
}
