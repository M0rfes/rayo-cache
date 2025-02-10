use bytes::BytesMut;
use common::message::{Command, Response};
use futures::{stream::SplitStream, Stream, StreamExt};
use rmp_serde::from_slice;
use thiserror::Error;
use tokio::{
    net::TcpStream,
    sync::mpsc::{error::SendError, Sender},
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{error, info};

#[derive(Debug, Error)]
pub enum ReaderError {
    #[error("Parse error occurred")]
    ParseError,

    #[error("Read Error")]
    ReadError,

    #[error("send to data task error {0}")]
    SendToDataTaskError(SendError<Command>),

    #[error("invalid uri {0}")]
    InvalidURI(String),
}

pub struct Reader {
    stream: SplitStream<Framed<TcpStream, LengthDelimitedCodec>>,
    command_tx: Sender<Command>,
}

impl Reader {
    pub fn new(
        stream: SplitStream<Framed<TcpStream, LengthDelimitedCodec>>,
        command_tx: Sender<Command>,
    ) -> Self {
        Self { stream, command_tx }
    }

    pub async fn run(mut self) -> Result<(), ReaderError> {
        while let Some(msg) = self.stream.next().await {
            match msg {
                Ok(msg) => {
                    self.process_message(msg).await?;
                }
                Err(e) => {
                    eprintln!("Failed to read from socket: {}", e);
                    return Err(ReaderError::ReadError);
                }
            }
        }
        Ok(())
    }

    async fn process_message(&self, msg: BytesMut) -> Result<(), ReaderError> {
        let command =
            from_slice::<common::message::Command>(&msg).map_err(|_| ReaderError::ParseError)?;
        if let Err(e) = self.command_tx.send(command).await {
            error!("Error forwarding command: {}", e);
            return Err(ReaderError::SendToDataTaskError(e));
        }
        Ok(())
    }
}
