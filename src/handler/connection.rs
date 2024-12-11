use crate::error::Result;
use log::{debug, error};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct ConnectionHandler {
    stream: TcpStream,
    peer_addr: SocketAddr,
    buffer_size: usize,
}

impl ConnectionHandler {
    pub fn new(stream: TcpStream, peer_addr: SocketAddr, buffer_size: usize) -> Self {
        Self {
            stream,
            peer_addr,
            buffer_size,
        }
    }

    pub async fn handle(&mut self) -> Result<()> {
        let mut buf = vec![0; self.buffer_size];

        // Convert to async stream
        let mut stream = tokio::net::TcpStream::from_std(self.stream.try_clone()?)?;

        loop {
            let n = match stream.read(&mut buf).await {
                Ok(0) => {
                    debug!("Connection closed by peer: {}", self.peer_addr);
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    error!("Error reading from connection {}: {}", self.peer_addr, e);
                    return Err(e.into());
                }
            };

            // @toDo - Echo the data back (replace this with actual protocol handling)
            if let Err(e) = stream.write_all(&buf[..n]).await {
                error!("Error writing to connection {}: {}", self.peer_addr, e);
                return Err(e.into());
            }
        }

        Ok(())
    }

    pub fn handle_blocking(&mut self) -> Result<()> {
        let mut buffer = vec![0; self.buffer_size];

        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    debug!("Connection closed by peer: {}", self.peer_addr);
                    break;
                }
                Ok(n) => {
                    if let Err(e) = self.stream.write_all(&buffer[..n]) {
                        debug!("Error writing to connection {}: {}", self.peer_addr, e);
                        break;
                    }
                }
                Err(e) => {
                    debug!("Error reading from connection {}: {}", self.peer_addr, e);
                    break;
                }
            }
        }
        Ok(())
    }
}
