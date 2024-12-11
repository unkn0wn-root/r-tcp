use crate::error::Result;
use crate::protocol::message::Message;
use crate::protocol::handler::ProtocolHandler;
use crate::storage::KeyValueStore;
use log::{debug, error};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct ProtocolConnectionHandler {
    stream: TcpStream,
    peer_addr: SocketAddr,
    handler: ProtocolHandler,
    buffer_size: usize,
}

impl ProtocolConnectionHandler {
    pub fn new(
        stream: TcpStream,
        peer_addr: SocketAddr,
        store: Arc<KeyValueStore>,
        buffer_size: usize,
    ) -> Self {
        Self {
            stream,
            peer_addr,
            handler: ProtocolHandler::new(store),
            buffer_size,
        }
    }

    pub async fn handle(&mut self) -> Result<()> {
        let mut stream = tokio::net::TcpStream::from_std(self.stream.try_clone()?)?;

        loop {
            match Message::read_from(&mut stream) {
                Ok(message) => {
                    debug!("Received message from {}: {:?}", self.peer_addr, message);

                    match self.handler.handle_message(message) {
                        Ok(response) => {
                            debug!("Sending response to {}: {:?}", self.peer_addr, response);
                            if let Err(e) = response.write_to(&mut stream) {
                                error!("Error sending response to {}: {}", self.peer_addr, e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Error handling message from {}: {}", self.peer_addr, e);
                            let error_message = Message::new_error(0, e.to_string());
                            if let Err(e) = error_message.write_to(&mut stream) {
                                error!("Error sending error response to {}: {}", self.peer_addr, e);
                            }
                            break;
                        }
                    }
                }
                Err(e) => {
                    if e.to_string().contains("connection reset") ||
                       e.to_string().contains("broken pipe") {
                        debug!("Connection closed by peer: {}", self.peer_addr);
                    } else {
                        error!("Error reading from {}: {}", self.peer_addr, e);
                    }
                    break;
                }
            }
        }

        Ok(())
    }
}
