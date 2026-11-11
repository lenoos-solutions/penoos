//! Caching implementations for Penoos

pub mod lru;
pub mod memory;
pub mod redis;

use crate::errors::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::time::Duration;

/// Trait for cache implementations
#[async_trait]
pub trait Cacher: Send + Sync {
    /// Get a value from the cache
    async fn get(&self, key: &str) -> Result<Option<Value>>;

    /// Set a value in the cache with optional TTL
    async fn set(&self, key: &str, value: Value, ttl: Option<Duration>) -> Result<()>;

    /// Delete a value from the cache
    async fn del(&self, key: &str) -> Result<bool>;

    /// Clear all values from the cache
    async fn clear(&self) -> Result<()>;

    /// Get cache statistics
    async fn get_stats(&self) -> Result<CacheStats>;

    /// Check if the cache is healthy
    async fn health_check(&self) -> Result<bool>;
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub keys: u64,
    pub size: u64,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub ttl: Option<Duration>,
    pub max_keys: Option<usize>,
    pub max_size: Option<usize>,
    pub prefix: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl: Some(Duration::from_secs(3600)), // 1 hour default
            max_keys: Some(1000),
            max_size: Some(1024 * 1024), // 1MB default
            prefix: None,
        }
    }
}
