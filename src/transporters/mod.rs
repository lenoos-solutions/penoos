//! Transport implementations for Penoos

pub mod memory;
pub mod tcp;
pub mod nats;
pub mod redis;

use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use crate::errors::Result;

/// Transport layer trait for service communication
#[async_trait]
pub trait Transporter: Send + Sync {
    /// Send a request to a service
    async fn request(&self, target: &str, action: &str, params: Value, options: RequestOptions) -> Result<Value>;
    
    /// Publish an event
    async fn publish(&self, event: &str, payload: Value, groups: Option<Vec<String>>) -> Result<()>;
    
    /// Subscribe to events
    async fn subscribe(&self, events: Vec<String>, handler: Box<dyn EventHandler>) -> Result<()>;
    
    /// Discover services
    async fn discover(&self) -> Result<Vec<ServiceInfo>>;
    
    /// Register this node
    async fn register_node(&self, node_info: NodeInfo) -> Result<()>;
    
    /// Unregister this node
    async fn unregister_node(&self, node_id: &str) -> Result<()>;
    
    /// Send heartbeat
    async fn heartbeat(&self, node_id: &str) -> Result<()>;
    
    /// Get transport statistics
    async fn get_stats(&self) -> Result<TransportStats>;
    
    /// Check if transport is connected
    async fn is_connected(&self) -> Result<bool>;
    
    /// Connect to the transport
    async fn connect(&self) -> Result<()>;
    
    /// Disconnect from the transport
    async fn disconnect(&self) -> Result<()>;
}

/// Event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &str, payload: Value, sender: &str) -> Result<()>;
}

/// Request options
#[derive(Debug, Clone)]
pub struct RequestOptions {
    pub timeout: Option<std::time::Duration>,
    pub retries: Option<u32>,
    pub fallback_response: Option<Value>,
    pub node_id: Option<String>,
    pub caller_id: String,
    pub request_id: String,
    pub meta: HashMap<String, Value>,
}

/// Service information for discovery
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub version: Option<String>,
    pub node_id: String,
    pub actions: Vec<ActionInfo>,
    pub events: Vec<EventInfo>,
    pub settings: HashMap<String, Value>,
    pub metadata: HashMap<String, Value>,
}

/// Action information
#[derive(Debug, Clone)]
pub struct ActionInfo {
    pub name: String,
    pub rest: Option<String>,
    pub cache: bool,
    pub params: Option<Value>,
}

/// Event information
#[derive(Debug, Clone)]
pub struct EventInfo {
    pub name: String,
    pub group: Option<String>,
}

/// Node information
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: String,
    pub hostname: String,
    pub ip_list: Vec<String>,
    pub port: Option<u16>,
    pub client: ClientInfo,
    pub config: HashMap<String, Value>,
    pub metadata: HashMap<String, Value>,
    pub services: Vec<ServiceInfo>,
}

/// Client information
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub type_name: String,
    pub version: String,
    pub lang_version: String,
}

/// Transport statistics
#[derive(Debug, Clone)]
pub struct TransportStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub errors: u64,
    pub connected_nodes: u64,
}

impl Default for RequestOptions {
    fn default() -> Self {
        Self {
            timeout: Some(std::time::Duration::from_secs(30)),
            retries: Some(3),
            fallback_response: None,
            node_id: None,
            caller_id: "unknown".to_string(),
            request_id: Uuid::new_v4().to_string(),
            meta: HashMap::new(),
        }
    }
}

impl Default for TransportStats {
    fn default() -> Self {
        Self {
            packets_sent: 0,
            packets_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            errors: 0,
            connected_nodes: 0,
        }
    }
}

impl Default for ClientInfo {
    fn default() -> Self {
        Self {
            type_name: "penoos-rust".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            lang_version: "Rust 1.70+".to_string(),
        }
    }
}
