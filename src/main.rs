use clap::Parser;

use manytris::cli_options;
use manytris::cli_options::{GameArgs};
use manytris::plugins;

fn main() {
    let args = get_args();
    println!("Args: {:?}", args);

    plugins::run(args.exec_command);
}

#[cfg(not(target_arch = "wasm32"))]
fn get_args() -> GameArgs {
    GameArgs::parse()
}

#[cfg(target_arch = "wasm32")]
fn get_args() -> GameArgs {
    cli_options::web_client_args()
}