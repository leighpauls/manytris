// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use crate::cli_options::ExecCommand;

mod assets;
mod block_render;
mod field_blocks;
mod input;
mod net_client;
mod net_listener;
pub mod root;
mod scoreboard;
mod shape_producer;
mod system_sets;
mod window_blocks;
mod net_protocol;


pub fn run(cfg: ExecCommand) {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        /* .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        )) */
        .add_plugins((
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
            app.insert_resource(net_client::NetClientConfig {
                host: hc.host,
                port: hc.port,
            })
            .add_plugins((input::plugin, root::client_plugin, net_client::plugin));
        }
        ExecCommand::Server(hc) => {
            app.insert_resource(net_listener::NetListenerConfig {
                host: hc.host,
                port: hc.port,
            })
            .add_plugins((net_listener::plugin, shape_producer::plugin));
        }
        ExecCommand::StandAlone => {panic!("Not implemented")}
    }

    app.run();
}
