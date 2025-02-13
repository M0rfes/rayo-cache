mod reader;
mod writer;
mod data_store;

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
    let (tx, rx) = mpsc::channel::<common::message::Response>(32);
    let (command_tx, command_rx) = mpsc::channel::<common::message::Command>(32);
    let reader = reader::Reader::new(reader_stream,command_tx);
    let writer = writer::Writer::new(writer_sink, rx);
    let data_store = data_store::DataStore::new(tx, command_rx);

    let writer_handle = tokio::spawn(async move { writer.run().await.unwrap() });
    let data_handler = tokio::spawn(async move {data_store.run().await.unwrap()});
    reader.run().await?;
    writer_handle.await?;
    data_handler.await?;
    info!("Connection with {} closed", peer_addr);
    Ok(())
}
