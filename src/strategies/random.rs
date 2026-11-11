//! Random load balancing strategy

use async_trait::async_trait;
use rand::seq::IndexedRandom;

use super::{LoadBalancingStrategy, ServiceEndpoint};
use crate::errors::Result;

/// Random load balancing strategy
pub struct RandomStrategy;

impl RandomStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LoadBalancingStrategy for RandomStrategy {
    async fn select(&self, endpoints: &[ServiceEndpoint]) -> Result<Option<ServiceEndpoint>> {
        let available_endpoints: Vec<&ServiceEndpoint> = endpoints
            .iter()
            .filter(|e| e.available)
            .collect();
        
        if available_endpoints.is_empty() {
            return Ok(None);
        }
        
        let mut rng = rand::rng();
        let selected = available_endpoints.choose(&mut rng);
        
        Ok(selected.map(|e| (*e).clone()))
    }
    
    fn name(&self) -> &'static str {
        "random"
    }
    
    async fn reset(&self) {
        // Random strategy is stateless, nothing to reset
    }
}

impl Default for RandomStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    
    #[tokio::test]
    async fn test_random_strategy() {
        let strategy = RandomStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false),
            ServiceEndpoint::new("test".to_string(), "node3".to_string(), false),
        ];
        
        // Test that random selection works
        let mut selected_nodes = HashSet::new();
        
        // Run many times to ensure we get different results
        for _ in 0..100 {
            let selected = strategy.select(&endpoints).await.unwrap().unwrap();
            selected_nodes.insert(selected.node_id);
        }
        
        // Should have selected from multiple nodes (with very high probability)
        assert!(selected_nodes.len() > 1);
    }
    
    #[tokio::test]
    async fn test_random_with_unavailable_endpoints() {
        let strategy = RandomStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false).set_available(false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false),
            ServiceEndpoint::new("test".to_string(), "node3".to_string(), false).set_available(false),
        ];
        
        // Should only select available endpoints
        for _ in 0..10 {
            let selected = strategy.select(&endpoints).await.unwrap().unwrap();
            assert_eq!(selected.node_id, "node2");
        }
    }
    
    #[tokio::test]
    async fn test_random_no_available_endpoints() {
        let strategy = RandomStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false).set_available(false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false).set_available(false),
        ];
        
        let result = strategy.select(&endpoints).await.unwrap();
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_random_empty_endpoints() {
        let strategy = RandomStrategy::new();
        
        let endpoints = vec![];
        
        let result = strategy.select(&endpoints).await.unwrap();
        assert!(result.is_none());
    }
}
