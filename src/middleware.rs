//! Middleware system for Penoos

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::{
    context::Context,
    errors::{Result, PenoosError},
};

/// Rate limiter state
#[derive(Debug)]
struct RateLimiterState {
    requests: Vec<Instant>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiterState {
    fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Vec::new(),
            max_requests,
            window,
        }
    }

    fn is_allowed(&mut self) -> bool {
        let now = Instant::now();
        self.requests.retain(|&time| now.duration_since(time) < self.window);
        
        if self.requests.len() < self.max_requests {
            self.requests.push(now);
            true
        } else {
            false
        }
    }
}

/// Middleware trait
pub trait Middleware: Send + Sync {
    /// Called when a service is created
    fn service_created(&self, _service_name: &str) -> Result<()> {
        Ok(())
    }

    /// Called when a service is started
    fn service_started(&self, _service_name: &str) -> Result<()> {
        Ok(())
    }

    /// Called when a service is stopped
    fn service_stopped(&self, _service_name: &str) -> Result<()> {
        Ok(())
    }

    /// Called before an action is executed
    fn before_call(&self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }

    /// Called after an action is executed
    fn after_call(&self, _ctx: &Context, _result: &Result<serde_json::Value>) -> Result<()> {
        Ok(())
    }

    /// Called before an event is emitted
    fn before_emit(&self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }

    /// Called after an event is emitted
    fn after_emit(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}

/// Middleware manager
pub struct MiddlewareManager {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareManager {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    pub fn add(&mut self, middleware: Arc<dyn Middleware>) {
        self.middlewares.push(middleware);
    }

    pub fn remove(&mut self, index: usize) -> Option<Arc<dyn Middleware>> {
        if index < self.middlewares.len() {
            Some(self.middlewares.remove(index))
        } else {
            None
        }
    }

    pub fn count(&self) -> usize {
        self.middlewares.len()
    }

    pub fn clear(&mut self) {
        self.middlewares.clear();
    }

    // Lifecycle hooks
    pub fn call_service_created(&self, service_name: &str) -> Result<()> {
        for middleware in &self.middlewares {
            middleware.service_created(service_name)?;
        }
        Ok(())
    }

    pub fn call_service_started(&self, service_name: &str) -> Result<()> {
        for middleware in &self.middlewares {
            middleware.service_started(service_name)?;
        }
        Ok(())
    }

    pub fn call_service_stopped(&self, service_name: &str) -> Result<()> {
        for middleware in &self.middlewares {
            middleware.service_stopped(service_name)?;
        }
        Ok(())
    }

    pub fn call_before_call(&self, ctx: &mut Context) -> Result<()> {
        for middleware in &self.middlewares {
            middleware.before_call(ctx)?;
        }
        Ok(())
    }

    pub fn call_after_call(&self, ctx: &Context, result: &Result<serde_json::Value>) -> Result<()> {
        for middleware in &self.middlewares {
            middleware.after_call(ctx, result)?;
        }
        Ok(())
    }

    pub fn call_before_emit(&self, ctx: &mut Context) -> Result<()> {
        for middleware in &self.middlewares {
            middleware.before_emit(ctx)?;
        }
        Ok(())
    }

    pub fn call_after_emit(&self, ctx: &Context) -> Result<()> {
        for middleware in &self.middlewares {
            middleware.after_emit(ctx)?;
        }
        Ok(())
    }
}

impl Default for MiddlewareManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in logging middleware
pub struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn service_created(&self, service_name: &str) -> Result<()> {
        log::info!("Service '{}' created", service_name);
        Ok(())
    }

    fn service_started(&self, service_name: &str) -> Result<()> {
        log::info!("Service '{}' started", service_name);
        Ok(())
    }

    fn service_stopped(&self, service_name: &str) -> Result<()> {
        log::info!("Service '{}' stopped", service_name);
        Ok(())
    }

    fn before_call(&self, ctx: &mut Context) -> Result<()> {
        log::debug!("Calling action '{}' with request ID '{}'", ctx.action, ctx.id);
        Ok(())
    }

    fn after_call(&self, ctx: &Context, result: &Result<serde_json::Value>) -> Result<()> {
        match result {
            Ok(_) => {
                log::debug!("Action '{}' completed successfully (request ID: '{}')", ctx.action, ctx.id);
            }
            Err(e) => {
                log::error!("Action '{}' failed: {} (request ID: '{}')", ctx.action, e, ctx.id);
            }
        }
        Ok(())
    }
}

/// Built-in metrics middleware
pub struct MetricsMiddleware {
    registry: Arc<crate::metrics::MetricsRegistry>,
}

impl MetricsMiddleware {
    pub fn new(registry: Arc<crate::metrics::MetricsRegistry>) -> Self {
        Self { registry }
    }
}

impl Middleware for MetricsMiddleware {
    fn before_call(&self, _ctx: &mut Context) -> Result<()> {
        self.registry.increment_counter("penoos.request.total", 1);
        self.registry.increment_counter("penoos.request.active", 1);
        Ok(())
    }

    fn after_call(&self, ctx: &Context, _result: &Result<serde_json::Value>) -> Result<()> {
        self.registry.increment_counter("penoos.request.active", u64::MAX); // Decrement
        let duration = ctx.elapsed() as f64 / 1000.0; // Convert to seconds
        self.registry.observe_histogram("penoos.request.time", duration);
        Ok(())
    }

    fn before_emit(&self, _ctx: &mut Context) -> Result<()> {
        self.registry.increment_counter("penoos.event.emit.total", 1);
        Ok(())
    }
}

/// Authentication middleware
#[allow(dead_code)]
pub struct AuthMiddleware {
    secret_key: String,
    skip_actions: Vec<String>,
}

impl AuthMiddleware {
    pub fn new(secret_key: String) -> Self {
        Self {
            secret_key,
            skip_actions: vec![
                "users.authenticate".to_string(),
                "users.register".to_string(),
            ],
        }
    }

    pub fn with_skip_actions(mut self, actions: Vec<String>) -> Self {
        self.skip_actions = actions;
        self
    }

    fn verify_token(&self, token: &str) -> Result<bool> {
        // Simple token verification (in real implementation, use JWT)
        if token.starts_with("Bearer ") {
            let token_part = &token[7..];
            // In real implementation, verify JWT signature with secret_key
            Ok(!token_part.is_empty())
        } else {
            Ok(false)
        }
    }
}

impl Middleware for AuthMiddleware {
    fn before_call(&self, ctx: &mut Context) -> Result<()> {
        let action_name = format!("{}.{}", ctx.service_name().unwrap_or(""), ctx.action_name().unwrap_or(""));
        
        if self.skip_actions.contains(&action_name) {
            return Ok(());
        }

        let auth_header = ctx.meta()
            .get("authorization")
            .or_else(|| ctx.meta().get("Authorization"))
            .and_then(|v| v.as_str());

        match auth_header {
            Some(token) => {
                if self.verify_token(token)? {
                    Ok(())
                } else {
                    Err(PenoosError::UnauthorizedError { message: "Invalid token".to_string() })
                }
            }
            None => Err(PenoosError::UnauthorizedError { message: "Missing authorization header".to_string() }),
        }
    }
}

/// Rate limiting middleware
pub struct RateLimitMiddleware {
    limiters: Arc<Mutex<HashMap<String, RateLimiterState>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimitMiddleware {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            limiters: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    fn get_client_id(&self, ctx: &Context) -> String {
        // Extract client ID from context (IP, user ID, etc.)
        ctx.meta()
            .get("client_ip")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string()
    }
}

impl Middleware for RateLimitMiddleware {
    fn before_call(&self, ctx: &mut Context) -> Result<()> {
        let client_id = self.get_client_id(ctx);
        let mut limiters = self.limiters.lock().unwrap();
        
        let limiter = limiters
            .entry(client_id.clone())
            .or_insert_with(|| RateLimiterState::new(self.max_requests, self.window));

        if limiter.is_allowed() {
            Ok(())
        } else {
            Err(PenoosError::RateLimitError { message: format!(
                "Rate limit exceeded for client: {}", client_id
            ) })
        }
    }
}

/// Cache middleware for action responses
pub struct CacheMiddleware {
    ttl: Duration,
    cache: Arc<Mutex<HashMap<String, (serde_json::Value, Instant)>>>,
}

impl CacheMiddleware {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn get_cache_key(&self, ctx: &Context) -> String {
        format!("{}:{}", 
            ctx.action_name().unwrap_or("unknown"),
            serde_json::to_string(ctx.params()).unwrap_or_default()
        )
    }

    fn get_cached(&self, key: &str) -> Option<serde_json::Value> {
        let mut cache = self.cache.lock().unwrap();
        if let Some((value, timestamp)) = cache.get(key) {
            if timestamp.elapsed() < self.ttl {
                Some(value.clone())
            } else {
                cache.remove(key);
                None
            }
        } else {
            None
        }
    }

    fn set_cache(&self, key: String, value: serde_json::Value) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(key, (value, Instant::now()));
    }
}

impl Middleware for CacheMiddleware {
    fn before_call(&self, ctx: &mut Context) -> Result<()> {
        let cache_key = self.get_cache_key(ctx);
        
        if let Some(cached_value) = self.get_cached(&cache_key) {
            // Store cached result in context meta for retrieval
            ctx.meta_mut().insert("cached_result".to_string(), cached_value);
            ctx.meta_mut().insert("from_cache".to_string(), serde_json::Value::Bool(true));
        }
        
        Ok(())
    }

    fn after_call(&self, ctx: &Context, result: &Result<serde_json::Value>) -> Result<()> {
        // Only cache successful results
        if let Ok(value) = result {
            // Don't cache if result was already from cache
            if !ctx.meta().get("from_cache").and_then(|v| v.as_bool()).unwrap_or(false) {
                let cache_key = self.get_cache_key(ctx);
                self.set_cache(cache_key, value.clone());
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestMiddleware {
        call_count: AtomicUsize,
    }

    impl TestMiddleware {
        fn new() -> Self {
            Self {
                call_count: AtomicUsize::new(0),
            }
        }

        fn get_call_count(&self) -> usize {
            self.call_count.load(Ordering::Relaxed)
        }
    }

    impl Middleware for TestMiddleware {
        fn service_created(&self, _service_name: &str) -> Result<()> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }

    #[test]
    fn test_middleware_manager() {
        let mut manager = MiddlewareManager::new();
        let middleware = Arc::new(TestMiddleware::new());
        
        manager.add(middleware.clone());
        assert_eq!(manager.count(), 1);

        manager.call_service_created("test").unwrap();
        assert_eq!(middleware.get_call_count(), 1);

        manager.clear();
        assert_eq!(manager.count(), 0);
    }
}
