//! Latency-based load balancing strategy

use async_trait::async_trait;

use super::{LoadBalancingStrategy, ServiceEndpoint};
use crate::errors::Result;

/// Latency-based load balancing strategy
/// Selects the endpoint with the lowest response time
pub struct LatencyStrategy;

impl LatencyStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LoadBalancingStrategy for LatencyStrategy {
    async fn select(&self, endpoints: &[ServiceEndpoint]) -> Result<Option<ServiceEndpoint>> {
        let available_endpoints: Vec<&ServiceEndpoint> = endpoints
            .iter()
            .filter(|e| e.available)
            .collect();
        
        if available_endpoints.is_empty() {
            return Ok(None);
        }
        
        // Find endpoint with lowest response time
        let best_endpoint = available_endpoints
            .iter()
            .min_by(|a, b| {
                let a_latency = a.response_time.unwrap_or(f64::MAX);
                let b_latency = b.response_time.unwrap_or(f64::MAX);
                a_latency.partial_cmp(&b_latency).unwrap_or(std::cmp::Ordering::Equal)
            });
        
        Ok(best_endpoint.map(|e| (*e).clone()))
    }
    
    fn name(&self) -> &'static str {
        "latency"
    }
    
    async fn reset(&self) {
        // Latency strategy is stateless, nothing to reset
    }
}

impl Default for LatencyStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_latency_strategy() {
        let strategy = LatencyStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false)
                .with_response_time(200.0),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false)
                .with_response_time(50.0),
            ServiceEndpoint::new("test".to_string(), "node3".to_string(), false)
                .with_response_time(150.0),
        ];
        
        // Should select node with lowest latency
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected.node_id, "node2");
        assert_eq!(selected.response_time, Some(50.0));
    }
    
    #[tokio::test]
    async fn test_latency_with_missing_data() {
        let strategy = LatencyStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false)
                .with_response_time(100.0),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false),
            // node2 has no response time data (will default to f64::MAX)
            ServiceEndpoint::new("test".to_string(), "node3".to_string(), false)
                .with_response_time(200.0),
        ];
        
        // Should select node1 (lowest actual latency)
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected.node_id, "node1");
    }
    
    #[tokio::test]
    async fn test_latency_with_unavailable_endpoints() {
        let strategy = LatencyStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false)
                .with_response_time(10.0)
                .set_available(false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false)
                .with_response_time(200.0),
            ServiceEndpoint::new("test".to_string(), "node3".to_string(), false)
                .with_response_time(100.0),
        ];
        
        // Should select node3 (lowest among available)
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected.node_id, "node3");
    }
    
    #[tokio::test]
    async fn test_latency_equal_values() {
        let strategy = LatencyStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false)
                .with_response_time(100.0),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false)
                .with_response_time(100.0),
        ];
        
        // Should select one of them (deterministic based on order)
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert!(selected.node_id == "node1" || selected.node_id == "node2");
    }
    
    #[tokio::test]
    async fn test_latency_no_available_endpoints() {
        let strategy = LatencyStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false)
                .set_available(false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false)
                .set_available(false),
        ];
        
        let result = strategy.select(&endpoints).await.unwrap();
        assert!(result.is_none());
    }
    
    #[tokio::test]
    async fn test_latency_all_missing_data() {
        let strategy = LatencyStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false),
        ];
        
        // Should select one of them since all have same default value
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert!(selected.node_id == "node1" || selected.node_id == "node2");
    }
}
