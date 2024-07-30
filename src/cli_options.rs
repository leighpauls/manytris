
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {

    #[arg(long="server")]
    pub is_server: bool,

    #[arg(long, default_value = "localhost")]
    pub host: String,

    #[arg(long, default_value_t=9989)]
    pub port: u16,
}
