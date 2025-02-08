use bytes::BytesMut;
use common::message::Command;
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

    #[error("Send Error")]
    SendError(SendError<Command>),
}

pub struct Reader {
    stream: SplitStream<Framed<TcpStream, LengthDelimitedCodec>>,
    tx: Sender<common::message::Command>,
}

impl Reader {
    pub fn new(
        stream: SplitStream<Framed<TcpStream, LengthDelimitedCodec>>,
        tx: Sender<common::message::Command>,
    ) -> Self {
        Self { stream, tx }
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
        match from_slice::<common::message::Command>(&msg) {
            Ok(common::message::Command::Ping) => {
                if let Err(e) = self.tx.send(common::message::Command::Ping).await {
                    error!("Error forwarding Pong to writer: {}", e);
                    return Err(ReaderError::SendError(e));
                }
            }
            Err(e) => {
                eprintln!("Failed to parse message: {}", e);
                return Err(ReaderError::ParseError);
            }
        };
        Ok(())
    }
}
