
use clap::Parser;
use common::message::{Command, Response};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{error, info};
use std::error::Error;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use futures::{SinkExt, StreamExt};
use rmp_serde::{to_vec, from_slice};

/// A simple caching service client that connects to a server.
#[derive(Parser, Debug)]
#[clap(author, version, about = "A blazing fast caching client", long_about = None)]
struct ClientArgs {
    /// Hostname or IP address of the server.
    #[clap(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Port to connect on.
    #[clap(short, long, default_value = "6379")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for logging.
    tracing_subscriber::fmt::init();

    // Parse command-line arguments.
    let args = ClientArgs::parse();
    let addr = format!("{}:{}", args.host, args.port);

    // Connect to the server.
    let stream = TcpStream::connect(&addr).await?;
    info!("Successfully connected to {}", addr);
    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
     let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

       // Main input loop.
    while let Some(line) = lines.next_line().await? {
        let command_text = line.trim().to_lowercase();
        if command_text == "ping" {
            // Create the Command::Ping message.
            let ping_cmd = Command::PING;
            // Serialize the command into MessagePack bytes.
            let bytes = to_vec(&ping_cmd)?;
            // Send the serialized command over the connection.
            framed.send(bytes.into()).await?;
            println!("Sent: Ping");

            // Wait for the server's response.
            if let Some(frame) = framed.next().await {
                match frame {
                    Ok(resp_bytes) => {
                        // Deserialize the response.
                        let response: Response = from_slice(&resp_bytes)?;
                        println!("Server responded: {:?}", response);
                    }
                    Err(e) => {
                        eprintln!("Error reading response: {}", e);
                    }
                }
            }
        } else {
            println!("Unrecognized command. Please enter 'ping'.");
        }
    }
    Ok(())
}
