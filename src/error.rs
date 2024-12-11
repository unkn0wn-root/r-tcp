use std::io;
use thiserror::Error;
use nix;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("System error: {0}")]
    System(#[from] nix::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Accept error: {0}")]
    Accept(String),

    #[error("Client error: {0}")]
    Client(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Storage error: {0}")]
    Storage(String),
}

impl From<Box<bincode::ErrorKind>> for ServerError {
    fn from(error: Box<bincode::ErrorKind>) -> Self {
        ServerError::Serialization(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ServerError>;
