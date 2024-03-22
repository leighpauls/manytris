use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

mod assets;
mod field_blocks;
mod input;
mod window_blocks;
mod root;
mod system_sets;
mod block_render;

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
            block_render::plugin
        ));

    app.run();
}
