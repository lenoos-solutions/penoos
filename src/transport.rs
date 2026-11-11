//! Transport layer abstraction for Penoos communication

use async_trait::async_trait;
use serde_json::Value;

use crate::{context::Context, errors::Result};

/// Transport layer trait for communication between nodes
#[async_trait]
pub trait Transport {
    /// Connect to the transport
    async fn connect(&self) -> Result<()>;

    /// Disconnect from the transport
    async fn disconnect(&self) -> Result<()>;

    /// Call a remote action
    async fn call(&self, action_name: &str, context: Context) -> Result<Value>;

    /// Broadcast an event to all nodes
    async fn broadcast(&self, event_name: &str, payload: Value) -> Result<()>;

    /// Send a packet to a specific node
    async fn send_to_node(&self, node_id: &str, packet: TransportPacket) -> Result<()>;

    /// Check if transport is connected
    fn is_connected(&self) -> bool;
}

/// Transport packet types
#[derive(Debug, Clone)]
pub enum TransportPacket {
    /// Request packet for action calls
    Request {
        id: String,
        action: String,
        params: Value,
        meta: std::collections::HashMap<String, Value>,
        timeout: Option<u64>,
        level: u32,
        caller: Option<String>,
        request_id: String,
    },
    /// Response packet for action calls
    Response {
        id: String,
        success: bool,
        data: Option<Value>,
        error: Option<String>,
        meta: std::collections::HashMap<String, Value>,
    },
    /// Event packet for broadcasting
    Event {
        event: String,
        data: Value,
        groups: Option<Vec<String>>,
        broadcast: bool,
    },
    /// Discover packet for service discovery
    Discover { sender: String },
    /// Info packet for node information
    Info {
        sender: String,
        services: Vec<ServiceInfo>,
        config: NodeConfig,
        metadata: std::collections::HashMap<String, Value>,
    },
    /// Heartbeat packet
    Heartbeat {
        sender: String,
        cpu: Option<f64>,
        memory: Option<f64>,
    },
    /// Disconnect packet
    Disconnect { sender: String, reason: String },
    /// Ping packet for connectivity check
    Ping { sender: String, time: u64 },
    /// Pong packet response to ping
    Pong {
        sender: String,
        time: u64,
        arrived: u64,
    },
}

/// Service information for transport
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub version: Option<String>,
    pub settings: std::collections::HashMap<String, Value>,
    pub metadata: std::collections::HashMap<String, Value>,
    pub actions: std::collections::HashMap<String, ActionInfo>,
    pub events: std::collections::HashMap<String, EventInfo>,
}

/// Action information
#[derive(Debug, Clone)]
pub struct ActionInfo {
    pub name: String,
    pub cache: bool,
    pub params: Option<Value>,
}

/// Event information
#[derive(Debug, Clone)]
pub struct EventInfo {
    pub name: String,
    pub group: Option<String>,
}

/// Node configuration
#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub heartbeat_interval: u64,
    pub heartbeat_timeout: u64,
    pub max_call_level: u32,
    pub request_timeout: u64,
}

/// Dummy transport for testing and local-only scenarios
pub struct LocalTransport {
    connected: std::sync::atomic::AtomicBool,
}

impl LocalTransport {
    pub fn new() -> Self {
        Self {
            connected: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl Default for LocalTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for LocalTransport {
    async fn connect(&self) -> Result<()> {
        self.connected
            .store(true, std::sync::atomic::Ordering::Relaxed);
        log::info!("Local transport connected");
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        self.connected
            .store(false, std::sync::atomic::Ordering::Relaxed);
        log::info!("Local transport disconnected");
        Ok(())
    }

    async fn call(&self, action_name: &str, _context: Context) -> Result<Value> {
        log::warn!("Local transport cannot call remote action: {}", action_name);
        Err(crate::errors::PenoosError::ServiceNotFound {
            service: action_name.to_string(),
        })
    }

    async fn broadcast(&self, event_name: &str, _payload: Value) -> Result<()> {
        log::debug!("Local transport broadcast: {}", event_name);
        // Local transport doesn't actually broadcast anywhere
        Ok(())
    }

    async fn send_to_node(&self, node_id: &str, _packet: TransportPacket) -> Result<()> {
        log::debug!("Local transport send to node: {}", node_id);
        // Local transport doesn't send to other nodes
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// TCP transport implementation
#[cfg(feature = "tcp-transport")]
pub mod tcp {
    use super::*;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    pub struct TcpTransport {
        address: SocketAddr,
        connected: std::sync::atomic::AtomicBool,
    }

    impl TcpTransport {
        pub fn new(address: SocketAddr) -> Self {
            Self {
                address,
                connected: std::sync::atomic::AtomicBool::new(false),
            }
        }
    }

    #[async_trait]
    impl Transport for TcpTransport {
        async fn connect(&self) -> Result<()> {
            let _listener = TcpListener::bind(self.address).await?;
            self.connected
                .store(true, std::sync::atomic::Ordering::Relaxed);
            log::info!("TCP transport connected on {}", self.address);
            Ok(())
        }

        async fn disconnect(&self) -> Result<()> {
            self.connected
                .store(false, std::sync::atomic::Ordering::Relaxed);
            log::info!("TCP transport disconnected");
            Ok(())
        }

        async fn call(&self, _action_name: &str, _context: Context) -> Result<Value> {
            // TODO: Implement TCP-based remote calls
            todo!("TCP transport call not implemented")
        }

        async fn broadcast(&self, _event_name: &str, _payload: Value) -> Result<()> {
            // TODO: Implement TCP-based broadcasting
            todo!("TCP transport broadcast not implemented")
        }

        async fn send_to_node(&self, _node_id: &str, _packet: TransportPacket) -> Result<()> {
            // TODO: Implement TCP packet sending
            todo!("TCP transport send_to_node not implemented")
        }

        fn is_connected(&self) -> bool {
            self.connected.load(std::sync::atomic::Ordering::Relaxed)
        }
    }
}

/// NATS transport implementation
#[cfg(feature = "nats-transport")]
pub mod nats {
    use super::*;

    pub struct NatsTransport {
        url: String,
        connected: std::sync::atomic::AtomicBool,
    }

    impl NatsTransport {
        pub fn new(url: &str) -> Self {
            Self {
                url: url.to_string(),
                connected: std::sync::atomic::AtomicBool::new(false),
            }
        }
    }

    #[async_trait]
    impl Transport for NatsTransport {
        async fn connect(&self) -> Result<()> {
            // TODO: Connect to NATS server
            self.connected
                .store(true, std::sync::atomic::Ordering::Relaxed);
            log::info!("NATS transport connected to {}", self.url);
            Ok(())
        }

        async fn disconnect(&self) -> Result<()> {
            self.connected
                .store(false, std::sync::atomic::Ordering::Relaxed);
            log::info!("NATS transport disconnected");
            Ok(())
        }

        async fn call(&self, _action_name: &str, _context: Context) -> Result<Value> {
            // TODO: Implement NATS-based remote calls
            todo!("NATS transport call not implemented")
        }

        async fn broadcast(&self, _event_name: &str, _payload: Value) -> Result<()> {
            // TODO: Implement NATS-based broadcasting
            todo!("NATS transport broadcast not implemented")
        }

        async fn send_to_node(&self, _node_id: &str, _packet: TransportPacket) -> Result<()> {
            // TODO: Implement NATS packet sending
            todo!("NATS transport send_to_node not implemented")
        }

        fn is_connected(&self) -> bool {
            self.connected.load(std::sync::atomic::Ordering::Relaxed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_transport() {
        let transport = LocalTransport::new();

        assert!(!transport.is_connected());

        transport.connect().await.unwrap();
        assert!(transport.is_connected());

        transport.disconnect().await.unwrap();
        assert!(!transport.is_connected());
    }
}
