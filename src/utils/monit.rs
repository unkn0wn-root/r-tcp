use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub struct ServerStats {
    start_time: Instant,
    total_connections: AtomicU64,
    total_bytes_read: AtomicU64,
    total_bytes_written: AtomicU64,
    active_connections: AtomicU64,
}

impl Default for ServerStats {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerStats {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            total_connections: AtomicU64::new(0),
            total_bytes_read: AtomicU64::new(0),
            total_bytes_written: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
        }
    }

    pub fn increment_connection(&self) {
        self.total_connections.fetch_add(1, Ordering::SeqCst);
        self.active_connections.fetch_add(1, Ordering::SeqCst);
    }

    pub fn decrement_connection(&self) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn add_bytes_read(&self, bytes: u64) {
        self.total_bytes_read.fetch_add(bytes, Ordering::SeqCst);
    }

    pub fn add_bytes_written(&self, bytes: u64) {
        self.total_bytes_written.fetch_add(bytes, Ordering::SeqCst);
    }

    pub fn get_stats(&self) -> ServerMetrics {
        ServerMetrics {
            uptime: self.start_time.elapsed(),
            total_connections: self.total_connections.load(Ordering::SeqCst),
            active_connections: self.active_connections.load(Ordering::SeqCst),
            total_bytes_read: self.total_bytes_read.load(Ordering::SeqCst),
            total_bytes_written: self.total_bytes_written.load(Ordering::SeqCst),
        }
    }
}

#[derive(Debug)]
pub struct ServerMetrics {
    pub uptime: Duration,
    pub total_connections: u64,
    pub active_connections: u64,
    pub total_bytes_read: u64,
    pub total_bytes_written: u64,
}

impl std::fmt::Display for ServerMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Server Metrics:\n\
             Uptime: {:?}\n\
             Total Connections: {}\n\
             Active Connections: {}\n\
             Total Bytes Read: {}\n\
             Total Bytes Written: {}\n",
            self.uptime,
            self.total_connections,
            self.active_connections,
            self.total_bytes_read,
            self.total_bytes_written
        )
    }
}
