use nix::sys::resource::{setrlimit, Resource};
use std::io::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use log;

pub struct SystemOptimizer;

// Global buffer size configuration
const MAX_BUFFER_SIZE: usize = 16_777_216; // 16MB
static GLOBAL_BUFFER_SIZE: AtomicUsize = AtomicUsize::new(4096);

impl SystemOptimizer {
    pub fn apply_optimizations() -> Result<()> {
        Self::optimize_file_limits()?;
        Self::optimize_tcp_settings()?;
        Self::optimize_memory_settings()?;
        Ok(())
    }

    fn optimize_file_limits() -> Result<()> {
        // increase system file descriptor limits
        match setrlimit(Resource::RLIMIT_NOFILE, 65535, 65535) {
            Ok(_) => log::info!("Successfully increased file descriptor limits"),
            Err(e) => log::warn!("Failed to set file descriptor limits: {}", e),
        }
        Ok(())
    }

    fn optimize_tcp_settings() -> Result<()> {
        use std::process::Command;

        let settings = [
            // enable TCP fast open
            "net.ipv4.tcp_fastopen=3",
            // increase TCP max syn backlog
            "net.ipv4.tcp_max_syn_backlog=4096",
            // enable TCP window scaling
            "net.ipv4.tcp_window_scaling=1",
            // increase maximum TCP receive buffer size
            "net.core.rmem_max=16777216",
            // increase maximum TCP send buffer size
            "net.core.wmem_max=16777216",
            // increase number of outstanding syn requests
            "net.ipv4.tcp_max_syn_backlog=4096",
        ];

        if cfg!(target_os = "linux") {
            for setting in settings.iter() {
                match Command::new("sysctl")
                    .arg("-w")
                    .arg(setting)
                    .output()
                {
                    Ok(output) if output.status.success() => {
                        log::info!("Successfully applied TCP setting: {}", setting);
                    }
                    Ok(_) => {
                        log::warn!("Failed to apply TCP setting: {}", setting);
                    }
                    Err(e) => {
                        log::warn!("Error applying TCP setting {}: {}", setting, e);
                    }
                }
            }
        } else {
            log::info!("Skipping TCP optimizations on non-Linux platform");
        }

        Ok(())
    }

    fn optimize_memory_settings() -> Result<()> {
        // optimal buffer sizes based on available system memory
        match sys_info::mem_info() {
            Ok(memory_info) => {
                let total_mem = memory_info.total;
                let buffer_size = (total_mem / 1024 / 10) as usize; // Use 10% of available memory
                let new_size = buffer_size.min(MAX_BUFFER_SIZE);

                // update global buffer size configuration using atomic operation
                GLOBAL_BUFFER_SIZE.store(new_size, Ordering::SeqCst);
                log::info!("Set buffer size to {} bytes", GLOBAL_BUFFER_SIZE.load(Ordering::SeqCst));
            }
            Err(e) => {
                log::warn!("Failed to get memory info: {}", e);
            }
        }
        Ok(())
    }
}

// thread-local buffer pool
thread_local! {
    static BUFFER_POOL: std::cell::RefCell<Vec<Vec<u8>>> = std::cell::RefCell::new(Vec::new());
}

pub fn get_buffer() -> Vec<u8> {
    let size = GLOBAL_BUFFER_SIZE.load(Ordering::SeqCst);
    BUFFER_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        pool.pop().unwrap_or_else(|| {
            Vec::with_capacity(size)
        })
    })
}

pub fn return_buffer(mut buf: Vec<u8>) {
    buf.clear();
    BUFFER_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        if pool.len() < 32 { // Limit pool size
            pool.push(buf);
        }
    });
}
