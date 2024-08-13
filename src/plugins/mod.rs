// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

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

pub enum GameConfig {
    Client(HostConfig),
    ReplicaServer(HostConfig),
}

pub struct HostConfig {
    pub host: String,
    pub port: u16,
}

pub fn run(cfg: GameConfig) {
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
        GameConfig::Client(hc) => {
            app.insert_resource(net_client::NetClientConfig {
                host: hc.host,
                port: hc.port,
            })
            .add_plugins((input::plugin, root::client_plugin, net_client::plugin));
        }
        GameConfig::ReplicaServer(hc) => {
            app.insert_resource(net_listener::NetListenerConfig {
                host: hc.host,
                port: hc.port,
            })
            .add_plugins((net_listener::plugin, shape_producer::plugin));
        }
    }

    app.run();
}
