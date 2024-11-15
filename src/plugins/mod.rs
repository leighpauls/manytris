use bevy::prelude::*;

// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use crate::cli_options::{BotConfig, ExecCommand};

mod assets;
mod block_render;
mod field_blocks;
mod game_container;
mod garbage_counter;
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

#[cfg(target_os = "macos")]
mod bot_input;
mod main_menu;
pub mod states;

pub fn run(cfg: ExecCommand) {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, spawn_camera)
        .add_plugins((
            cfg.configure_states_plugin(),
            main_menu::plugin,
            root::common_plugin,
            window_blocks::plugin,
            field_blocks::plugin,
            assets::plugin,
            system_sets::plugin,
            block_render::plugin,
            scoreboard::plugin,
            game_container::common_plugin,
            garbage_counter::plugin,
            net_client::plugin,
            input::plugin,
            net_listener::plugin,
            shape_producer::plugin,
        ));

    match &cfg {
        ExecCommand::Server(hc) => {
            app.insert_resource(net_listener::NetListenerConfig(hc.clone()));
        }
        ExecCommand::Client(server) | ExecCommand::Bot(BotConfig { server, .. }) => {
            app.insert_resource(net_client::NetClientConfig(server.clone()));
        }
    }

    if let ExecCommand::Bot(BotConfig { bot_millis, .. }) = &cfg {
        add_bot_input_plugin(&mut app, *bot_millis);
    }

    app.run();
}

#[cfg(target_os = "macos")]
fn add_bot_input_plugin(app: &mut App, bot_millis: u64) {
    app.add_plugins(bot_input::BotInputPlugin {
        bot_period_millis: bot_millis,
    });
}

#[cfg(not(target_os = "macos"))]
fn add_bot_input_plugin(_app: &mut App, _bot_millis: u64) {}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
