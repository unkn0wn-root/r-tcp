use crate::{config::ServerConfig, error::Result};
use crate::handler::ConnectionHandler;

use log::{error, info};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct StdServer {
    config: ServerConfig,
    connection_limit: Arc<Semaphore>,
}

impl StdServer {
    pub fn new(config: ServerConfig) -> Self {
        let connection_limit = Arc::new(Semaphore::new(config.max_connections));
        Self {
            config,
            connection_limit,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let addr = SocketAddr::new(self.config.host, self.config.port);
        let listener = TcpListener::bind(addr).await?;

        info!("TCP server listening on {}", addr);

        loop {
            let permit = self.connection_limit.clone().acquire_owned().await.unwrap();
            let (socket, peer_addr) = match listener.accept().await {
                Ok((socket, peer_addr)) => (socket, peer_addr),
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            info!("Accepted connection from {}", peer_addr);

            let config = self.config.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::process_connection(socket, peer_addr, config).await {
                    error!("Error processing connection from {}: {}", peer_addr, e);
                }
                drop(permit);
            });
        }
    }

    async fn process_connection(
        socket: TcpStream,
        peer_addr: SocketAddr,
        config: ServerConfig,
    ) -> Result<()> {
        let std_stream = socket.into_std()?;

        std_stream.set_read_timeout(Some(std::time::Duration::from_millis(config.read_timeout_ms)))?;
        std_stream.set_write_timeout(Some(std::time::Duration::from_millis(config.write_timeout_ms)))?;

        let mut handler = ConnectionHandler::new(std_stream, peer_addr, config.buffer_size);
        handler.handle().await?;

        Ok(())
    }
}
