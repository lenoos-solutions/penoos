//! Prefer local load balancing strategy

use async_trait::async_trait;

use super::{LoadBalancingStrategy, ServiceEndpoint};
use crate::errors::Result;

/// Prefer local load balancing strategy
/// Prioritizes local endpoints over remote ones
pub struct PreferLocalStrategy;

impl PreferLocalStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LoadBalancingStrategy for PreferLocalStrategy {
    async fn select(&self, endpoints: &[ServiceEndpoint]) -> Result<Option<ServiceEndpoint>> {
        let available_endpoints: Vec<&ServiceEndpoint> =
            endpoints.iter().filter(|e| e.available).collect();

        if available_endpoints.is_empty() {
            return Ok(None);
        }

        // First, try to find a local endpoint
        if let Some(local_endpoint) = available_endpoints.iter().find(|e| e.is_local) {
            return Ok(Some((*local_endpoint).clone()));
        }

        // If no local endpoints, select the first remote endpoint
        Ok(Some(available_endpoints[0].clone()))
    }

    fn name(&self) -> &'static str {
        "prefer-local"
    }

    async fn reset(&self) {
        // Prefer local strategy is stateless, nothing to reset
    }
}

impl Default for PreferLocalStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prefer_local_strategy_with_local() {
        let strategy = PreferLocalStrategy::new();

        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false),
            ServiceEndpoint::new("test".to_string(), "local".to_string(), true),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false),
        ];

        // Should prefer local endpoint
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected.node_id, "local");
        assert!(selected.is_local);
    }

    #[tokio::test]
    async fn test_prefer_local_strategy_no_local() {
        let strategy = PreferLocalStrategy::new();

        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false),
            ServiceEndpoint::new("test".to_string(), "node3".to_string(), false),
        ];

        // Should select first remote endpoint
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected.node_id, "node1");
        assert!(!selected.is_local);
    }

    #[tokio::test]
    async fn test_prefer_local_with_unavailable_local() {
        let strategy = PreferLocalStrategy::new();

        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false),
            ServiceEndpoint::new("test".to_string(), "local".to_string(), true)
                .set_available(false),
            ServiceEndpoint::new("test".to_string(), "node2".to_string(), false),
        ];

        // Should select first available remote endpoint since local is unavailable
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected.node_id, "node1");
        assert!(!selected.is_local);
    }

    #[tokio::test]
    async fn test_prefer_local_multiple_local() {
        let strategy = PreferLocalStrategy::new();

        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "remote1".to_string(), false),
            ServiceEndpoint::new("test".to_string(), "local1".to_string(), true),
            ServiceEndpoint::new("test".to_string(), "local2".to_string(), true),
            ServiceEndpoint::new("test".to_string(), "remote2".to_string(), false),
        ];

        // Should select first local endpoint
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected.node_id, "local1");
        assert!(selected.is_local);
    }

    #[tokio::test]
    async fn test_prefer_local_no_available_endpoints() {
        let strategy = PreferLocalStrategy::new();

        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "node1".to_string(), false)
                .set_available(false),
            ServiceEndpoint::new("test".to_string(), "local".to_string(), true)
                .set_available(false),
        ];

        let result = strategy.select(&endpoints).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_prefer_local_empty_endpoints() {
        let strategy = PreferLocalStrategy::new();

        let endpoints = vec![];

        let result = strategy.select(&endpoints).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_prefer_local_only_local() {
        let strategy = PreferLocalStrategy::new();

        let endpoints = vec![
            ServiceEndpoint::new("test".to_string(), "local1".to_string(), true),
            ServiceEndpoint::new("test".to_string(), "local2".to_string(), true),
        ];

        // Should select first local endpoint
        let selected = strategy.select(&endpoints).await.unwrap().unwrap();
        assert_eq!(selected.node_id, "local1");
        assert!(selected.is_local);
    }
}
