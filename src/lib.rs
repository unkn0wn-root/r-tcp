pub mod config;
pub mod error;
pub mod handler;
pub mod server;
pub mod utils;
pub mod client;
pub mod protocol;
pub mod storage;

use crate::utils::optimizations::SystemOptimizer;
use log::info;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        env_logger::init();

        if let Err(e) = SystemOptimizer::apply_optimizations() {
            log::warn!("Failed to apply some system optimizations: {}", e);
        }

        info!("Server initialized with optimized settings");
    });
}
