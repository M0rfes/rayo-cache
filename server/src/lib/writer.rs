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

    pub async fn run(mut self) -> Result<(), WriterError> {
        while let Some(msg) = self.rx.recv().await {
            let msg = Response::to_vec(&msg).unwrap();
            let bytes = Bytes::from(msg);
            if let Err(e) = self.sink.send(bytes).await {
                error!("Failed to send message: {}", e);
                return Err(WriterError::SendError);
            }
        }
        Ok(())
    }
}
