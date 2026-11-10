//! Cache management for Penoos services

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde_json::Value;

use crate::errors::Result;

/// Cache manager for storing and retrieving cached data
pub struct CacheManager {
    backend: Box<dyn CacheBackend + Send + Sync>,
}

/// Cache backend trait
pub trait CacheBackend {
    fn get(&self, key: &str) -> Result<Option<Value>>;
    fn set(&self, key: &str, value: Value, ttl: Option<Duration>) -> Result<()>;
    fn del(&self, key: &str) -> Result<bool>;
    fn clear(&self) -> Result<()>;
    fn keys(&self, pattern: Option<&str>) -> Result<Vec<String>>;
}

/// In-memory cache backend
pub struct MemoryCache {
    data: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

struct CacheEntry {
    value: Value,
    expires_at: Option<Instant>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut data = self.data.write();
        data.retain(|_, entry| {
            if let Some(expires_at) = entry.expires_at {
                expires_at > now
            } else {
                true
            }
        });
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheBackend for MemoryCache {
    fn get(&self, key: &str) -> Result<Option<Value>> {
        self.cleanup_expired();
        let data = self.data.read();
        if let Some(entry) = data.get(key) {
            if let Some(expires_at) = entry.expires_at {
                if expires_at <= Instant::now() {
                    return Ok(None);
                }
            }
            Ok(Some(entry.value.clone()))
        } else {
            Ok(None)
        }
    }

    fn set(&self, key: &str, value: Value, ttl: Option<Duration>) -> Result<()> {
        let expires_at = ttl.map(|duration| Instant::now() + duration);
        let entry = CacheEntry { value, expires_at };
        
        let mut data = self.data.write();
        data.insert(key.to_string(), entry);
        Ok(())
    }

    fn del(&self, key: &str) -> Result<bool> {
        let mut data = self.data.write();
        Ok(data.remove(key).is_some())
    }

    fn clear(&self) -> Result<()> {
        let mut data = self.data.write();
        data.clear();
        Ok(())
    }

    fn keys(&self, pattern: Option<&str>) -> Result<Vec<String>> {
        self.cleanup_expired();
        let data = self.data.read();
        let keys: Vec<String> = if let Some(pattern) = pattern {
            // Simple pattern matching with wildcards
            let regex_pattern = pattern.replace('*', ".*");
            let regex = regex::Regex::new(&regex_pattern).map_err(|e| {
                crate::errors::PenoosError::CacheError {
                    message: format!("Invalid pattern: {}", e),
                }
            })?;
            data.keys()
                .filter(|key| regex.is_match(key))
                .cloned()
                .collect()
        } else {
            data.keys().cloned().collect()
        };
        Ok(keys)
    }
}

impl CacheManager {
    pub fn new(backend: Box<dyn CacheBackend + Send + Sync>) -> Self {
        Self { backend }
    }

    pub fn with_memory() -> Self {
        Self::new(Box::new(MemoryCache::new()))
    }

    pub fn get(&self, key: &str) -> Result<Option<Value>> {
        self.backend.get(key)
    }

    pub fn set(&self, key: &str, value: Value, ttl: Option<Duration>) -> Result<()> {
        self.backend.set(key, value, ttl)
    }

    pub fn del(&self, key: &str) -> Result<bool> {
        self.backend.del(key)
    }

    pub fn clear(&self) -> Result<()> {
        self.backend.clear()
    }

    pub fn keys(&self, pattern: Option<&str>) -> Result<Vec<String>> {
        self.backend.keys(pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_memory_cache() {
        let cache = MemoryCache::new();
        let key = "test_key";
        let value = json!({"message": "hello"});

        // Test set and get
        cache.set(key, value.clone(), None).unwrap();
        assert_eq!(cache.get(key).unwrap(), Some(value));

        // Test delete
        assert!(cache.del(key).unwrap());
        assert_eq!(cache.get(key).unwrap(), None);

        // Test non-existent key
        assert!(!cache.del("nonexistent").unwrap());
    }

    #[test]
    fn test_cache_ttl() {
        let cache = MemoryCache::new();
        let key = "ttl_key";
        let value = json!("test");

        // Set with very short TTL
        cache.set(key, value.clone(), Some(Duration::from_millis(1))).unwrap();
        
        // Sleep a bit to let it expire
        std::thread::sleep(Duration::from_millis(10));
        
        // Should be None now
        assert_eq!(cache.get(key).unwrap(), None);
    }

    #[test]
    fn test_cache_keys() {
        let cache = MemoryCache::new();
        
        cache.set("user:1", json!("user1"), None).unwrap();
        cache.set("user:2", json!("user2"), None).unwrap();
        cache.set("post:1", json!("post1"), None).unwrap();

        let all_keys = cache.keys(None).unwrap();
        assert_eq!(all_keys.len(), 3);

        let user_keys = cache.keys(Some("user:*")).unwrap();
        assert_eq!(user_keys.len(), 2);
    }
}
