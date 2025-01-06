use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GetAddressResponse {
    NoServer,
    Ready { host: String, port: u16 },
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
