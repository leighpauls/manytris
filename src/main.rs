use clap::Parser;

use manytris::cli_options::GameArgs;
use manytris::plugins;

fn main() {
    let args = GameArgs::parse();
    println!("Args: {:?}", args);

    plugins::run(args.exec_command);
}
