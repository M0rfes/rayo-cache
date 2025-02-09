use bytes::Bytes;
use common::message::{Command, Response};
use futures::{stream::SplitSink, SinkExt};
use thiserror::Error;
use tokio::{net::TcpStream, sync::mpsc::Receiver};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{error, info};

#[derive(Debug, Error)]
pub enum WriterError {
    #[error("Send Error")]
    SendError,
}

pub struct Writer {
    sink: SplitSink<Framed<TcpStream, LengthDelimitedCodec>, Bytes>,
    rx: Receiver<Response>,
}

impl Writer {
    pub fn new(
        sink: SplitSink<Framed<TcpStream, LengthDelimitedCodec>, Bytes>,
        rx: Receiver<Response>,
    ) -> Self {
        Self { sink, rx }
    }

    pub async fn write(mut self) -> Result<(), WriterError> {
        while let Some(msg_bytes) = self.rx.recv().await {
            match msg_bytes {
                Response::Pong => {
                    let msg = rmp_serde::to_vec(&Response::Pong).unwrap();
                    let bytes = Bytes::from(msg);
                    if let Err(e) = self.sink.send(bytes).await {
                        error!("Failed to send message: {}", e);
                        return Err(WriterError::SendError);
                    }
                }
                Response::Ok => {
                    let msg = rmp_serde::to_vec(&Response::Ok).unwrap();
                    let bytes = Bytes::from(msg);
                    if let Err(e) = self.sink.send(bytes).await {
                        error!("Failed to send message: {}", e);
                        return Err(WriterError::SendError);
                    }
                }
            };
        }
        Ok(())
    }
}
