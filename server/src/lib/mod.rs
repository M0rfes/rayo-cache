mod reader;
mod writer;

use futures::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;

pub async fn handle_connection(stream: TcpStream)->Result<(),Box<dyn std::error::Error>> {
    let peer_addr = stream.peer_addr().unwrap();
    info!("Accepted connection from {}", peer_addr);

    // Wrap the TCP stream with a length-delimited codec.
    let framed = Framed::new(stream, LengthDelimitedCodec::new());
    // Split into writer (sink) and reader (stream) halves.
    let ( writer_sink, reader_stream) = framed.split();

    // Create an mpsc channel to pass serialized responses from the reader to the writer.
    let (tx, rx) = mpsc::channel::<common::message::Command>(32);
    let reader = reader::Reader::new(reader_stream, tx);
    let writer = writer::Writer::new(writer_sink, rx);
    let writer_handle = tokio::spawn(async move { writer.write().await.unwrap() });
    reader.run().await?;
    writer_handle.await?;
    info!("Connection with {} closed", peer_addr);
    Ok(())
}
