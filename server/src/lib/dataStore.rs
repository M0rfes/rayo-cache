use std::{collections, hash::DefaultHasher};

use common::message::{Command, Response};
use dashmap::DashMap;
use futures::future::ok;
use rmp_serde::config;
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
                    if let Err(e) = self.tx.send(Response::PONG).await {
                        error!("Error forwarding Pong to writer: {}", e);
                        return Err(DSError::SendError(e));
                    }
                }
                Command::POST { uri, body } => {
                    println!("processing post");
                    let (name, _) = uri.split_once('/').unwrap_or((uri.as_str(), ""));
                    // let Some(obj) = body.as_object_mut() else {
                    //     return Err(DSError::ParseError);
                    // };
                    let id = Ulid::new();
                    // obj.insert("_id".to_string(), json!(id.to_string()));
                    // let obj = json!(obj);
                    if let Some(ds) = self.kv.get_mut(name) {
                        ds.insert(id, body);
                    } else {
                        let collection = DashMap::from_iter(vec![(id.clone(), body)]);
                        self.kv.insert(name.to_string(), collection);
                    }
                    println!("processed post");

                    if let Err(e) = self.tx.send(Response::ID(id.to_string())).await {
                        error!("Error forwarding Pong to writer: {}", e);
                        return Err(DSError::SendError(e));
                    }
                }
                Command::GET { uri } => {
                    let (name, id) = uri.split_once('/').unwrap_or((uri.as_str(), ""));
                    if id.is_empty() {
                        let collection = self.kv.get(name);
                        if collection.is_none() {
                            if let Err(e) = self.tx.send(Response::NIL).await {
                                error!("Error forwarding Pong to writer: {}", e);
                                return Err(DSError::SendError(e));
                            }
                        } else {
                            let collection = collection.unwrap();
                            let res = Response::COLLECTION(
                                collection
                                    .value()
                                    .iter()
                                    .map(|r| r.value().clone())
                                    .collect::<Vec<_>>(),
                            );
                            if let Err(e) = self.tx.send(res).await {
                                error!("Error forwarding Pong to writer: {}", e);
                                return Err(DSError::SendError(e));
                            }
                        }
                    } else {
                        let Some(collection) = self.kv.get(name) else {
                            if let Err(e) = self.tx.send(Response::NIL).await {
                                error!("Error forwarding Pong to writer: {}", e);
                                return Err(DSError::SendError(e));
                            }
                            continue;
                        };
                        let Ok(ulid) = Ulid::from_string(id) else {
                            if let Err(e) = self.tx.send(Response::NIL).await {
                                error!("Error forwarding Pong to writer: {}", e);
                                return Err(DSError::SendError(e));
                            }
                            continue;
                        };
                        let Some(obj) = collection.get(&ulid) else {
                            if let Err(e) = self.tx.send(Response::NIL).await {
                                error!("Error forwarding Pong to writer: {}", e);
                                return Err(DSError::SendError(e));
                            }
                            continue;
                        };
                        let o = obj.value();
                        if let Err(e) = self.tx.send(Response::OBJECT(o.clone())).await {
                            error!("Error forwarding Pong to writer: {}", e);
                            return Err(DSError::SendError(e));
                        }
                    }
                }
                _ => todo!(),
            }
        }
        Ok(())
    }
}
