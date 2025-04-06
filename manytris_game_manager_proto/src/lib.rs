use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GetAddressResponse {
    NoServer,
    Ready { host: String, host_port: u16, container_port: u16 },
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
pub struct StatsServerResponse {
    pub num_connected_players: u16,
    pub num_active_games: u16,
}

