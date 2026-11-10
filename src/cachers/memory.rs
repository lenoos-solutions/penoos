//! In-memory cache implementation

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde_json::Value;
use async_trait::async_trait;

use super::{Cacher, CacheStats, CacheConfig};
use crate::errors::Result;

/// In-memory cache implementation
pub struct MemoryCacher {
    data: Arc<RwLock<HashMap<String, CacheEntry>>>,
    stats: Arc<RwLock<CacheStats>>,
    config: CacheConfig,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: Value,
    created_at: Instant,
    ttl: Option<Duration>,
}

impl CacheEntry {
    fn new(value: Value, ttl: Option<Duration>) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }
    
    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }
}

impl MemoryCacher {
    /// Create a new memory cache
    pub fn new(config: CacheConfig) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            config,
        }
    }
    
    /// Clean expired entries
    async fn cleanup_expired(&self) {
        let mut data = self.data.write().await;
        let keys_to_remove: Vec<String> = data
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();
            
        for key in keys_to_remove {
            data.remove(&key);
        }
    }
    
    /// Apply key prefix if configured
    fn apply_prefix(&self, key: &str) -> String {
        if let Some(prefix) = &self.config.prefix {
            format!("{}:{}", prefix, key)
        } else {
            key.to_string()
        }
    }
}

#[async_trait]
impl Cacher for MemoryCacher {
    async fn get(&self, key: &str) -> Result<Option<Value>> {
        let prefixed_key = self.apply_prefix(key);
        let mut data = self.data.write().await;
        let mut stats = self.stats.write().await;
        
        if let Some(entry) = data.get(&prefixed_key) {
            if entry.is_expired() {
                data.remove(&prefixed_key);
                stats.misses += 1;
                Ok(None)
            } else {
                stats.hits += 1;
                Ok(Some(entry.value.clone()))
            }
        } else {
            stats.misses += 1;
            Ok(None)
        }
    }
    
    async fn set(&self, key: &str, value: Value, ttl: Option<Duration>) -> Result<()> {
        let prefixed_key = self.apply_prefix(key);
        let mut data = self.data.write().await;
        
        // Check max_keys limit
        if let Some(max_keys) = self.config.max_keys {
            if data.len() >= max_keys && !data.contains_key(&prefixed_key) {
                // Remove oldest entry if we're at the limit
                if let Some(oldest_key) = data.keys().next().cloned() {
                    data.remove(&oldest_key);
                }
            }
        }
        
        let effective_ttl = ttl.or(self.config.ttl);
        let entry = CacheEntry::new(value, effective_ttl);
        data.insert(prefixed_key, entry);
        
        Ok(())
    }
    
    async fn del(&self, key: &str) -> Result<bool> {
        let prefixed_key = self.apply_prefix(key);
        let mut data = self.data.write().await;
        Ok(data.remove(&prefixed_key).is_some())
    }
    
    async fn clear(&self) -> Result<()> {
        let mut data = self.data.write().await;
        data.clear();
        
        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();
        
        Ok(())
    }
    
    async fn get_stats(&self) -> Result<CacheStats> {
        // Clean expired entries first
        self.cleanup_expired().await;
        
        let data = self.data.read().await;
        let mut stats = self.stats.read().await.clone();
        
        stats.keys = data.len() as u64;
        stats.size = data
            .values()
            .map(|entry| serde_json::to_string(&entry.value).unwrap_or_default().len())
            .sum::<usize>() as u64;
        
        Ok(stats)
    }
    
    async fn health_check(&self) -> Result<bool> {
        // Memory cache is always healthy if we can read from it
        let _data = self.data.read().await;
        Ok(true)
    }
}

impl Default for MemoryCacher {
    fn default() -> Self {
        Self::new(CacheConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_basic_operations() {
        let cache = MemoryCacher::default();
        
        // Test set and get
        cache.set("key1", json!("value1"), None).await.unwrap();
        let result = cache.get("key1").await.unwrap();
        assert_eq!(result, Some(json!("value1")));
        
        // Test non-existent key
        let result = cache.get("nonexistent").await.unwrap();
        assert_eq!(result, None);
        
        // Test delete
        let deleted = cache.del("key1").await.unwrap();
        assert!(deleted);
        
        let result = cache.get("key1").await.unwrap();
        assert_eq!(result, None);
    }
    
    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache = MemoryCacher::default();
        
        // Set value with short TTL
        cache.set("key1", json!("value1"), Some(Duration::from_millis(10))).await.unwrap();
        
        // Should be available immediately
        let result = cache.get("key1").await.unwrap();
        assert_eq!(result, Some(json!("value1")));
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        // Should be expired
        let result = cache.get("key1").await.unwrap();
        assert_eq!(result, None);
    }
    
    #[tokio::test]
    async fn test_stats() {
        let cache = MemoryCacher::default();
        
        // Initial stats
        let stats = cache.get_stats().await.unwrap();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.keys, 0);
        
        // Add some data
        cache.set("key1", json!("value1"), None).await.unwrap();
        cache.set("key2", json!("value2"), None).await.unwrap();
        
        // Hit
        cache.get("key1").await.unwrap();
        
        // Miss
        cache.get("nonexistent").await.unwrap();
        
        let stats = cache.get_stats().await.unwrap();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.keys, 2);
    }
    
    #[tokio::test]
    async fn test_clear() {
        let cache = MemoryCacher::default();
        
        cache.set("key1", json!("value1"), None).await.unwrap();
        cache.set("key2", json!("value2"), None).await.unwrap();
        
        cache.clear().await.unwrap();
        
        let result = cache.get("key1").await.unwrap();
        assert_eq!(result, None);
        
        let stats = cache.get_stats().await.unwrap();
        assert_eq!(stats.keys, 0);
    }
}
