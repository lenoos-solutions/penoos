//! Service registry for managing service discovery and load balancing

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::RwLock;

use crate::{
    errors::{PenoosError, Result},
    service::Service,
};

/// Service registry for managing local and remote services
pub struct ServiceRegistry {
    /// Local services registered on this node
    local_services: Arc<RwLock<HashMap<String, Arc<Service>>>>,
    /// Remote services discovered from other nodes
    remote_services: Arc<DashMap<String, RemoteServiceInfo>>,
    /// Node information
    nodes: Arc<DashMap<String, NodeInfo>>,
}

/// Information about a remote service
#[derive(Debug, Clone)]
pub struct RemoteServiceInfo {
    pub name: String,
    pub version: Option<String>,
    pub node_id: String,
    pub actions: Vec<String>,
    pub events: Vec<String>,
    pub settings: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Information about a node
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: String,
    pub available: bool,
    pub last_heartbeat: u64,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
    pub services: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self {
            local_services: Arc::new(RwLock::new(HashMap::new())),
            remote_services: Arc::new(DashMap::new()),
            nodes: Arc::new(DashMap::new()),
        }
    }

    /// Register a local service
    pub async fn register_local_service(&self, service: &Arc<Service>) -> Result<()> {
        let mut services = self.local_services.write();
        
        if services.contains_key(&service.name) {
            return Err(PenoosError::ServiceAlreadyExists {
                service: service.name.clone(),
            });
        }

        services.insert(service.name.clone(), service.clone());
        log::debug!("Registered local service: {}", service.name);
        Ok(())
    }

    /// Unregister a local service
    pub async fn unregister_local_service(&self, service_name: &str) -> Result<()> {
        let mut services = self.local_services.write();
        
        if services.remove(service_name).is_some() {
            log::debug!("Unregistered local service: {}", service_name);
            Ok(())
        } else {
            Err(PenoosError::ServiceNotFound {
                service: service_name.to_string(),
            })
        }
    }

    /// Register a remote service
    pub async fn register_remote_service(&self, service_info: RemoteServiceInfo) -> Result<()> {
        let key = format!("{}:{}", service_info.node_id, service_info.name);
        self.remote_services.insert(key, service_info.clone());
        log::debug!("Registered remote service: {} on node {}", service_info.name, service_info.node_id);
        Ok(())
    }

    /// Unregister a remote service
    pub async fn unregister_remote_service(&self, node_id: &str, service_name: &str) -> Result<()> {
        let key = format!("{}:{}", node_id, service_name);
        if self.remote_services.remove(&key).is_some() {
            log::debug!("Unregistered remote service: {} from node {}", service_name, node_id);
            Ok(())
        } else {
            Err(PenoosError::ServiceNotFound {
                service: service_name.to_string(),
            })
        }
    }

    /// Check if a service is available (local or remote)
    pub async fn has_service(&self, service_name: &str) -> bool {
        // Check local services
        if self.local_services.read().contains_key(service_name) {
            return true;
        }

        // Check remote services
        for entry in self.remote_services.iter() {
            if entry.value().name == service_name {
                return true;
            }
        }

        false
    }

    /// Get a local service
    pub fn get_local_service(&self, service_name: &str) -> Option<Arc<Service>> {
        self.local_services.read().get(service_name).cloned()
    }

    /// Get all local services
    pub fn get_local_services(&self) -> Vec<Arc<Service>> {
        self.local_services.read().values().cloned().collect()
    }

    /// Get remote services by name
    pub fn get_remote_services(&self, service_name: &str) -> Vec<RemoteServiceInfo> {
        self.remote_services
            .iter()
            .filter(|entry| entry.value().name == service_name)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get all remote services
    pub fn get_all_remote_services(&self) -> Vec<RemoteServiceInfo> {
        self.remote_services.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Register a node
    pub fn register_node(&self, node_info: NodeInfo) {
        self.nodes.insert(node_info.id.clone(), node_info);
    }

    /// Unregister a node
    pub fn unregister_node(&self, node_id: &str) {
        self.nodes.remove(node_id);
        
        // Remove all services from this node
        self.remote_services.retain(|key, _| !key.starts_with(&format!("{}:", node_id)));
    }

    /// Get node information
    pub fn get_node(&self, node_id: &str) -> Option<NodeInfo> {
        self.nodes.get(node_id).map(|entry| entry.value().clone())
    }

    /// Get all nodes
    pub fn get_all_nodes(&self) -> Vec<NodeInfo> {
        self.nodes.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Update node heartbeat
    pub fn update_node_heartbeat(&self, node_id: &str, timestamp: u64) {
        if let Some(mut node) = self.nodes.get_mut(node_id) {
            node.last_heartbeat = timestamp;
        }
    }

    /// Mark node as unavailable
    pub fn mark_node_unavailable(&self, node_id: &str) {
        if let Some(mut node) = self.nodes.get_mut(node_id) {
            node.available = false;
        }
    }

    /// Get service statistics
    pub fn get_statistics(&self) -> RegistryStatistics {
        let local_count = self.local_services.read().len();
        let remote_count = self.remote_services.len();
        let node_count = self.nodes.len();
        let available_nodes = self.nodes.iter().filter(|entry| entry.value().available).count();

        RegistryStatistics {
            local_services: local_count,
            remote_services: remote_count,
            total_nodes: node_count,
            available_nodes,
        }
    }

    /// Find services for load balancing
    pub fn find_services(&self, service_name: &str, strategy: LoadBalancingStrategy) -> Vec<ServiceEndpoint> {
        let mut endpoints = Vec::new();

        // Add local service if exists
        if let Some(service) = self.get_local_service(service_name) {
            endpoints.push(ServiceEndpoint {
                service_name: service.name.clone(),
                node_id: "local".to_string(),
                is_local: true,
                cpu_usage: None,
                response_time: None,
            });
        }

        // Add remote services
        for remote_service in self.get_remote_services(service_name) {
            if let Some(node) = self.get_node(&remote_service.node_id) {
                if node.available {
                    endpoints.push(ServiceEndpoint {
                        service_name: remote_service.name,
                        node_id: remote_service.node_id,
                        is_local: false,
                        cpu_usage: node.cpu_usage,
                        response_time: None, // TODO: Implement response time tracking
                    });
                }
            }
        }

        // Apply load balancing strategy
        self.apply_load_balancing_strategy(endpoints, strategy)
    }

    /// Apply load balancing strategy
    fn apply_load_balancing_strategy(&self, mut endpoints: Vec<ServiceEndpoint>, strategy: LoadBalancingStrategy) -> Vec<ServiceEndpoint> {
        match strategy {
            LoadBalancingStrategy::RoundRobin => {
                // TODO: Implement proper round-robin with state
                endpoints
            }
            LoadBalancingStrategy::Random => {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                endpoints.shuffle(&mut rng);
                endpoints
            }
            LoadBalancingStrategy::CpuUsage => {
                endpoints.sort_by(|a, b| {
                    let a_cpu = a.cpu_usage.unwrap_or(0.0);
                    let b_cpu = b.cpu_usage.unwrap_or(0.0);
                    a_cpu.partial_cmp(&b_cpu).unwrap_or(std::cmp::Ordering::Equal)
                });
                endpoints
            }
            LoadBalancingStrategy::Latency => {
                endpoints.sort_by(|a, b| {
                    let a_time = a.response_time.unwrap_or(f64::MAX);
                    let b_time = b.response_time.unwrap_or(f64::MAX);
                    a_time.partial_cmp(&b_time).unwrap_or(std::cmp::Ordering::Equal)
                });
                endpoints
            }
            LoadBalancingStrategy::PreferLocal => {
                endpoints.sort_by(|a, b| b.is_local.cmp(&a.is_local));
                endpoints
            }
        }
    }
}

/// Service endpoint for load balancing
#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    pub service_name: String,
    pub node_id: String,
    pub is_local: bool,
    pub cpu_usage: Option<f64>,
    pub response_time: Option<f64>,
}

/// Load balancing strategies
#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    Random,
    CpuUsage,
    Latency,
    PreferLocal,
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStatistics {
    pub local_services: usize,
    pub remote_services: usize,
    pub total_nodes: usize,
    pub available_nodes: usize,
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for LoadBalancingStrategy {
    fn default() -> Self {
        LoadBalancingStrategy::RoundRobin
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::Service;

    #[tokio::test]
    async fn test_service_registration() {
        let registry = ServiceRegistry::new();
        let service = Arc::new(Service::new("test-service"));

        // Register service
        registry.register_local_service(&service).await.unwrap();
        assert!(registry.has_service("test-service").await);

        // Try to register the same service again
        let result = registry.register_local_service(&service).await;
        assert!(result.is_err());

        // Unregister service
        registry.unregister_local_service("test-service").await.unwrap();
        assert!(!registry.has_service("test-service").await);
    }

    #[tokio::test]
    async fn test_remote_service_registration() {
        let registry = ServiceRegistry::new();
        
        let remote_service = RemoteServiceInfo {
            name: "remote-service".to_string(),
            version: Some("1.0".to_string()),
            node_id: "node-1".to_string(),
            actions: vec!["hello".to_string()],
            events: vec!["test".to_string()],
            settings: HashMap::new(),
            metadata: HashMap::new(),
        };

        registry.register_remote_service(remote_service).await.unwrap();
        assert!(registry.has_service("remote-service").await);

        let remote_services = registry.get_remote_services("remote-service");
        assert_eq!(remote_services.len(), 1);
        assert_eq!(remote_services[0].node_id, "node-1");
    }

    #[test]
    fn test_load_balancing_strategies() {
        let registry = ServiceRegistry::new();
        
        let endpoints = vec![
            ServiceEndpoint {
                service_name: "test".to_string(),
                node_id: "node1".to_string(),
                is_local: false,
                cpu_usage: Some(10.0),
                response_time: Some(100.0),
            },
            ServiceEndpoint {
                service_name: "test".to_string(),
                node_id: "local".to_string(),
                is_local: true,
                cpu_usage: Some(50.0),
                response_time: Some(50.0),
            },
        ];

        // Test prefer local strategy
        let result = registry.apply_load_balancing_strategy(endpoints.clone(), LoadBalancingStrategy::PreferLocal);
        assert!(result[0].is_local);

        // Test CPU usage strategy
        let result = registry.apply_load_balancing_strategy(endpoints.clone(), LoadBalancingStrategy::CpuUsage);
        assert_eq!(result[0].node_id, "node1"); // Lower CPU usage should come first
    }
}
