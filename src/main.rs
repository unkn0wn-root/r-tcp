use log::{error, info};
use std::env;
use tcp_server::{
    config::ServerConfig,
    server::{RawServer, StdServer},
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = match ServerConfig::new() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Determine which server implementation to use
    let use_raw = env::var("USE_RAW_SERVER")
        .map(|v| v.parse().unwrap_or(false))
        .unwrap_or(false);

    info!("Starting TCP server...");

    if use_raw {
        let server = RawServer::new(config);
        if let Err(e) = server.run() {
            error!("Server error: {}", e);
            std::process::exit(1);
        }
    } else {
        let server = StdServer::new(config);
        if let Err(e) = server.run().await {
            error!("Server error: {}", e);
            std::process::exit(1);
        }
    }
}
