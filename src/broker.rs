//! Service Broker - The core component of Penoos framework
//!
//! The ServiceBroker is the main component of Penoos. It handles communication
//! between services, manages the service registry, and coordinates all framework features.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use parking_lot::RwLock;
use serde_json::Value;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    PenoosConfig,
    cache::CacheManager,
    context::Context,
    errors::{PenoosError, Result},
    metrics::MetricsRegistry,
    registry::ServiceRegistry,
    service::Service,
    transport::Transport,
    utils,
};

/// Options for calling actions
#[derive(Debug, Clone, Default)]
pub struct CallOptions {
    pub timeout: Option<Duration>,
    pub retries: Option<u32>,
    pub node_id: Option<String>,
    pub meta: Option<HashMap<String, Value>>,
}

/// Multiple call definition for batch calls
#[derive(Debug)]
pub enum MCallDefinition {
    Array(Vec<CallDefinition>),
    Object(HashMap<String, CallDefinition>),
}

#[derive(Debug)]
pub struct CallDefinition {
    pub action: String,
    pub params: Value,
    pub options: Option<CallOptions>,
}

/// The main ServiceBroker struct
pub struct ServiceBroker {
    /// Unique node identifier
    pub node_id: String,
    /// Namespace for this broker instance
    pub namespace: String,
    /// Instance ID for this broker
    pub instance_id: String,
    /// Broker configuration
    pub config: PenoosConfig,
    /// Service registry
    pub registry: Arc<ServiceRegistry>,
    /// Transport layer
    pub transport: Option<Arc<dyn Transport + Send + Sync>>,
    /// Cache manager
    pub cache: Option<Arc<CacheManager>>,
    /// Metrics registry
    pub metrics: Arc<MetricsRegistry>,
    /// Local services
    services: Arc<RwLock<Vec<Arc<Service>>>>,
    /// Started flag
    started: Arc<RwLock<bool>>,
    /// Event emitter for internal events
    event_sender: broadcast::Sender<InternalEvent>,
    /// Action call handlers
    call_handlers: Arc<DashMap<String, ActionCallHandler>>,
}

/// Internal events for the broker
#[derive(Debug, Clone)]
pub enum InternalEvent {
    ServiceStarted(String),
    ServiceStopped(String),
    NodeConnected(String),
    NodeDisconnected(String),
    Error(String),
}

/// Handler for action calls
/// Action call handler type for the broker
type ActionCallHandler = Arc<dyn Fn(Context) -> Result<Value> + Send + Sync>;

impl ServiceBroker {
    /// Create a new ServiceBroker instance
    pub fn new() -> Result<Self> {
        Self::with_config(PenoosConfig::default())
    }

    /// Create a new ServiceBroker with custom configuration
    pub fn with_config(config: PenoosConfig) -> Result<Self> {
        let node_id = config
            .node_id
            .clone()
            .unwrap_or_else(utils::generate_node_id);
        let instance_id = Uuid::new_v4().to_string();
        let (event_sender, _) = broadcast::channel(1000);

        Ok(ServiceBroker {
            node_id,
            namespace: config.namespace.clone(),
            instance_id,
            registry: Arc::new(ServiceRegistry::new()),
            transport: None,
            cache: None,
            metrics: Arc::new(MetricsRegistry::new()),
            services: Arc::new(RwLock::new(Vec::new())),
            started: Arc::new(RwLock::new(false)),
            event_sender,
            call_handlers: Arc::new(DashMap::new()),
            config,
        })
    }

    /// Start the service broker
    pub async fn start(&mut self) -> Result<()> {
        if *self.started.read() {
            return Err(PenoosError::AlreadyStarted);
        }

        log::info!("Starting ServiceBroker...");
        log::info!("Penoos v{}", crate::VERSION);
        log::info!("Node ID: {}", self.node_id);
        log::info!(
            "Namespace: {}",
            if self.namespace.is_empty() {
                "<not defined>"
            } else {
                &self.namespace
            }
        );

        // Initialize transport if configured
        if let Some(transport) = &self.transport {
            transport.connect().await?;
        }

        // Start all registered services
        let services = self.services.read().clone();
        for service in services {
            self.start_service(&service).await?;
        }

        *self.started.write() = true;

        // Emit started event
        let _ = self
            .event_sender
            .send(InternalEvent::ServiceStarted(self.node_id.clone()));

        log::info!("✓ ServiceBroker started successfully");
        Ok(())
    }

    /// Stop the service broker
    pub async fn stop(&mut self) -> Result<()> {
        if !*self.started.read() {
            return Ok(());
        }

        log::info!("Stopping ServiceBroker...");

        // Stop all services
        let services = self.services.read().clone();
        for service in services {
            self.stop_service(&service).await?;
        }

        // Disconnect transport
        if let Some(transport) = &self.transport {
            transport.disconnect().await?;
        }

        *self.started.write() = false;

        log::info!("ServiceBroker stopped");
        Ok(())
    }

    /// Create and register a new service
    pub async fn create_service(&mut self, service: Service) -> Result<Arc<Service>> {
        let service_arc = Arc::new(service);

        // Register service in the registry
        self.registry.register_local_service(&service_arc).await?;

        // Add to local services
        self.services.write().push(service_arc.clone());

        // Register action handlers
        for (action_name, handler) in &service_arc.schema.actions {
            let full_action_name = format!("{}.{}", service_arc.name, action_name);
            self.call_handlers.insert(full_action_name, handler.clone());
        }

        // Start service if broker is already started
        if *self.started.read() {
            self.start_service(&service_arc).await?;
        }

        log::info!("Service '{}' created", service_arc.name);
        Ok(service_arc)
    }

    /// Call an action
    pub async fn call(&self, action_name: &str, params: Value) -> Result<Value> {
        self.call_with_options(action_name, params, CallOptions::default())
            .await
    }

    /// Call an action with options
    pub async fn call_with_options(
        &self,
        action_name: &str,
        params: Value,
        options: CallOptions,
    ) -> Result<Value> {
        // Create context
        let context = Context::new(
            self.node_id.clone(),
            action_name.to_string(),
            params,
            options.meta.unwrap_or_default(),
        );

        // Find action handler
        if let Some(handler) = self.call_handlers.get(action_name) {
            // Local call
            handler(context)
        } else {
            // Remote call through transport
            if let Some(transport) = &self.transport {
                transport.call(action_name, context).await
            } else {
                Err(PenoosError::ServiceNotFound {
                    service: action_name.to_string(),
                })
            }
        }
    }

    /// Multiple call - call multiple actions
    pub async fn mcall(&self, def: MCallDefinition) -> Result<Value> {
        match def {
            MCallDefinition::Array(calls) => {
                let mut results = Vec::new();
                for call in calls {
                    let result = self
                        .call_with_options(
                            &call.action,
                            call.params,
                            call.options.unwrap_or_default(),
                        )
                        .await?;
                    results.push(result);
                }
                Ok(serde_json::to_value(results)?)
            }
            MCallDefinition::Object(calls) => {
                let mut results = HashMap::new();
                for (key, call) in calls {
                    let result = self
                        .call_with_options(
                            &call.action,
                            call.params,
                            call.options.unwrap_or_default(),
                        )
                        .await?;
                    results.insert(key, result);
                }
                Ok(serde_json::to_value(results)?)
            }
        }
    }

    /// Emit an event to local services
    pub async fn emit(&self, event_name: &str, payload: Value) -> Result<()> {
        // Find local event handlers
        let services = self.services.read();
        for service in services.iter() {
            if let Some(handler) = service.schema.events.get(event_name) {
                let context = Context::new(
                    self.node_id.clone(),
                    event_name.to_string(),
                    payload.clone(),
                    HashMap::new(),
                );

                // Execute handler in background
                let handler = handler.clone();
                let event_name_owned = event_name.to_string();
                tokio::spawn(async move {
                    if let Err(e) = handler(context) {
                        log::error!("Error in event handler '{}': {}", event_name_owned, e);
                    }
                });
            }
        }
        Ok(())
    }

    /// Broadcast an event to all services (local and remote)
    pub async fn broadcast(&self, event_name: &str, payload: Value) -> Result<()> {
        // Emit to local services
        self.emit(event_name, payload.clone()).await?;

        // Broadcast to remote services via transport
        if let Some(transport) = &self.transport {
            transport.broadcast(event_name, payload).await?;
        }

        Ok(())
    }

    /// Wait for services to be available
    pub async fn wait_for_services(
        &self,
        service_names: &[&str],
        timeout: Option<Duration>,
    ) -> Result<()> {
        let timeout = timeout.unwrap_or(Duration::from_secs(30));
        let start = std::time::Instant::now();

        loop {
            let mut all_available = true;
            for service_name in service_names {
                if !self.registry.has_service(service_name).await {
                    all_available = false;
                    break;
                }
            }

            if all_available {
                return Ok(());
            }

            if start.elapsed() >= timeout {
                return Err(PenoosError::Timeout {
                    action: format!("waiting for services: {:?}", service_names),
                    timeout: timeout.as_millis() as u64,
                });
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Get local service by name
    pub fn get_local_service(&self, name: &str) -> Option<Arc<Service>> {
        let services = self.services.read();
        services.iter().find(|s| s.name == name).cloned()
    }

    /// Check if broker is started
    pub fn is_started(&self) -> bool {
        *self.started.read()
    }

    /// Get node ID
    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }

    /// Get namespace
    pub fn get_namespace(&self) -> &str {
        &self.namespace
    }

    /// Start a specific service
    async fn start_service(&self, service: &Arc<Service>) -> Result<()> {
        log::info!("Starting service '{}'", service.name);

        // Call service started hook if it exists
        if let Some(started_hook) = &service.schema.started {
            started_hook()?;
        }

        log::info!("Service '{}' started", service.name);
        Ok(())
    }

    /// Stop a specific service
    async fn stop_service(&self, service: &Arc<Service>) -> Result<()> {
        log::info!("Stopping service '{}'", service.name);

        // Call service stopped hook if it exists
        if let Some(stopped_hook) = &service.schema.stopped {
            stopped_hook()?;
        }

        log::info!("Service '{}' stopped", service.name);
        Ok(())
    }
}
