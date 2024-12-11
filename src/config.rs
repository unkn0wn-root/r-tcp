use serde::Deserialize;
use std::net::IpAddr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Environment error: {0}")]
    EnvError(#[from] dotenv::Error),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
    pub backlog: i32,
    pub max_connections: usize,
    pub read_timeout_ms: u64,
    pub write_timeout_ms: u64,
    pub buffer_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".parse().unwrap(),
            port: 8080,
            backlog: 128,
            max_connections: 1000,
            read_timeout_ms: 5000,
            write_timeout_ms: 5000,
            buffer_size: 4096,
        }
    }
}

impl ServerConfig {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv::dotenv().ok();

        let config = Self {
            host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string())
                .parse()
                .map_err(|e| ConfigError::ConfigError(format!("Invalid host: {}", e)))?,
            port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|e| ConfigError::ConfigError(format!("Invalid port: {}", e)))?,
            backlog: std::env::var("SERVER_BACKLOG")
                .unwrap_or_else(|_| "128".to_string())
                .parse()
                .map_err(|e| ConfigError::ConfigError(format!("Invalid backlog: {}", e)))?,
            max_connections: std::env::var("SERVER_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .map_err(|e| ConfigError::ConfigError(format!("Invalid max connections: {}", e)))?,
            read_timeout_ms: std::env::var("SERVER_READ_TIMEOUT_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .map_err(|e| ConfigError::ConfigError(format!("Invalid read timeout: {}", e)))?,
            write_timeout_ms: std::env::var("SERVER_WRITE_TIMEOUT_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .map_err(|e| ConfigError::ConfigError(format!("Invalid write timeout: {}", e)))?,
            buffer_size: std::env::var("SERVER_BUFFER_SIZE")
                .unwrap_or_else(|_| "4096".to_string())
                .parse()
                .map_err(|e| ConfigError::ConfigError(format!("Invalid buffer size: {}", e)))?,
        };

        Ok(config)
    }
}
