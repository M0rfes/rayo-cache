use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Pong,
}