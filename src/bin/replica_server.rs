use manytris::plugins;
use manytris::plugins::GameConfig;

fn main() {
    plugins::run(GameConfig::ReplicaServer);
}
