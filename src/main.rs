use crate::assets::RenderAssets;
use crate::input::{InputEvent, RepeatTimes};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

mod assets;
mod entities;
mod game_state;
mod input;
mod preview_entities;
mod root_entity;
mod shapes;
mod upcoming;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_plugins(preview_entities::preview_plugin)
        .init_resource::<RenderAssets>()
        .init_resource::<RepeatTimes>()
        .add_event::<InputEvent>()
        .add_systems(
            Startup,
            (root_entity::setup_root, entities::setup_field).chain(),
        )
        .add_systems(
            Update,
            (
                input::update_for_input,
                entities::update_field_tick,
                entities::update_block_colors,
            )
                .chain(),
        );

    app.run();
}
