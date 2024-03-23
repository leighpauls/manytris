use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;

mod assets;
mod block_render;
mod field_blocks;
mod input;
mod root;
mod system_sets;
mod window_blocks;

pub fn run() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_plugins((
            root::plugin,
            window_blocks::plugin,
            field_blocks::plugin,
            input::plugin,
            assets::plugin,
            system_sets::plugin,
            block_render::plugin,
        ));

    app.run();
}
