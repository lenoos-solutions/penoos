//! CPU usage-based load balancing strategy

use async_trait::async_trait;

use super::{LoadBalancingStrategy, ServiceEndpoint};
use crate::errors::Result;

/// CPU usage-based load balancing strategy
/// Selects the endpoint with the lowest CPU usage
pub struct CpuUsageStrategy;

impl CpuUsageStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LoadBalancingStrategy for CpuUsageStrategy {
    async fn select(&self, endpoints: &[ServiceEndpoint]) -> Result<Option<ServiceEndpoint>> {
        let available_endpoints: Vec<&ServiceEndpoint> = endpoints
            .iter()
            .filter(|e| e.available)
            .collect();
        
        if available_endpoints.is_empty() {
            return Ok(None);
        }
        
        // Find endpoint with lowest CPU usage
        let best_endpoint = available_endpoints
            .iter()
            .min_by(|a, b| {
                let a_cpu = a.cpu_usage.unwrap_or(0.0);
                let b_cpu = b.cpu_usage.unwrap_or(0.0);
                a_cpu.partial_cmp(&b_cpu).unwrap_or(std::cmp::Ordering::Equal)
            });
        
        Ok(best_endpoint.map(|e| (*e).clone()))
    }
    
    fn name(&self) -> &'static str {
        "cpu-usage"
    }
    
    async fn reset(&self) {
        // CPU usage strategy is stateless, nothing to reset
    }
}

impl Default for CpuUsageStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cpu_usage_strategy() {
        let strategy = CpuUsageStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false)
                .with_cpu_usage(80.0),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false)
                .with_cpu_usage(20.0),
            ServiceEndpoint::new("test".to_string(), "node3".to_string(), false)
                .with_cpu_usage(50.0),
        ];
        
        // Should select node with lowest CPU usage
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected.node_id, "node2");
        assert_eq!(selected.cpu_usage, Some(20.0));
    }
    
    #[tokio::test]
    async fn test_cpu_usage_with_missing_data() {
        let strategy = CpuUsageStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false)
                .with_cpu_usage(80.0),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false),
            // node2 has no CPU usage data (will default to 0.0)
            ServiceEndpoint::new("test".to_string(), "node3".to_string(), false)
                .with_cpu_usage(50.0),
        ];
        
        // Should select node2 (default 0.0 CPU usage)
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected.node_id, "node2");
    }
    
    #[tokio::test]
    async fn test_cpu_usage_with_unavailable_endpoints() {
        let strategy = CpuUsageStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false)
                .with_cpu_usage(10.0)
                .set_available(false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false)
                .with_cpu_usage(90.0),
            ServiceEndpoint::new("test".to_string(), "node3".to_string(), false)
                .with_cpu_usage(50.0),
        ];
        
        // Should select node3 (lowest among available)
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected.node_id, "node3");
    }
    
    #[tokio::test]
    async fn test_cpu_usage_equal_values() {
        let strategy = CpuUsageStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false)
                .with_cpu_usage(50.0),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false)
                .with_cpu_usage(50.0),
        ];
        
        // Should select one of them (deterministic based on order)
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert!(selected.node_id == "node1" || selected.node_id == "node2");
    }
    
    #[tokio::test]
    async fn test_cpu_usage_no_available_endpoints() {
        let strategy = CpuUsageStrategy::new();
        
        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false)
                .set_available(false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false)
                .set_available(false),
        ];
        
        let result = strategy.select(&endpoints).await.unwrap();
        assert!(result.is_none());
    }
}
