//! Service mixins for code reuse and composition
//! 
//! Mixins allow you to share common functionality across services,
//! promoting code reuse and consistent patterns.

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use crate::errors::Result;
#[allow(unused_imports)]
use crate::service::{Service, ActionHandler, EventHandler};

/// A mixin that can be applied to services
pub trait Mixin: Send + Sync {
    /// Get the name of this mixin
    fn name(&self) -> &str;
    
    /// Apply this mixin to a service
    fn apply(&self, service: &mut Service) -> Result<()>;
    
    /// Get mixin metadata
    fn metadata(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
    
    /// Get mixin settings
    fn settings(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
}

/// Database mixin providing common database operations
pub struct DatabaseMixin {
    pub connection_string: String,
    pub pool_size: u32,
    pub timeout: u64,
}

impl DatabaseMixin {
    pub fn new(connection_string: String) -> Self {
        Self {
            connection_string,
            pool_size: 10,
            timeout: 30000,
        }
    }
    
    pub fn with_pool_size(mut self, size: u32) -> Self {
        self.pool_size = size;
        self
    }
    
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Mixin for DatabaseMixin {
    fn name(&self) -> &str {
        "database"
    }
    
    fn apply(&self, service: &mut Service) -> Result<()> {
        // Add database connection action
        service.add_action("db.connect", Arc::new(|_ctx| {
            println!("🔗 Connecting to database...");
            Ok(serde_json::json!({
                "connected": true,
                "timestamp": chrono::Utc::now().timestamp()
            }))
        }));
        
        // Add database health check action
        service.add_action("db.health", Arc::new(|_ctx| {
            println!("❤️ Database health check");
            Ok(serde_json::json!({
                "status": "healthy",
                "uptime": 12345,
                "connections": 5
            }))
        }));
        
        // Add database disconnect action
        service.add_action("db.disconnect", Arc::new(|_ctx| {
            println!("🔌 Disconnecting from database...");
            Ok(serde_json::json!({
                "disconnected": true,
                "timestamp": chrono::Utc::now().timestamp()
            }))
        }));
        
        Ok(())
    }
    
    fn metadata(&self) -> HashMap<String, Value> {
        let mut meta = HashMap::new();
        meta.insert("connectionString".to_string(), Value::String(self.connection_string.clone()));
        meta.insert("poolSize".to_string(), Value::Number(self.pool_size.into()));
        meta.insert("timeout".to_string(), Value::Number(self.timeout.into()));
        meta
    }
}

/// Cache mixin providing caching functionality
pub struct CacheMixin {
    pub ttl: u64,
    pub max_keys: Option<u64>,
    pub prefix: Option<String>,
}

impl CacheMixin {
    pub fn new() -> Self {
        Self {
            ttl: 3600, // 1 hour default
            max_keys: None,
            prefix: None,
        }
    }
    
    pub fn with_ttl(mut self, ttl: u64) -> Self {
        self.ttl = ttl;
        self
    }
    
    pub fn with_max_keys(mut self, max_keys: u64) -> Self {
        self.max_keys = Some(max_keys);
        self
    }
    
    pub fn with_prefix(mut self, prefix: String) -> Self {
        self.prefix = Some(prefix);
        self
    }
}

impl Default for CacheMixin {
    fn default() -> Self {
        Self::new()
    }
}

impl Mixin for CacheMixin {
    fn name(&self) -> &str {
        "cache"
    }
    
    fn apply(&self, service: &mut Service) -> Result<()> {
        // Add cache get action
        service.add_action("cache.get", Arc::new(|ctx| {
            let key = ctx.params().get("key").unwrap().as_str().unwrap();
            println!("🗂️ Cache GET: {}", key);
            Ok(serde_json::json!({
                "key": key,
                "value": null,
                "hit": false
            }))
        }));
        
        // Add cache set action
        service.add_action("cache.set", Arc::new(|ctx| {
            let key = ctx.params().get("key").unwrap().as_str().unwrap();
            let value = ctx.params().get("value").unwrap();
            println!("🗂️ Cache SET: {} = {}", key, value);
            Ok(serde_json::json!({
                "key": key,
                "value": value,
                "stored": true
            }))
        }));
        
        // Add cache delete action
        service.add_action("cache.del", Arc::new(|ctx| {
            let key = ctx.params().get("key").unwrap().as_str().unwrap();
            println!("🗂️ Cache DEL: {}", key);
            Ok(serde_json::json!({
                "key": key,
                "deleted": true
            }))
        }));
        
        // Add cache clear action
        service.add_action("cache.clear", Arc::new(|_ctx| {
            println!("🗂️ Cache CLEAR: all keys");
            Ok(serde_json::json!({
                "cleared": true,
                "timestamp": chrono::Utc::now().timestamp()
            }))
        }));
        
        Ok(())
    }
    
    fn settings(&self) -> HashMap<String, Value> {
        let mut settings = HashMap::new();
        settings.insert("ttl".to_string(), Value::Number(self.ttl.into()));
        if let Some(max_keys) = self.max_keys {
            settings.insert("maxKeys".to_string(), Value::Number(max_keys.into()));
        }
        if let Some(ref prefix) = self.prefix {
            settings.insert("prefix".to_string(), Value::String(prefix.clone()));
        }
        settings
    }
}

/// Authentication mixin providing auth functionality
pub struct AuthMixin {
    pub jwt_secret: String,
    pub token_expiry: u64,
    pub refresh_enabled: bool,
}

impl AuthMixin {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            token_expiry: 3600, // 1 hour
            refresh_enabled: true,
        }
    }
    
    pub fn with_expiry(mut self, expiry: u64) -> Self {
        self.token_expiry = expiry;
        self
    }
    
    pub fn with_refresh(mut self, enabled: bool) -> Self {
        self.refresh_enabled = enabled;
        self
    }
}

impl Mixin for AuthMixin {
    fn name(&self) -> &str {
        "auth"
    }
    
    fn apply(&self, service: &mut Service) -> Result<()> {
        // Add login action
        service.add_action("auth.login", Arc::new(|ctx| {
            let username = ctx.params().get("username").unwrap().as_str().unwrap();
            let password = ctx.params().get("password").unwrap().as_str().unwrap();
            
            println!("🔐 Login attempt for user: {}", username);
            
            // Simulate authentication (always successful for demo)
            let success = !username.is_empty() && !password.is_empty();
            
            if success {
                Ok(serde_json::json!({
                    "success": true,
                    "token": format!("jwt_token_for_{}", username),
                    "user": {
                        "username": username,
                        "id": 123,
                        "roles": ["user"]
                    },
                    "expiresIn": 3600
                }))
            } else {
                Ok(serde_json::json!({
                    "success": false,
                    "error": "Invalid credentials"
                }))
            }
        }));
        
        // Add token verification action
        service.add_action("auth.verify", Arc::new(|ctx| {
            let token = ctx.params().get("token").unwrap().as_str().unwrap();
            println!("🔍 Verifying token: {}...", &token[..10.min(token.len())]);
            
            // Simulate token verification
            let valid = token.starts_with("jwt_token_");
            
            Ok(serde_json::json!({
                "valid": valid,
                "decoded": if valid {
                    serde_json::json!({
                        "userId": 123,
                        "username": "demo_user",
                        "exp": chrono::Utc::now().timestamp() + 3600
                    })
                } else {
                    Value::Null
                }
            }))
        }));
        
        // Add logout action
        service.add_action("auth.logout", Arc::new(|ctx| {
            let token = ctx.params().get("token").unwrap().as_str().unwrap();
            println!("👋 Logout for token: {}...", &token[..10.min(token.len())]);
            
            Ok(serde_json::json!({
                "success": true,
                "message": "Logged out successfully"
            }))
        }));
        
        Ok(())
    }
    
    fn metadata(&self) -> HashMap<String, Value> {
        let mut meta = HashMap::new();
        meta.insert("tokenExpiry".to_string(), Value::Number(self.token_expiry.into()));
        meta.insert("refreshEnabled".to_string(), Value::Bool(self.refresh_enabled));
        meta
    }
}

/// Logging mixin providing structured logging
pub struct LoggingMixin {
    pub level: String,
    pub format: String,
    pub include_metadata: bool,
}

impl LoggingMixin {
    pub fn new() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            include_metadata: true,
        }
    }
    
    pub fn with_level(mut self, level: String) -> Self {
        self.level = level;
        self
    }
    
    pub fn with_format(mut self, format: String) -> Self {
        self.format = format;
        self
    }
}

impl Default for LoggingMixin {
    fn default() -> Self {
        Self::new()
    }
}

impl Mixin for LoggingMixin {
    fn name(&self) -> &str {
        "logging"
    }
    
    fn apply(&self, service: &mut Service) -> Result<()> {
        // Add log action
        service.add_action("log.write", Arc::new(|ctx| {
            let level = ctx.params().get("level").unwrap().as_str().unwrap();
            let message = ctx.params().get("message").unwrap().as_str().unwrap();
            let metadata = ctx.params().get("metadata").cloned().unwrap_or(Value::Null);
            
            println!("📝 [{}] {}", level.to_uppercase(), message);
            if !metadata.is_null() {
                println!("    Metadata: {}", metadata);
            }
            
            Ok(serde_json::json!({
                "logged": true,
                "level": level,
                "message": message,
                "timestamp": chrono::Utc::now().timestamp()
            }))
        }));
        
        // Add log query action
        service.add_action("log.query", Arc::new(|ctx| {
            let level = ctx.params().get("level").cloned();
            let since = ctx.params().get("since").cloned();
            
            println!("🔍 Querying logs with filters: level={:?}, since={:?}", level, since);
            
            Ok(serde_json::json!({
                "logs": [
                    {
                        "level": "info",
                        "message": "Service started",
                        "timestamp": chrono::Utc::now().timestamp() - 3600
                    },
                    {
                        "level": "warn",
                        "message": "High memory usage",
                        "timestamp": chrono::Utc::now().timestamp() - 1800
                    }
                ],
                "total": 2
            }))
        }));
        
        Ok(())
    }
    
    fn settings(&self) -> HashMap<String, Value> {
        let mut settings = HashMap::new();
        settings.insert("level".to_string(), Value::String(self.level.clone()));
        settings.insert("format".to_string(), Value::String(self.format.clone()));
        settings.insert("includeMetadata".to_string(), Value::Bool(self.include_metadata));
        settings
    }
}

/// Metrics mixin providing metrics collection
pub struct MetricsMixin {
    pub enabled: bool,
    pub collect_internal: bool,
    pub collect_custom: bool,
}

impl MetricsMixin {
    pub fn new() -> Self {
        Self {
            enabled: true,
            collect_internal: true,
            collect_custom: true,
        }
    }
    
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            collect_internal: false,
            collect_custom: false,
        }
    }
}

impl Default for MetricsMixin {
    fn default() -> Self {
        Self::new()
    }
}

impl Mixin for MetricsMixin {
    fn name(&self) -> &str {
        "metrics"
    }
    
    fn apply(&self, service: &mut Service) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        // Add metrics collection action
        service.add_action("metrics.get", Arc::new(|_ctx| {
            println!("📊 Collecting metrics...");
            
            Ok(serde_json::json!({
                "metrics": {
                    "requests_total": 1250,
                    "requests_per_second": 12.5,
                    "error_rate": 0.02,
                    "response_time_avg": 145.6,
                    "memory_usage": 67.8,
                    "cpu_usage": 23.4
                },
                "timestamp": chrono::Utc::now().timestamp()
            }))
        }));
        
        // Add custom metric recording action
        service.add_action("metrics.record", Arc::new(|ctx| {
            let name = ctx.params().get("name").unwrap().as_str().unwrap();
            let value = ctx.params().get("value").unwrap().as_f64().unwrap();
            let metric_type = ctx.params().get("type").unwrap().as_str().unwrap();
            
            println!("📊 Recording metric: {} = {} ({})", name, value, metric_type);
            
            Ok(serde_json::json!({
                "recorded": true,
                "metric": {
                    "name": name,
                    "value": value,
                    "type": metric_type,
                    "timestamp": chrono::Utc::now().timestamp()
                }
            }))
        }));
        
        Ok(())
    }
    
    fn settings(&self) -> HashMap<String, Value> {
        let mut settings = HashMap::new();
        settings.insert("enabled".to_string(), Value::Bool(self.enabled));
        settings.insert("collectInternal".to_string(), Value::Bool(self.collect_internal));
        settings.insert("collectCustom".to_string(), Value::Bool(self.collect_custom));
        settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::Service;

    #[test]
    fn test_database_mixin() {
        let mut service = Service::new("test");
        let mixin = DatabaseMixin::new("postgresql://localhost:5432/test".to_string())
            .with_pool_size(20)
            .with_timeout(60000);
            
        assert_eq!(mixin.name(), "database");
        mixin.apply(&mut service).unwrap();
        
        // Service should now have database actions
        assert!(service.has_action("db.connect"));
        assert!(service.has_action("db.health"));
        assert!(service.has_action("db.disconnect"));
    }

    #[test]
    fn test_cache_mixin() {
        let mut service = Service::new("test");
        let mixin = CacheMixin::new()
            .with_ttl(7200)
            .with_max_keys(1000)
            .with_prefix("app:".to_string());
            
        assert_eq!(mixin.name(), "cache");
        mixin.apply(&mut service).unwrap();
        
        // Service should now have cache actions
        assert!(service.has_action("cache.get"));
        assert!(service.has_action("cache.set"));
        assert!(service.has_action("cache.del"));
        assert!(service.has_action("cache.clear"));
    }

    #[test]
    fn test_auth_mixin() {
        let mut service = Service::new("test");
        let mixin = AuthMixin::new("super-secret-key".to_string())
            .with_expiry(7200)
            .with_refresh(false);
            
        assert_eq!(mixin.name(), "auth");
        mixin.apply(&mut service).unwrap();
        
        // Service should now have auth actions
        assert!(service.has_action("auth.login"));
        assert!(service.has_action("auth.verify"));
        assert!(service.has_action("auth.logout"));
    }

    #[test]
    fn test_multiple_mixins() {
        let mut service = Service::new("test");
        
        let db_mixin = DatabaseMixin::new("postgresql://localhost:5432/test".to_string());
        let cache_mixin = CacheMixin::default();
        let auth_mixin = AuthMixin::new("secret".to_string());
        
        db_mixin.apply(&mut service).unwrap();
        cache_mixin.apply(&mut service).unwrap();
        auth_mixin.apply(&mut service).unwrap();
        
        // Service should have actions from all mixins
        assert!(service.has_action("db.connect"));
        assert!(service.has_action("cache.get"));
        assert!(service.has_action("auth.login"));
    }
}
