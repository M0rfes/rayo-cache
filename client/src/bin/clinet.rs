use clap::Parser;
use tokio::net::TcpStream;
use tracing::{error, info};

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
    match TcpStream::connect(&addr).await {
        Ok(_stream) => info!("Successfully connected to {}", addr),
        Err(e) => error!("Failed to connect to {}: {}", addr, e),
    }
    Ok(())
}
