use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crate::error::{Result, ServerError};

// Message type identifiers
const MESSAGE_TYPE_REQUEST: u8 = 1;
const MESSAGE_TYPE_RESPONSE: u8 = 2;
const MESSAGE_TYPE_ERROR: u8 = 3;

// Operation codes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    Ping = 1,
    Store = 2,
    Retrieve = 3,
    Delete = 4,
    List = 5,
}

impl TryFrom<u8> for OpCode {
    type Error = ServerError;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(OpCode::Ping),
            2 => Ok(OpCode::Store),
            3 => Ok(OpCode::Retrieve),
            4 => Ok(OpCode::Delete),
            5 => Ok(OpCode::List),
            _ => Err(ServerError::Protocol(format!("Invalid opcode: {}", value))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub message_type: u8,
    pub request_id: u32,
    pub op_code: OpCode,
    pub payload_len: u32,
    pub payload: Vec<u8>,
}

impl Message {
    pub fn new_request(request_id: u32, op_code: OpCode, payload: Vec<u8>) -> Self {
        Self {
            message_type: MESSAGE_TYPE_REQUEST,
            request_id,
            op_code,
            payload_len: payload.len() as u32,
            payload,
        }
    }

    pub fn new_response(request_id: u32, payload: Vec<u8>) -> Self {
        Self {
            message_type: MESSAGE_TYPE_RESPONSE,
            request_id,
            op_code: OpCode::Ping, // Not relevant for responses
            payload_len: payload.len() as u32,
            payload,
        }
    }

    pub fn new_error(request_id: u32, error_message: String) -> Self {
        Self {
            message_type: MESSAGE_TYPE_ERROR,
            request_id,
            op_code: OpCode::Ping, // Not relevant for errors
            payload_len: error_message.len() as u32,
            payload: error_message.into_bytes(),
        }
    }

    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let message_type = reader.read_u8()?;
        let request_id = reader.read_u32::<BigEndian>()?;
        let op_code = reader.read_u8()?;
        let payload_len = reader.read_u32::<BigEndian>()?;

        let mut payload = vec![0u8; payload_len as usize];
        reader.read_exact(&mut payload)?;

        Ok(Self {
            message_type,
            request_id,
            op_code: OpCode::try_from(op_code)?,
            payload_len,
            payload,
        })
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u8(self.message_type)?;
        writer.write_u32::<BigEndian>(self.request_id)?;
        writer.write_u8(self.op_code as u8)?;
        writer.write_u32::<BigEndian>(self.payload_len)?;
        writer.write_all(&self.payload)?;
        writer.flush()?;
        Ok(())
    }

    pub fn is_request(&self) -> bool {
        self.message_type == MESSAGE_TYPE_REQUEST
    }

    pub fn is_response(&self) -> bool {
        self.message_type == MESSAGE_TYPE_RESPONSE
    }

    pub fn is_error(&self) -> bool {
        self.message_type == MESSAGE_TYPE_ERROR
    }
}
