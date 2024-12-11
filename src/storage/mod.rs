use dashmap::DashMap;
use crate::error::{Result, ServerError};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct KeyValueStore {
    data: DashMap<String, Vec<u8>>,
    size: AtomicU64,
    max_size: u64,
}

impl KeyValueStore {
    pub fn new(max_size: u64) -> Self {
        Self {
            data: DashMap::new(),
            size: AtomicU64::new(0),
            max_size,
        }
    }

    pub fn set(&self, key: &str, value: Vec<u8>) -> Result<()> {
        let value_size = value.len() as u64;

        // check if we're updating an existing key
        if let Some(existing) = self.data.get(key) {
            let existing_size = existing.len() as u64;
            let size_diff = value_size as i64 - existing_size as i64;

            if size_diff > 0 &&
               self.size.load(Ordering::Relaxed) + size_diff as u64 > self.max_size {
                return Err(ServerError::Storage("Storage capacity exceeded".into()));
            }

            self.size.fetch_add(size_diff as u64, Ordering::Relaxed);
        } else {
            // new key
            if self.size.load(Ordering::Relaxed) + value_size > self.max_size {
                return Err(ServerError::Storage("Storage capacity exceeded".into()));
            }

            self.size.fetch_add(value_size, Ordering::Relaxed);
        }

        self.data.insert(key.to_string(), value);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.data.get(key).map(|v| v.clone()))
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        if let Some((_, value)) = self.data.remove(key) {
            self.size.fetch_sub(value.len() as u64, Ordering::Relaxed);
        }
        Ok(())
    }

    pub fn list_keys(&self) -> Result<Vec<String>> {
        Ok(self.data.iter().map(|r| r.key().clone()).collect())
    }

    pub fn current_size(&self) -> u64 {
        self.size.load(Ordering::Relaxed)
    }

    pub fn entry_count(&self) -> usize {
        self.data.len()
    }
}
