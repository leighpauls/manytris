use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GetAddressResponse {
    NoServer,
    Ready { host: String, host_port: u16, container_port: u16, host_stats_port: u16, container_stats_port: u16 },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CreateResponse {
    AlreadyExists,
    Created,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeleteResponse {
    NotFound,
    Deleting,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HeartbeatResponse {
    NotFound,
    Live,
    GracePeriod,
    Deleted,
}

pub const STATS_SERVER_PORT: u16 = 9990;
pub const STATS_SERVER_ROUTE: &str = "/stats";

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsServerResponse {
    pub num_connected_players: u16,
    pub num_active_games: u16,
    pub connectionless_time_secs: u32,
}

