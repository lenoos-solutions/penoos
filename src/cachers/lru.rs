//! LRU (Least Recently Used) cache implementation

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::{CacheConfig, CacheStats, Cacher};
use crate::errors::Result;

/// LRU cache implementation
pub struct LruCacher {
    data: Arc<RwLock<LruCache>>,
    stats: Arc<RwLock<CacheStats>>,
    config: CacheConfig,
}

struct LruCache {
    map: HashMap<String, LruNode>,
    head: Option<String>,
    tail: Option<String>,
    capacity: usize,
}

#[derive(Debug, Clone)]
struct LruNode {
    value: Value,
    created_at: Instant,
    ttl: Option<Duration>,
    prev: Option<String>,
    next: Option<String>,
}

impl LruNode {
    fn new(value: Value, ttl: Option<Duration>) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
            prev: None,
            next: None,
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

impl LruCache {
    fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            head: None,
            tail: None,
            capacity,
        }
    }

    fn get(&mut self, key: &str) -> Option<&Value> {
        if let Some(node) = self.map.get(key) {
            if node.is_expired() {
                self.remove(key);
                None
            } else {
                self.move_to_head(key);
                self.map.get(key).map(|n| &n.value)
            }
        } else {
            None
        }
    }

    fn put(&mut self, key: String, value: Value, ttl: Option<Duration>) {
        if self.map.contains_key(&key) {
            // Update existing
            if let Some(node) = self.map.get_mut(&key) {
                node.value = value;
                node.created_at = Instant::now();
                node.ttl = ttl;
            }
            self.move_to_head(&key);
        } else {
            // Add new
            if self.map.len() >= self.capacity
                && let Some(tail_key) = self.tail.clone()
            {
                self.remove(&tail_key);
            }

            let node = LruNode::new(value, ttl);
            self.map.insert(key.clone(), node);
            self.add_to_head(key);
        }
    }

    fn remove(&mut self, key: &str) -> bool {
        if let Some(node) = self.map.remove(key) {
            // Update prev node
            if let Some(prev_key) = &node.prev {
                if let Some(prev_node) = self.map.get_mut(prev_key) {
                    prev_node.next = node.next.clone();
                }
            } else {
                self.head = node.next.clone();
            }

            // Update next node
            if let Some(next_key) = &node.next {
                if let Some(next_node) = self.map.get_mut(next_key) {
                    next_node.prev = node.prev.clone();
                }
            } else {
                self.tail = node.prev.clone();
            }

            true
        } else {
            false
        }
    }

    fn move_to_head(&mut self, key: &str) {
        if self.head.as_deref() == Some(key) {
            return; // Already at head
        }

        // Remove from current position
        if let Some(node) = self.map.get(key).cloned() {
            // Update prev node
            if let Some(prev_key) = &node.prev
                && let Some(prev_node) = self.map.get_mut(prev_key)
            {
                prev_node.next = node.next.clone();
            }

            // Update next node
            if let Some(next_key) = &node.next {
                if let Some(next_node) = self.map.get_mut(next_key) {
                    next_node.prev = node.prev.clone();
                }
            } else {
                self.tail = node.prev.clone();
            }
        }

        // Add to head
        self.add_to_head(key.to_string());
    }

    fn add_to_head(&mut self, key: String) {
        if let Some(node) = self.map.get_mut(&key) {
            node.prev = None;
            node.next = self.head.clone();
        }

        if let Some(old_head) = &self.head
            && let Some(old_head_node) = self.map.get_mut(old_head)
        {
            old_head_node.prev = Some(key.clone());
        }

        self.head = Some(key.clone());

        if self.tail.is_none() {
            self.tail = Some(key);
        }
    }

    fn clear(&mut self) {
        self.map.clear();
        self.head = None;
        self.tail = None;
    }

    fn cleanup_expired(&mut self) {
        let expired_keys: Vec<String> = self
            .map
            .iter()
            .filter(|(_, node)| node.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.remove(&key);
        }
    }
}

impl LruCacher {
    /// Create a new LRU cache
    pub fn new(config: CacheConfig) -> Self {
        let capacity = config.max_keys.unwrap_or(1000);

        Self {
            data: Arc::new(RwLock::new(LruCache::new(capacity))),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            config,
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
impl Cacher for LruCacher {
    async fn get(&self, key: &str) -> Result<Option<Value>> {
        let prefixed_key = self.apply_prefix(key);
        let mut data = self.data.write().await;
        let mut stats = self.stats.write().await;

        if let Some(value) = data.get(&prefixed_key) {
            stats.hits += 1;
            Ok(Some(value.clone()))
        } else {
            stats.misses += 1;
            Ok(None)
        }
    }

    async fn set(&self, key: &str, value: Value, ttl: Option<Duration>) -> Result<()> {
        let prefixed_key = self.apply_prefix(key);
        let mut data = self.data.write().await;

        let effective_ttl = ttl.or(self.config.ttl);
        data.put(prefixed_key, value, effective_ttl);

        Ok(())
    }

    async fn del(&self, key: &str) -> Result<bool> {
        let prefixed_key = self.apply_prefix(key);
        let mut data = self.data.write().await;
        Ok(data.remove(&prefixed_key))
    }

    async fn clear(&self) -> Result<()> {
        let mut data = self.data.write().await;
        data.clear();

        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();

        Ok(())
    }

    async fn get_stats(&self) -> Result<CacheStats> {
        let mut data = self.data.write().await;
        data.cleanup_expired();

        let mut stats = self.stats.read().await.clone();
        stats.keys = data.map.len() as u64;
        stats.size = data
            .map
            .values()
            .map(|node| serde_json::to_string(&node.value).unwrap_or_default().len())
            .sum::<usize>() as u64;

        Ok(stats)
    }

    async fn health_check(&self) -> Result<bool> {
        // LRU cache is always healthy if we can read from it
        let _data = self.data.read().await;
        Ok(true)
    }
}

impl Default for LruCacher {
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
    async fn test_lru_basic_operations() {
        let config = CacheConfig {
            max_keys: Some(3),
            ..Default::default()
        };
        let cache = LruCacher::new(config);

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
    async fn test_lru_eviction() {
        let config = CacheConfig {
            max_keys: Some(2),
            ..Default::default()
        };
        let cache = LruCacher::new(config);

        // Fill cache to capacity
        cache.set("key1", json!("value1"), None).await.unwrap();
        cache.set("key2", json!("value2"), None).await.unwrap();

        // Access key1 to make it more recently used than key2
        assert_eq!(cache.get("key1").await.unwrap(), Some(json!("value1")));

        // Add one more (should evict least recently used, which is key2)
        cache.set("key3", json!("value3"), None).await.unwrap();

        // key1 should still be available, key2 should be evicted
        assert_eq!(cache.get("key1").await.unwrap(), Some(json!("value1")));
        assert_eq!(cache.get("key2").await.unwrap(), None);
        assert_eq!(cache.get("key3").await.unwrap(), Some(json!("value3")));
    }

    #[tokio::test]
    async fn test_lru_ttl_expiration() {
        let cache = LruCacher::default();

        // Set value with short TTL
        cache
            .set("key1", json!("value1"), Some(Duration::from_millis(10)))
            .await
            .unwrap();

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
    async fn test_lru_stats() {
        let cache = LruCacher::default();

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
}
