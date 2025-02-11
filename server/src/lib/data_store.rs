use std::{collections, f32::consts::E, hash::DefaultHasher};

use common::message::{Command, Response};
use dashmap::{DashMap, Entry};
use serde_json::{json, Value};
use std::hash::{Hash, Hasher};
use thiserror::Error;
use tokio::sync::mpsc::{error::SendError, Receiver, Sender};
use tracing::{error, info};
use ulid::Ulid;

pub struct DataStore {
    kv: DashMap<String, DashMap<Ulid, Value>>,
    tx: Sender<Response>,
    rx: Receiver<Command>,
}

#[derive(Debug, Error)]
pub enum DSError {
    #[error("Parse error occurred")]
    ParseError,

    #[error("Read Error")]
    ReadError,

    #[error("send to data task error {0}")]
    SendError(SendError<Response>),
}

impl DataStore {
    pub fn new(tx: Sender<Response>, rx: Receiver<Command>) -> Self {
        let kv = DashMap::new();
        Self { kv, tx, rx }
    }

    pub async fn run(mut self) -> Result<(), DSError> {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                Command::PING => {
                    self.send_response(Response::PONG).await;
                }
                Command::POST { uri, body } => {
                    let (name, _) = uri.split_once('/').unwrap_or((uri.as_str(), ""));
                    let id = Ulid::new();
                    if let Some(ds) = self.kv.get_mut(name) {
                        ds.insert(id, body);
                    } else {
                        let collection = DashMap::from_iter(vec![(id.clone(), body)]);
                        self.kv.insert(name.to_string(), collection);
                    }

                    self.send_response(Response::ID(id.to_string())).await;
                }
                Command::GET { uri } => {
                    let (name, id) = uri.split_once('/').unwrap_or((uri.as_str(), ""));
                    if id.is_empty() {
                        let collection = self.kv.get(name);
                        if collection.is_none() {
                            self.send_response(Response::ERROR("collection not found".to_string()))
                                .await;
                        } else {
                            let collection = collection.unwrap();
                            let res = Response::COLLECTION(
                                collection
                                    .value()
                                    .iter()
                                    .map(|r| {
                                        json!({
                                            "ID": r.key().to_string(),
                                            "value": r.value().clone()
                                        })
                                    })
                                    .collect::<Vec<_>>(),
                            );
                            self.send_response(res).await;
                        }
                    } else {
                        let Some(collection) = self.kv.get(name) else {
                            self.send_response(Response::ERROR("collection not found".to_string()))
                                .await;
                            continue;
                        };
                        let Ok(ulid) = Ulid::from_string(id) else {
                            self.send_response(Response::ERROR("invalid ID".to_string()))
                                .await;
                            continue;
                        };
                        let Some(obj) = collection.get(&ulid) else {
                            self.send_response(Response::ERROR("object not found".to_string()))
                                .await;
                            continue;
                        };

                        self.send_response(Response::OBJECT(json!({
                            "ID": obj.key().to_string(),
                            "value": obj.clone()
                        })))
                        .await;
                    }
                }
                Command::PUT { ref uri, ref body } => {
                    let Some((name, id)) = uri.split_once("/") else {
                        self.send_response(Response::ERROR("invalid path".to_string()))
                            .await;
                        continue;
                    };
                    let Some(collection) = self.kv.get_mut(name) else {
                        self.send_response(Response::ERROR("collection not found".to_string()))
                            .await;
                        continue;
                    };
                    let Ok(id) = Ulid::from_string(id) else {
                        self.send_response(Response::ERROR("invalid id".to_string()))
                            .await;
                        continue;
                    };
                    let Entry::Occupied(_) = collection.entry(id).and_modify(|v| *v = body.clone())
                    else {
                        self.send_response(Response::ERROR("object not found".to_string()))
                            .await;
                        continue;
                    };
                    self.send_response(Response::OK).await;
                }
                Command::DUMP { file } => {}
                _ => todo!(),
            }
        }
        Ok(())
    }

    async fn send_response(&self, response: Response) {
        if let Err(e) = self.tx.send(response).await {
            error!("Error forwarding {}", e);
        }
    }
}
