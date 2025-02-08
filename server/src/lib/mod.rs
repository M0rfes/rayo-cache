
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use futures::StreamExt;
use tracing::info;
// use rayo_cache_common::message::{Command, Response};

pub async fn handle_connection(stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap();
    info!("Accepted connection from {}", peer_addr);

    // Wrap the TCP stream with a length-delimited codec.
    let framed = Framed::new(stream, LengthDelimitedCodec::new());
    // Split into writer (sink) and reader (stream) halves.
    let (mut writer_sink, mut reader_stream) = framed.split();

    // Create an mpsc channel to pass serialized responses from the reader to the writer.
    let (tx, mut rx) = mpsc::channel::<common::message::Command>(32);
}