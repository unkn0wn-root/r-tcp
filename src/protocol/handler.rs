use super::message::{Message, OpCode};
use crate::error::Result;
use crate::storage::KeyValueStore;
use log::debug;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct StoreRequest {
    key: String,
    value: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RetrieveRequest {
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeleteRequest {
    key: String,
}

pub struct ProtocolHandler {
    store: Arc<KeyValueStore>,
}

impl ProtocolHandler {
    pub fn new(store: Arc<KeyValueStore>) -> Self {
        Self { store }
    }

    pub fn handle_message(&self, message: Message) -> Result<Message> {
        if !message.is_request() {
            return Ok(Message::new_error(
                message.request_id,
                "Invalid message type".to_string(),
            ));
        }

        match message.op_code {
            OpCode::Ping => self.handle_ping(message),
            OpCode::Store => self.handle_store(message),
            OpCode::Retrieve => self.handle_retrieve(message),
            OpCode::Delete => self.handle_delete(message),
            OpCode::List => self.handle_list(message),
        }
    }

    fn handle_ping(&self, message: Message) -> Result<Message> {
        debug!("Handling PING request");
        Ok(Message::new_response(message.request_id, b"PONG".to_vec()))
    }

    fn handle_store(&self, message: Message) -> Result<Message> {
        debug!("Handling STORE request");
        let request: StoreRequest = bincode::deserialize(&message.payload)?;

        self.store.set(&request.key, request.value)?;

        Ok(Message::new_response(
            message.request_id,
            b"OK".to_vec(),
        ))
    }

    fn handle_retrieve(&self, message: Message) -> Result<Message> {
        debug!("Handling RETRIEVE request");
        let request: RetrieveRequest = bincode::deserialize(&message.payload)?;

        match self.store.get(&request.key)? {
            Some(value) => Ok(Message::new_response(message.request_id, value)),
            None => Ok(Message::new_error(
                message.request_id,
                "Key not found".to_string(),
            )),
        }
    }

    fn handle_delete(&self, message: Message) -> Result<Message> {
        debug!("Handling DELETE request");
        let request: DeleteRequest = bincode::deserialize(&message.payload)?;

        self.store.delete(&request.key)?;

        Ok(Message::new_response(
            message.request_id,
            b"OK".to_vec(),
        ))
    }

    fn handle_list(&self, message: Message) -> Result<Message> {
        debug!("Handling LIST request");
        let keys = self.store.list_keys()?;
        let response = bincode::serialize(&keys)?;

        Ok(Message::new_response(message.request_id, response))
    }
}
