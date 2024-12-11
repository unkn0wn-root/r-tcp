use crate::error::{Result, ServerError};
use crate::protocol::message::{Message, OpCode};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU32, Ordering};
use serde::Serialize;

pub struct Client {
    stream: TcpStream,
    request_id: AtomicU32,
}

impl Client {
    pub fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;
        Ok(Self {
            stream,
            request_id: AtomicU32::new(1),
        })
    }

    pub fn ping(&mut self) -> Result<String> {
        let request_id = self.next_request_id();
        let message = Message::new_request(request_id, OpCode::Ping, Vec::new());
        self.send_and_receive(message)
            .map(|response| String::from_utf8_lossy(&response.payload).to_string())
    }

    pub fn store<T: Serialize>(&mut self, key: &str, value: T) -> Result<()> {
        let request_id = self.next_request_id();
        let payload = bincode::serialize(&(key, value)).map_err(|e| ServerError::Serialization(e.to_string()))?;
        let message = Message::new_request(request_id, OpCode::Store, payload);
        let response = self.send_and_receive(message)?;
        if response.is_error() {
            Err(ServerError::Client(
                String::from_utf8_lossy(&response.payload).to_string()
            ))
        } else {
            Ok(())
        }
    }

    pub fn retrieve(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        let request_id = self.next_request_id();
        let payload = bincode::serialize(&key).map_err(|e| ServerError::Serialization(e.to_string()))?;
        let message = Message::new_request(request_id, OpCode::Retrieve, payload);
        let response = self.send_and_receive(message)?;
        if response.is_error() {
            if String::from_utf8_lossy(&response.payload).contains("Key not found") {
                Ok(None)
            } else {
                Err(ServerError::Client(
                    String::from_utf8_lossy(&response.payload).to_string()
                ))
            }
        } else {
            Ok(Some(response.payload))
        }
    }

    pub fn delete(&mut self, key: &str) -> Result<()> {
        let request_id = self.next_request_id();
        let payload = bincode::serialize(&key).map_err(|e| ServerError::Serialization(e.to_string()))?;
        let message = Message::new_request(request_id, OpCode::Delete, payload);
        let response = self.send_and_receive(message)?;
        if response.is_error() {
            Err(ServerError::Client(
                String::from_utf8_lossy(&response.payload).to_string()
            ))
        } else {
            Ok(())
        }
    }

    pub fn list(&mut self) -> Result<Vec<String>> {
        let request_id = self.next_request_id();
        let message = Message::new_request(request_id, OpCode::List, Vec::new());
        let response = self.send_and_receive(message)?;
        if response.is_error() {
            Err(ServerError::Client(
                String::from_utf8_lossy(&response.payload).to_string()
            ))
        } else {
            bincode::deserialize(&response.payload)
                .map_err(|e| ServerError::Serialization(e.to_string()))
        }
    }

    fn send_and_receive(&mut self, message: Message) -> Result<Message> {
        message.write_to(&mut self.stream)?;
        Message::read_from(&mut self.stream)
    }

    fn next_request_id(&self) -> u32 {
        self.request_id.fetch_add(1, Ordering::Relaxed)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}
