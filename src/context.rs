//! Context for action calls and events
//!
//! The Context object is passed to action handlers and event handlers,
//! containing the request parameters, metadata, and other contextual information.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use uuid::Uuid;

/// Context for action calls and events
#[derive(Debug, Clone)]
pub struct Context {
    /// Unique call identifier
    pub id: String,
    /// Node ID where the call originates
    pub node_id: String,
    /// Request ID for tracing
    pub request_id: String,
    /// Action or event name
    pub action: String,
    /// Call parameters
    params: Value,
    /// Metadata
    meta: HashMap<String, Value>,
    /// Headers for transport
    headers: HashMap<String, String>,
    /// Response headers
    response_headers: HashMap<String, String>,
    /// Local storage for the context
    locals: HashMap<String, Value>,
    /// Call level (for nested calls)
    pub level: u32,
    /// Parent context ID
    pub parent_id: Option<String>,
    /// Caller service name
    pub caller: Option<String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Timeout for the call
    pub timeout: Option<u64>,
    /// Number of retries
    pub retries: Option<u32>,
}

impl Context {
    /// Create a new context
    pub fn new(
        node_id: String,
        action: String,
        params: Value,
        meta: HashMap<String, Value>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let request_id = id.clone();
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            id,
            node_id,
            request_id,
            action,
            params,
            meta,
            headers: HashMap::new(),
            response_headers: HashMap::new(),
            locals: HashMap::new(),
            level: 1,
            parent_id: None,
            caller: None,
            created_at,
            timeout: None,
            retries: None,
        }
    }

    /// Create a child context for nested calls
    pub fn child(&self, action: String, params: Value) -> Self {
        let id = Uuid::new_v4().to_string();
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            id,
            node_id: self.node_id.clone(),
            request_id: self.request_id.clone(),
            action,
            params,
            meta: self.meta.clone(),
            headers: self.headers.clone(),
            response_headers: HashMap::new(),
            locals: HashMap::new(),
            level: self.level + 1,
            parent_id: Some(self.id.clone()),
            caller: Some(self.action.clone()),
            created_at,
            timeout: self.timeout,
            retries: self.retries,
        }
    }

    /// Get the call parameters
    pub fn params(&self) -> &Value {
        &self.params
    }

    /// Get a mutable reference to the parameters
    pub fn params_mut(&mut self) -> &mut Value {
        &mut self.params
    }

    /// Get metadata
    pub fn meta(&self) -> &HashMap<String, Value> {
        &self.meta
    }

    /// Get a mutable reference to metadata
    pub fn meta_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.meta
    }

    /// Set a metadata value
    pub fn set_meta<T: Into<Value>>(&mut self, key: &str, value: T) {
        self.meta.insert(key.to_string(), value.into());
    }

    /// Get a metadata value
    pub fn get_meta(&self, key: &str) -> Option<&Value> {
        self.meta.get(key)
    }

    /// Get headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Get a mutable reference to headers
    pub fn headers_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.headers
    }

    /// Set a header
    pub fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    /// Get a header value
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    /// Get response headers
    pub fn response_headers(&self) -> &HashMap<String, String> {
        &self.response_headers
    }

    /// Get a mutable reference to response headers
    pub fn response_headers_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.response_headers
    }

    /// Set a response header
    pub fn set_response_header(&mut self, key: &str, value: &str) {
        self.response_headers
            .insert(key.to_string(), value.to_string());
    }

    /// Get a response header value
    pub fn get_response_header(&self, key: &str) -> Option<&String> {
        self.response_headers.get(key)
    }

    /// Get local storage
    pub fn locals(&self) -> &HashMap<String, Value> {
        &self.locals
    }

    /// Get a mutable reference to local storage
    pub fn locals_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.locals
    }

    /// Set a local value
    pub fn set_local<T: Into<Value>>(&mut self, key: &str, value: T) {
        self.locals.insert(key.to_string(), value.into());
    }

    /// Get a local value
    pub fn get_local(&self, key: &str) -> Option<&Value> {
        self.locals.get(key)
    }

    /// Set the timeout for this context
    pub fn set_timeout(&mut self, timeout: u64) {
        self.timeout = Some(timeout);
    }

    /// Set the number of retries for this context
    pub fn set_retries(&mut self, retries: u32) {
        self.retries = Some(retries);
    }

    /// Check if the context has timed out
    pub fn has_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
            now - self.created_at > timeout
        } else {
            false
        }
    }

    /// Get the elapsed time since context creation
    pub fn elapsed(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        now - self.created_at
    }

    /// Copy the context for broadcasting
    pub fn copy(&self) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            node_id: self.node_id.clone(),
            request_id: self.request_id.clone(),
            action: self.action.clone(),
            params: self.params.clone(),
            meta: self.meta.clone(),
            headers: self.headers.clone(),
            response_headers: HashMap::new(),
            locals: HashMap::new(),
            level: self.level,
            parent_id: self.parent_id.clone(),
            caller: self.caller.clone(),
            created_at: self.created_at,
            timeout: self.timeout,
            retries: self.retries,
        }
    }

    /// Check if this is a root context (level 1)
    pub fn is_root(&self) -> bool {
        self.level == 1
    }

    /// Check if this is a child context
    pub fn is_child(&self) -> bool {
        self.level > 1
    }

    /// Get the full call chain
    pub fn call_chain(&self) -> Vec<String> {
        let mut chain = Vec::new();
        if let Some(caller) = &self.caller {
            chain.push(caller.clone());
        }
        chain.push(self.action.clone());
        chain
    }

    /// Get service name from action name
    pub fn service_name(&self) -> Option<&str> {
        self.action.split('.').next()
    }

    /// Get action name without service prefix
    pub fn action_name(&self) -> Option<&str> {
        self.action.split('.').nth(1)
    }

    /// Call another service action (placeholder - needs broker)
    pub async fn call(&self, action: &str, _params: &Value) -> crate::errors::Result<Value> {
        // This would normally call through the broker
        // For now, return an error indicating it needs broker integration
        Err(crate::errors::PenoosError::ServiceNotFound {
            service: action.to_string(),
        })
    }

    /// Emit an event (placeholder - needs broker)
    pub async fn emit(&self, event: &str, data: &Value) -> crate::errors::Result<()> {
        // This would normally emit through the broker
        // For now, just log it
        log::info!("Event emitted: {} with data: {}", event, data);
        Ok(())
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new(
            "local".to_string(),
            "unknown".to_string(),
            Value::Null,
            HashMap::new(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_context_creation() {
        let params = json!({"name": "test"});
        let mut meta = HashMap::new();
        meta.insert("user".to_string(), json!("testuser"));

        let ctx = Context::new(
            "node1".to_string(),
            "test.action".to_string(),
            params.clone(),
            meta.clone(),
        );

        assert_eq!(ctx.node_id, "node1");
        assert_eq!(ctx.action, "test.action");
        assert_eq!(ctx.params(), &params);
        assert_eq!(ctx.meta().get("user"), Some(&json!("testuser")));
        assert_eq!(ctx.level, 1);
        assert!(ctx.is_root());
        assert!(!ctx.is_child());
    }

    #[test]
    fn test_child_context() {
        let parent = Context::new(
            "node1".to_string(),
            "parent.action".to_string(),
            json!({}),
            HashMap::new(),
        );

        let child = parent.child("child.action".to_string(), json!({"data": "test"}));

        assert_eq!(child.level, 2);
        assert_eq!(child.parent_id, Some(parent.id.clone()));
        assert_eq!(child.caller, Some("parent.action".to_string()));
        assert_eq!(child.request_id, parent.request_id);
        assert!(!child.is_root());
        assert!(child.is_child());
    }

    #[test]
    fn test_context_metadata() {
        let mut ctx = Context::default();

        ctx.set_meta("key1", "value1");
        ctx.set_meta("key2", 42);

        assert_eq!(ctx.get_meta("key1"), Some(&json!("value1")));
        assert_eq!(ctx.get_meta("key2"), Some(&json!(42)));
        assert_eq!(ctx.get_meta("nonexistent"), None);
    }

    #[test]
    fn test_context_headers() {
        let mut ctx = Context::default();

        ctx.set_header("Authorization", "Bearer token123");
        ctx.set_response_header("Content-Type", "application/json");

        assert_eq!(
            ctx.get_header("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(
            ctx.get_response_header("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_context_locals() {
        let mut ctx = Context::default();

        ctx.set_local("temp_data", json!({"important": true}));

        assert_eq!(
            ctx.get_local("temp_data"),
            Some(&json!({"important": true}))
        );
    }

    #[test]
    fn test_call_chain() {
        let parent = Context::new(
            "node1".to_string(),
            "service1.action1".to_string(),
            json!({}),
            HashMap::new(),
        );

        let child = parent.child("service2.action2".to_string(), json!({}));

        let chain = child.call_chain();
        assert_eq!(chain, vec!["service1.action1", "service2.action2"]);
    }
}
