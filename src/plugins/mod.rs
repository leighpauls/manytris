use bevy::prelude::*;

// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use crate::cli_options::{ClientConfig, ClientType, ExecCommand};
use crate::plugins::states::PlayingState;

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
mod states;

pub fn run(cfg: ExecCommand) {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins).add_plugins((
        states::StatesPlugin {
            initial_state: PlayingState::Playing,
        },
        root::common_plugin,
        window_blocks::plugin,
        field_blocks::plugin,
        assets::plugin,
        system_sets::plugin,
        block_render::plugin,
        scoreboard::plugin,
        game_container::common_plugin,
        garbage_counter::plugin,
    ));

    match cfg {
        ExecCommand::Client(ClientConfig {
            server,
            client_type,
            bot_millis,
        }) => {
            app.insert_resource(net_client::NetClientConfig(server))
                .add_plugins((
                    game_container::multiplayer_client_plugin,
                    root::client_plugin,
                    net_client::plugin,
                    net_game_control_manager::client_plugin,
                ));
            match client_type {
                ClientType::Human => {
                    app.add_plugins(input::plugin);
                }
                ClientType::Bot => {
                    add_bot_input_plugin(&mut app, bot_millis);
                }
            };
        }
        ExecCommand::Server(hc) => {
            app.insert_resource(net_listener::NetListenerConfig(hc))
                .add_plugins((
                    game_container::server_plugin,
                    net_listener::plugin,
                    shape_producer::plugin,
                    net_game_control_manager::server_plugin,
                ));
        }
        ExecCommand::StandAlone => {
            app.add_plugins((
                game_container::stand_alone_plugin,
                input::plugin,
                root::stand_alone_plugin,
                shape_producer::plugin,
            ));
        }
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
