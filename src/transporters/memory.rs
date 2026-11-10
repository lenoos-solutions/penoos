//! In-memory transport for local development and testing

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use serde_json::Value;
use async_trait::async_trait;

#[allow(unused_imports)]
use super::{
    Transporter, EventHandler, RequestOptions, ServiceInfo, NodeInfo, 
    TransportStats, ActionInfo, EventInfo, ClientInfo
};
use crate::errors::{Result, PenoosError};

/// In-memory transport implementation
pub struct MemoryTransporter {
    node_id: String,
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    event_handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
    event_sender: broadcast::Sender<EventMessage>,
    stats: Arc<RwLock<TransportStats>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct EventMessage {
    event: String,
    payload: Value,
    sender: String,
    groups: Option<Vec<String>>,
}

impl MemoryTransporter {
    /// Create a new memory transporter
    pub fn new(node_id: String) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            node_id,
            services: Arc::new(RwLock::new(HashMap::new())),
            nodes: Arc::new(RwLock::new(HashMap::new())),
            event_handlers: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            stats: Arc::new(RwLock::new(TransportStats::default())),
        }
    }
    
    /// Register a service
    pub async fn register_service(&self, service: ServiceInfo) -> Result<()> {
        let mut services = self.services.write().await;
        services.insert(service.name.clone(), service);
        Ok(())
    }
    
    /// Unregister a service
    pub async fn unregister_service(&self, service_name: &str) -> Result<()> {
        let mut services = self.services.write().await;
        services.remove(service_name);
        Ok(())
    }
    
    /// Start event listener
    pub async fn start_event_listener(&self) -> Result<()> {
        let mut receiver = self.event_sender.subscribe();
        let handlers = self.event_handlers.clone();
        
        tokio::spawn(async move {
            while let Ok(event_msg) = receiver.recv().await {
                let handlers_map = handlers.read().await;
                
                if let Some(event_handlers) = handlers_map.get(&event_msg.event) {
                    for handler in event_handlers {
                        if let Err(e) = handler.handle(&event_msg.event, event_msg.payload.clone(), &event_msg.sender).await {
                            log::error!("Error handling event '{}': {}", event_msg.event, e);
                        }
                    }
                }
                
                // Handle wildcard events
                if let Some(wildcard_handlers) = handlers_map.get("*") {
                    for handler in wildcard_handlers {
                        if let Err(e) = handler.handle(&event_msg.event, event_msg.payload.clone(), &event_msg.sender).await {
                            log::error!("Error handling wildcard event '{}': {}", event_msg.event, e);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
}

#[async_trait]
impl Transporter for MemoryTransporter {
    async fn request(&self, target: &str, action: &str, params: Value, options: RequestOptions) -> Result<Value> {
        let services = self.services.read().await;
        
        // Find the target service
        let service = services.get(target)
            .ok_or_else(|| PenoosError::ServiceNotFound {
                service: target.to_string(),
            })?;
        
        // Check if the action exists
        let action_exists = service.actions.iter()
            .any(|a| a.name == action);
            
        if !action_exists {
            return Err(PenoosError::ActionNotFound {
                action: format!("{}.{}", target, action),
            });
        }
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.packets_sent += 1;
        stats.bytes_sent += serde_json::to_string(&params).unwrap_or_default().len() as u64;
        
        // For memory transport, we simulate a successful response
        // In a real implementation, this would forward the request to the actual service
        let response = serde_json::json!({
            "success": true,
            "action": action,
            "params": params,
            "node_id": self.node_id,
            "request_id": options.request_id
        });
        
        stats.packets_received += 1;
        stats.bytes_received += serde_json::to_string(&response).unwrap_or_default().len() as u64;
        
        Ok(response)
    }
    
    async fn publish(&self, event: &str, payload: Value, groups: Option<Vec<String>>) -> Result<()> {
        let event_msg = EventMessage {
            event: event.to_string(),
            payload,
            sender: self.node_id.clone(),
            groups,
        };
        
        // Send event to all subscribers
        if let Err(_) = self.event_sender.send(event_msg) {
            // No receivers, which is fine
        }
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.packets_sent += 1;
        
        Ok(())
    }
    
    async fn subscribe(&self, events: Vec<String>, handler: Box<dyn EventHandler>) -> Result<()> {
        let mut handlers = self.event_handlers.write().await;
        let handler: Arc<dyn EventHandler> = Arc::from(handler);
        
        for event in events {
            handlers.entry(event).or_insert_with(Vec::new).push(handler.clone());
        }
        
        Ok(())
    }
    
    async fn discover(&self) -> Result<Vec<ServiceInfo>> {
        let services = self.services.read().await;
        Ok(services.values().cloned().collect())
    }
    
    async fn register_node(&self, node_info: NodeInfo) -> Result<()> {
        let mut nodes = self.nodes.write().await;
        nodes.insert(node_info.id.clone(), node_info);
        
        let mut stats = self.stats.write().await;
        stats.connected_nodes = nodes.len() as u64;
        
        Ok(())
    }
    
    async fn unregister_node(&self, node_id: &str) -> Result<()> {
        let mut nodes = self.nodes.write().await;
        nodes.remove(node_id);
        
        let mut stats = self.stats.write().await;
        stats.connected_nodes = nodes.len() as u64;
        
        Ok(())
    }
    
    async fn heartbeat(&self, node_id: &str) -> Result<()> {
        let nodes = self.nodes.read().await;
        
        if !nodes.contains_key(node_id) {
            return Err(PenoosError::Custom {
                code: 404,
                message: format!("Node '{}' not found", node_id),
                data: None,
            });
        }
        
        // In a real implementation, this would update the node's last heartbeat time
        log::debug!("Heartbeat received from node: {}", node_id);
        
        Ok(())
    }
    
    async fn get_stats(&self) -> Result<TransportStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
    
    async fn is_connected(&self) -> Result<bool> {
        // Memory transport is always "connected"
        Ok(true)
    }
    
    async fn connect(&self) -> Result<()> {
        self.start_event_listener().await?;
        log::info!("Memory transport connected");
        Ok(())
    }
    
    async fn disconnect(&self) -> Result<()> {
        // Clear all data
        self.services.write().await.clear();
        self.nodes.write().await.clear();
        self.event_handlers.write().await.clear();
        
        log::info!("Memory transport disconnected");
        Ok(())
    }
}

/// Simple event handler implementation for testing
pub struct SimpleEventHandler<F>
where
    F: Fn(&str, Value, &str) -> Result<()> + Send + Sync,
{
    handler: F,
}

impl<F> SimpleEventHandler<F>
where
    F: Fn(&str, Value, &str) -> Result<()> + Send + Sync,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

#[async_trait]
impl<F> EventHandler for SimpleEventHandler<F>
where
    F: Fn(&str, Value, &str) -> Result<()> + Send + Sync,
{
    async fn handle(&self, event: &str, payload: Value, sender: &str) -> Result<()> {
        (self.handler)(event, payload, sender)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_memory_transport_basic() {
        let transport = MemoryTransporter::new("test-node".to_string());
        
        // Connect transport
        transport.connect().await.unwrap();
        
        // Register a test service
        let service = ServiceInfo {
            name: "test-service".to_string(),
            version: Some("1.0".to_string()),
            node_id: "test-node".to_string(),
            actions: vec![ActionInfo {
                name: "hello".to_string(),
                rest: None,
                cache: false,
                params: None,
            }],
            events: vec![EventInfo {
                name: "test.event".to_string(),
                group: None,
            }],
            settings: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        transport.register_service(service).await.unwrap();
        
        // Test discovery
        let services = transport.discover().await.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "test-service");
        
        // Test request
        let result = transport.request(
            "test-service",
            "hello",
            json!({"name": "world"}),
            RequestOptions::default(),
        ).await.unwrap();
        
        assert!(result.is_object());
        
        // Test stats
        let stats = transport.get_stats().await.unwrap();
        assert_eq!(stats.packets_sent, 1);
        assert_eq!(stats.packets_received, 1);
    }
    
    #[tokio::test]
    async fn test_memory_transport_events() {
        let transport = MemoryTransporter::new("test-node".to_string());
        transport.connect().await.unwrap();
        
        let received_events = Arc::new(RwLock::new(Vec::new()));
        let received_events_clone = received_events.clone();
        
        let handler = SimpleEventHandler::new(move |event, payload, sender| {
            let received = received_events_clone.clone();
            let event = event.to_string();
            let sender = sender.to_string();
            tokio::spawn(async move {
                received.write().await.push((event, payload, sender));
            });
            Ok(())
        });
        
        // Subscribe to events
        transport.subscribe(vec!["test.event".to_string()], Box::new(handler)).await.unwrap();
        
        // Publish an event
        transport.publish("test.event", json!({"message": "hello"}), None).await.unwrap();
        
        // Give some time for event processing
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        let events = received_events.read().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "test.event");
    }
    
    #[tokio::test]
    async fn test_memory_transport_node_management() {
        let transport = MemoryTransporter::new("test-node".to_string());
        
        let node_info = NodeInfo {
            id: "node-1".to_string(),
            hostname: "localhost".to_string(),
            ip_list: vec!["127.0.0.1".to_string()],
            port: Some(3000),
            client: ClientInfo::default(),
            config: HashMap::new(),
            metadata: HashMap::new(),
            services: vec![],
        };
        
        // Register node
        transport.register_node(node_info).await.unwrap();
        
        // Check stats
        let stats = transport.get_stats().await.unwrap();
        assert_eq!(stats.connected_nodes, 1);
        
        // Heartbeat
        transport.heartbeat("node-1").await.unwrap();
        
        // Unregister node
        transport.unregister_node("node-1").await.unwrap();
        
        let stats = transport.get_stats().await.unwrap();
        assert_eq!(stats.connected_nodes, 0);
    }
}
