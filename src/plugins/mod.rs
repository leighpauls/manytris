// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use crate::cli_options::ExecCommand;
use bevy::prelude::*;

mod assets;
mod block_render;
mod field_blocks;
mod input;
mod net_client;
mod net_game_control_manager;
mod net_listener;
mod net_protocol;
pub mod root;
mod scoreboard;
pub mod shape_producer;
mod system_sets;
mod window_blocks;

pub fn run(cfg: ExecCommand) {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins).add_plugins((
        root::common_plugin,
        window_blocks::plugin,
        field_blocks::plugin,
        assets::plugin,
        system_sets::plugin,
        block_render::plugin,
        scoreboard::plugin,
    ));

    match cfg {
        ExecCommand::Client(hc) => {
            app.insert_resource(net_client::NetClientConfig(hc))
                .add_plugins((
                    input::plugin,
                    root::client_plugin,
                    net_client::plugin,
                    net_game_control_manager::plugin,
                ));
        }
        ExecCommand::Server(hc) => {
            app.insert_resource(net_listener::NetListenerConfig(hc))
                .add_plugins((
                    net_listener::plugin,
                    shape_producer::plugin,
                    net_game_control_manager::plugin,
                ));
        }
        ExecCommand::StandAlone => {
            app.add_plugins((
                input::plugin,
                root::stand_alone_plugin,
                shape_producer::plugin,
            ));
        }
    }

    app.run();
}
