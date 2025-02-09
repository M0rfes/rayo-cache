use common::message::{Command, Response};
use dashmap::DashMap;
use serde_json::Value;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct DataStore {
    kv: DashMap<String, Value>,
    tx: Sender<Response>,
    rx: Receiver<Command>,
}

impl DataStore {
    pub fn new(tx: Sender<Response>, rx: Receiver<Command>) -> Self {
        let kv = DashMap::new();
        Self { kv, tx, rx }
    }

    pub async fn run(mut self) {
        
    }
}
