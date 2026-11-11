//! Service definition and management
//!
//! Services are the core building blocks of a Penoos application.
//! Each service can define actions (RPC endpoints) and events (pub/sub handlers).

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use crate::{context::Context, errors::Result, mixins::Mixin};

/// Service action handler type
pub type ActionHandler = Arc<dyn Fn(Context) -> Result<Value> + Send + Sync>;

/// Service event handler type
pub type EventHandler = Arc<dyn Fn(Context) -> Result<()> + Send + Sync>;

/// Service lifecycle hook type
pub type LifecycleHook = Arc<dyn Fn() -> Result<()> + Send + Sync>;

/// Service schema defining the structure and behavior of a service
#[derive(Default)]
pub struct ServiceSchema {
    /// Service name
    pub name: String,
    /// Service version
    pub version: Option<String>,
    /// Service description
    pub description: Option<String>,
    /// Service settings
    pub settings: HashMap<String, Value>,
    /// Service metadata
    pub metadata: HashMap<String, Value>,
    /// Service dependencies
    pub dependencies: Vec<String>,
    /// Service actions (RPC endpoints)
    pub actions: HashMap<String, ActionHandler>,
    /// Service events (pub/sub handlers)
    pub events: HashMap<String, EventHandler>,
    /// Service created lifecycle hook
    pub created: Option<LifecycleHook>,
    /// Service started lifecycle hook
    pub started: Option<LifecycleHook>,
    /// Service stopped lifecycle hook
    pub stopped: Option<LifecycleHook>,
    /// Applied mixins
    pub mixins: Vec<Arc<dyn Mixin>>,
}

/// A Penoos service
pub struct Service {
    /// Service name
    pub name: String,
    /// Service schema
    pub schema: ServiceSchema,
}

impl Service {
    /// Create a new service with the given name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            schema: ServiceSchema {
                name: name.to_string(),
                ..Default::default()
            },
        }
    }

    /// Create a new service with name and version
    pub fn with_version(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            schema: ServiceSchema {
                name: name.to_string(),
                version: Some(version.to_string()),
                ..Default::default()
            },
        }
    }

    /// Set service version
    pub fn version(mut self, version: &str) -> Self {
        self.schema.version = Some(version.to_string());
        self
    }

    /// Set service description
    pub fn description(mut self, description: &str) -> Self {
        self.schema.description = Some(description.to_string());
        self
    }

    /// Add dependencies
    pub fn dependencies(mut self, deps: Vec<&str>) -> Self {
        self.schema.dependencies = deps.into_iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set service settings
    pub fn settings(mut self, settings: HashMap<String, Value>) -> Self {
        self.schema.settings = settings;
        self
    }

    /// Set service metadata
    pub fn metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.schema.metadata = metadata;
        self
    }

    /// Add an action handler
    pub fn action<F>(mut self, name: &str, handler: F) -> Self
    where
        F: Fn(Context) -> Result<Value> + Send + Sync + 'static,
    {
        self.schema
            .actions
            .insert(name.to_string(), Arc::new(handler));
        self
    }

    /// Add an event handler
    pub fn event<F>(mut self, name: &str, handler: F) -> Self
    where
        F: Fn(Context) -> Result<()> + Send + Sync + 'static,
    {
        self.schema
            .events
            .insert(name.to_string(), Arc::new(handler));
        self
    }

    /// Set service created hook
    pub fn created<F>(mut self, hook: F) -> Self
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        self.schema.created = Some(Arc::new(hook));
        self
    }

    /// Set service started hook
    pub fn started<F>(mut self, hook: F) -> Self
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        self.schema.started = Some(Arc::new(hook));
        self
    }

    /// Set service stopped hook
    pub fn stopped<F>(mut self, hook: F) -> Self
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        self.schema.stopped = Some(Arc::new(hook));
        self
    }

    /// Add an action directly (used by mixins)
    pub fn add_action(&mut self, name: &str, handler: ActionHandler) {
        let full_name = if name.contains('.') {
            name.to_string()
        } else {
            format!("{}.{}", self.name, name)
        };
        self.schema.actions.insert(full_name, handler);
    }

    /// Add an event handler directly (used by mixins)
    pub fn add_event(&mut self, name: &str, handler: EventHandler) {
        self.schema.events.insert(name.to_string(), handler);
    }

    /// Add a mixin to the service
    pub fn add_mixin(&mut self, mixin: Arc<dyn Mixin>) -> Result<()> {
        // Apply the mixin to the service
        mixin.apply(self)?;

        // Store the mixin reference
        self.schema.mixins.push(mixin);

        Ok(())
    }

    /// Get all applied mixins
    pub fn get_mixins(&self) -> &[Arc<dyn Mixin>] {
        &self.schema.mixins
    }

    /// Check if a specific mixin is applied
    pub fn has_mixin(&self, mixin_name: &str) -> bool {
        self.schema.mixins.iter().any(|m| m.name() == mixin_name)
    }

    /// Get action handler
    pub fn get_action(&self, action_name: &str) -> Option<&ActionHandler> {
        self.schema.actions.get(action_name)
    }

    /// Check whether an action with the given name exists
    pub fn has_action(&self, action_name: &str) -> bool {
        self.schema.actions.contains_key(action_name)
    }

    /// Get event handler
    pub fn get_event(&self, event_name: &str) -> Option<&EventHandler> {
        self.schema.events.get(event_name)
    }

    /// Get all action names
    pub fn get_actions(&self) -> Vec<&String> {
        self.schema.actions.keys().collect()
    }

    /// Get all event names
    pub fn get_events(&self) -> Vec<&String> {
        self.schema.events.keys().collect()
    }
}

impl From<ServiceSchema> for Service {
    fn from(schema: ServiceSchema) -> Self {
        let name = schema.name.clone();
        Self { name, schema }
    }
}

impl Clone for Service {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            schema: ServiceSchema {
                name: self.schema.name.clone(),
                version: self.schema.version.clone(),
                description: self.schema.description.clone(),
                settings: self.schema.settings.clone(),
                metadata: self.schema.metadata.clone(),
                dependencies: self.schema.dependencies.clone(),
                actions: HashMap::new(), // Actions can't be cloned due to closures
                events: HashMap::new(),  // Events can't be cloned due to closures
                created: None,           // Hooks can't be cloned due to closures
                started: None,
                stopped: None,
                mixins: self.schema.mixins.clone(),
            },
        }
    }
}

/// Builder for convenient service creation
pub struct ServiceBuilder {
    schema: ServiceSchema,
}

impl ServiceBuilder {
    /// Create a new service builder
    pub fn new(name: &str) -> Self {
        Self {
            schema: ServiceSchema {
                name: name.to_string(),
                ..Default::default()
            },
        }
    }

    /// Set version
    pub fn version(mut self, version: &str) -> Self {
        self.schema.version = Some(version.to_string());
        self
    }

    /// Set description
    pub fn description(mut self, description: &str) -> Self {
        self.schema.description = Some(description.to_string());
        self
    }

    /// Add settings
    pub fn settings(mut self, settings: HashMap<String, Value>) -> Self {
        self.schema.settings = settings;
        self
    }

    /// Add dependencies
    pub fn dependencies(mut self, deps: Vec<&str>) -> Self {
        self.schema.dependencies = deps.into_iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add an action
    pub fn action<F>(mut self, name: &str, handler: F) -> Self
    where
        F: Fn(Context) -> Result<Value> + Send + Sync + 'static,
    {
        self.schema
            .actions
            .insert(name.to_string(), Arc::new(handler));
        self
    }

    /// Add an event
    pub fn event<F>(mut self, name: &str, handler: F) -> Self
    where
        F: Fn(Context) -> Result<()> + Send + Sync + 'static,
    {
        self.schema
            .events
            .insert(name.to_string(), Arc::new(handler));
        self
    }

    /// Build the service
    pub fn build(self) -> Service {
        Service::from(self.schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let service = Service::new("test");
        assert_eq!(service.name, "test");
        assert_eq!(service.schema.name, "test");
    }

    #[test]
    fn test_service_builder() {
        let service = ServiceBuilder::new("test")
            .version("1.0.0")
            .description("Test service")
            .build();

        assert_eq!(service.schema.version, Some("1.0.0".to_string()));
        assert_eq!(service.schema.description, Some("Test service".to_string()));
    }
}
