//! Redis cache implementation

use async_trait::async_trait;
use serde_json::Value;
use std::time::Duration;

use super::{CacheConfig, CacheStats, Cacher};
use crate::errors::{PenoosError, Result};

/// Redis cache implementation
#[cfg(feature = "redis-cache")]
#[allow(dead_code)]
pub struct RedisCacher {
    client: redis::Client,
    connection: tokio::sync::Mutex<redis::aio::MultiplexedConnection>,
    config: CacheConfig,
    stats: std::sync::Arc<tokio::sync::RwLock<CacheStats>>,
}

#[cfg(feature = "redis-cache")]
impl RedisCacher {
    /// Create a new Redis cache
    pub async fn new(redis_url: &str, config: CacheConfig) -> Result<Self> {
        let client = redis::Client::open(redis_url).map_err(|e| PenoosError::CacheError {
            message: format!("Failed to connect to Redis: {}", e),
        })?;

        let connection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| PenoosError::CacheError {
                message: format!("Failed to get Redis connection: {}", e),
            })?;

        Ok(Self {
            client,
            connection: tokio::sync::Mutex::new(connection),
            config,
            stats: std::sync::Arc::new(tokio::sync::RwLock::new(CacheStats::default())),
        })
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

#[cfg(feature = "redis-cache")]
#[async_trait]
impl Cacher for RedisCacher {
    async fn get(&self, key: &str) -> Result<Option<Value>> {
        use redis::AsyncCommands;

        let prefixed_key = self.apply_prefix(key);
        let mut conn = self.connection.lock().await;
        let mut stats = self.stats.write().await;

        let result: Option<String> =
            conn.get(&prefixed_key)
                .await
                .map_err(|e| PenoosError::CacheError {
                    message: format!("Redis GET error: {}", e),
                })?;

        if let Some(value_str) = result {
            match serde_json::from_str(&value_str) {
                Ok(value) => {
                    stats.hits += 1;
                    Ok(Some(value))
                }
                Err(e) => {
                    stats.misses += 1;
                    Err(PenoosError::SerializationError {
                        message: format!("Failed to deserialize cached value: {}", e),
                    })
                }
            }
        } else {
            stats.misses += 1;
            Ok(None)
        }
    }

    async fn set(&self, key: &str, value: Value, ttl: Option<Duration>) -> Result<()> {
        use redis::AsyncCommands;

        let prefixed_key = self.apply_prefix(key);
        let mut conn = self.connection.lock().await;

        let value_str =
            serde_json::to_string(&value).map_err(|e| PenoosError::SerializationError {
                message: format!("Failed to serialize value: {}", e),
            })?;

        let effective_ttl = ttl.or(self.config.ttl);

        if let Some(ttl_duration) = effective_ttl {
            let ttl_seconds = ttl_duration.as_secs();
            conn.set_ex::<_, _, ()>(&prefixed_key, value_str, ttl_seconds)
                .await
        } else {
            conn.set::<_, _, ()>(&prefixed_key, value_str).await
        }
        .map_err(|e| PenoosError::CacheError {
            message: format!("Redis SET error: {}", e),
        })?;

        Ok(())
    }

    async fn del(&self, key: &str) -> Result<bool> {
        use redis::AsyncCommands;

        let prefixed_key = self.apply_prefix(key);
        let mut conn = self.connection.lock().await;

        let deleted: i32 = conn
            .del(&prefixed_key)
            .await
            .map_err(|e| PenoosError::CacheError {
                message: format!("Redis DEL error: {}", e),
            })?;

        Ok(deleted > 0)
    }

    async fn clear(&self) -> Result<()> {
        use redis::AsyncCommands;

        let mut conn = self.connection.lock().await;

        if let Some(prefix) = &self.config.prefix {
            // Delete all keys with the prefix
            let pattern = format!("{}:*", prefix);
            let keys: Vec<String> =
                conn.keys(&pattern)
                    .await
                    .map_err(|e| PenoosError::CacheError {
                        message: format!("Redis KEYS error: {}", e),
                    })?;

            if !keys.is_empty() {
                let _: () = conn.del(keys).await.map_err(|e| PenoosError::CacheError {
                    message: format!("Redis DEL error: {}", e),
                })?;
            }
        } else {
            // Clear all keys (dangerous!)
            let _: () = redis::cmd("FLUSHDB")
                .query_async(&mut *conn)
                .await
                .map_err(|e| PenoosError::CacheError {
                    message: format!("Redis FLUSHDB error: {}", e),
                })?;
        }

        // Reset stats
        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();

        Ok(())
    }

    async fn get_stats(&self) -> Result<CacheStats> {
        use redis::AsyncCommands;

        let mut conn = self.connection.lock().await;
        let mut stats = self.stats.read().await.clone();

        // Get key count
        if let Some(prefix) = &self.config.prefix {
            let pattern = format!("{}:*", prefix);
            let keys: Vec<String> =
                conn.keys(&pattern)
                    .await
                    .map_err(|e| PenoosError::CacheError {
                        message: format!("Redis KEYS error: {}", e),
                    })?;
            stats.keys = keys.len() as u64;
        } else {
            let info: String = redis::cmd("INFO")
                .arg("keyspace")
                .query_async(&mut *conn)
                .await
                .map_err(|e| PenoosError::CacheError {
                    message: format!("Redis INFO error: {}", e),
                })?;

            // Parse keyspace info to get key count
            stats.keys = info
                .lines()
                .filter(|line| line.starts_with("db0:"))
                .filter_map(|line| {
                    line.split(',')
                        .find(|part| part.starts_with("keys="))
                        .and_then(|part| part.strip_prefix("keys="))
                        .and_then(|num| num.parse::<u64>().ok())
                })
                .next()
                .unwrap_or(0);
        }

        Ok(stats)
    }

    async fn health_check(&self) -> Result<bool> {
        let mut conn = self.connection.lock().await;
        let result: String = redis::cmd("PING")
            .query_async(&mut *conn)
            .await
            .map_err(|e| PenoosError::CacheError {
                message: format!("Redis PING error: {}", e),
            })?;

        Ok(result == "PONG")
    }
}

// Stub implementation when Redis feature is not enabled
#[cfg(not(feature = "redis-cache"))]
pub struct RedisCacher;

#[cfg(not(feature = "redis-cache"))]
impl RedisCacher {
    pub async fn new(_redis_url: &str, _config: CacheConfig) -> Result<Self> {
        Err(PenoosError::CacheError {
            message: "Redis cache is not enabled. Enable the 'redis-cache' feature".to_string(),
        })
    }
}

#[cfg(not(feature = "redis-cache"))]
#[async_trait]
impl Cacher for RedisCacher {
    async fn get(&self, _key: &str) -> Result<Option<Value>> {
        Err(PenoosError::CacheError {
            message: "Redis cache is not enabled".to_string(),
        })
    }

    async fn set(&self, _key: &str, _value: Value, _ttl: Option<Duration>) -> Result<()> {
        Err(PenoosError::CacheError {
            message: "Redis cache is not enabled".to_string(),
        })
    }

    async fn del(&self, _key: &str) -> Result<bool> {
        Err(PenoosError::CacheError {
            message: "Redis cache is not enabled".to_string(),
        })
    }

    async fn clear(&self) -> Result<()> {
        Err(PenoosError::CacheError {
            message: "Redis cache is not enabled".to_string(),
        })
    }

    async fn get_stats(&self) -> Result<CacheStats> {
        Err(PenoosError::CacheError {
            message: "Redis cache is not enabled".to_string(),
        })
    }

    async fn health_check(&self) -> Result<bool> {
        Err(PenoosError::CacheError {
            message: "Redis cache is not enabled".to_string(),
        })
    }
}

#[cfg(test)]
#[cfg(feature = "redis-cache")]
mod tests {
    use super::*;
    use serde_json::json;

    // Note: These tests require a running Redis instance
    // Run with: docker run -d -p 6379:6379 redis:alpine

    async fn create_test_cache() -> RedisCacher {
        let config = CacheConfig {
            prefix: Some("test".to_string()),
            ..Default::default()
        };

        RedisCacher::new("redis://127.0.0.1:6379", config)
            .await
            .unwrap()
    }

    #[tokio::test]
    #[ignore] // Requires Redis server
    async fn test_redis_basic_operations() {
        let cache = create_test_cache().await;

        // Clear any existing test data
        cache.clear().await.unwrap();

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
    #[ignore] // Requires Redis server
    async fn test_redis_health_check() {
        let cache = create_test_cache().await;
        let healthy = cache.health_check().await.unwrap();
        assert!(healthy);
    }
}
