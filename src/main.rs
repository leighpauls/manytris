use manytris_bevy::cli_options::GameArgs;
use manytris_bevy::plugins;

fn main() {
    let args = get_args();
    println!("Args: {:?}", args);

    plugins::run(args.exec_command);
}

#[cfg(not(target_arch = "wasm32"))]
fn get_args() -> GameArgs {
    use clap::Parser;
    GameArgs::parse()
}

#[cfg(target_arch = "wasm32")]
fn get_args() -> GameArgs {
    use manytris_bevy::cli_options;
    cli_options::web_client_args()
}
