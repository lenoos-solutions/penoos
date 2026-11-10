//! Round-robin load balancing strategy

use std::sync::atomic::{AtomicUsize, Ordering};
use async_trait::async_trait;

use super::{LoadBalancingStrategy, ServiceEndpoint};
use crate::errors::Result;

/// Round-robin load balancing strategy
pub struct RoundRobinStrategy {
    counter: AtomicUsize,
}

impl RoundRobinStrategy {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl LoadBalancingStrategy for RoundRobinStrategy {
    async fn select(&self, endpoints: &[ServiceEndpoint]) -> Result<Option<ServiceEndpoint>> {
        let available_endpoints: Vec<&ServiceEndpoint> = endpoints
            .iter()
            .filter(|e| e.available)
            .collect();
        
        if available_endpoints.is_empty() {
            return Ok(None);
        }
        
        let index = self.counter.fetch_add(1, Ordering::Relaxed) % available_endpoints.len();
        Ok(Some(available_endpoints[index].clone()))
    }
    
    fn name(&self) -> &'static str {
        "round-robin"
    }
    
    async fn reset(&self) {
        self.counter.store(0, Ordering::Relaxed);
    }
}

impl Default for RoundRobinStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_round_robin_strategy() {
        let strategy = RoundRobinStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false),
            ServiceEndpoint::new("test".to_string(), "node3".to_string(), false),
        ];
        
        // Test round-robin selection
        let selected1 = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected1.node_id, "node1");
        
        let selected2 = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected2.node_id, "node2");
        
        let selected3 = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected3.node_id, "node3");
        
        // Should wrap around
        let selected4 = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected4.node_id, "node1");
    }
    
    #[tokio::test]
    async fn test_round_robin_with_unavailable_endpoints() {
        let strategy = RoundRobinStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false).set_available(false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false),
            ServiceEndpoint::new("test".to_string(), "node3".to_string(), false).set_available(false),
        ];
        
        // Should only select available endpoints
        let selected1 = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected1.node_id, "node2");
        
        let selected2 = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected2.node_id, "node2");
    }
    
    #[tokio::test]
    async fn test_round_robin_no_available_endpoints() {
        let strategy = RoundRobinStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false).set_available(false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false).set_available(false),
        ];
        
        let result = strategy.select(&endpoints).await.unwrap();
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_round_robin_reset() {
        let strategy = RoundRobinStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false),
        ];
        
        // Select a few times
        strategy.select(&endpoints).await.unwrap();
        strategy.select(&endpoints).await.unwrap();
        
        // Reset should start from beginning
        strategy.reset().await;
        
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected.node_id, "node1");
    }
}
