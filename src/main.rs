use crate::assets::RenderAssets;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

mod assets;
mod entities;
mod game_state;
mod input;
mod preview_entities;
mod root_entity;
mod shapes;
mod system_sets;
mod upcoming;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_plugins((
            root_entity::root_plugin,
            preview_entities::preview_plugin,
            entities::entities_plugin,
            input::input_plugin,
            assets::assets_plugin,
            system_sets::system_sets_plugin,
        ));

    app.run();
}
