use clap::Parser;
use cli_options::Args;
use manytris::{cli_options, plugins};
use manytris::plugins::{GameConfig, HostConfig};

fn main() {
    let args = Args::parse();
    println!("Args: {:?}", args);

    let hc = HostConfig{host: args.host, port: args.port};
    if args.is_server {
        plugins::run(GameConfig::ReplicaServer(hc));
    } else {
        plugins::run(GameConfig::Client(hc));
    }
}