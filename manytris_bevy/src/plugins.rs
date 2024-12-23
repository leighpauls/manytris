use crate::cli_options::{BotConfig, ClientConfig, ExecCommand, ServerConfig};
use crate::{
    assets, block_render, connecting_screen, field_blocks, game_container, garbage_counter, input,
    main_menu, net_client, net_listener, root, scoreboard, shape_producer, system_sets,
    window_blocks,
};
use bevy::prelude::*;

pub fn run(cfg: ExecCommand) {
    let mut app = App::new();

    let headless = matches!(
        cfg,
        ExecCommand::Server(ServerConfig { headless: true, .. })
    );

    if headless {
        app.add_plugins(MinimalPlugins);
    } else {
        app.add_plugins((DefaultPlugins, assets::plugin))
            .add_systems(Startup, spawn_camera);
    }

    app.add_plugins((
        cfg.configure_states_plugin(),
        main_menu::plugin,
        connecting_screen::plugin,
        root::common_plugin,
        window_blocks::plugin,
        field_blocks::plugin,
        system_sets::plugin,
        block_render::plugin,
        scoreboard::plugin,
        game_container::plugin,
        garbage_counter::plugin,
        net_client::plugin,
        input::plugin,
        net_listener::plugin,
        shape_producer::plugin,
    ));

    match &cfg {
        ExecCommand::Server(ServerConfig { server, .. }) => {
            app.insert_resource(net_listener::NetListenerConfig(server.clone()));
        }
        ExecCommand::Client(ClientConfig { server, .. })
        | ExecCommand::Bot(BotConfig { server, .. }) => {
            app.insert_resource(net_client::NetClientConfig(server.clone()));
        }
    }

    if let ExecCommand::Bot(BotConfig { bot_millis, .. }) = &cfg {
        add_bot_input_plugin(&mut app, *bot_millis);
    }

    app.run();
}

#[cfg(feature = "bot")]
fn add_bot_input_plugin(app: &mut App, bot_millis: u64) {
    use crate::bot_input;

    app.add_plugins(bot_input::BotInputPlugin {
        bot_period_millis: bot_millis,
    });
}

#[cfg(not(feature = "bot"))]
fn add_bot_input_plugin(_app: &mut App, _bot_millis: u64) {}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
