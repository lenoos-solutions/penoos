//! Load balancing and routing strategies for Penoos

pub mod cpu_usage;
pub mod latency;
pub mod prefer_local;
pub mod random;
pub mod round_robin;

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::errors::Result;

/// Load balancing strategy trait
#[async_trait]
pub trait LoadBalancingStrategy: Send + Sync {
    /// Select the best endpoint from available options
    async fn select(&self, endpoints: &[ServiceEndpoint]) -> Result<Option<ServiceEndpoint>>;

    /// Get strategy name
    fn name(&self) -> &'static str;

    /// Reset strategy state (for stateful strategies like round-robin)
    async fn reset(&self);
}

/// Service endpoint information for load balancing
#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    pub service_name: String,
    pub node_id: String,
    pub is_local: bool,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
    pub response_time: Option<f64>,
    pub available: bool,
    pub metadata: HashMap<String, Value>,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout: std::time::Duration,
    pub reset_timeout: std::time::Duration,
    pub half_open_max_calls: u32,
}

/// Circuit breaker for fault tolerance
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<std::time::Instant>,
    half_open_calls: u32,
}

impl ServiceEndpoint {
    pub fn new(service_name: String, node_id: String, is_local: bool) -> Self {
        Self {
            service_name,
            node_id,
            is_local,
            cpu_usage: None,
            memory_usage: None,
            response_time: None,
            available: true,
            metadata: HashMap::new(),
        }
    }

    pub fn with_cpu_usage(mut self, cpu_usage: f64) -> Self {
        self.cpu_usage = Some(cpu_usage);
        self
    }

    pub fn with_memory_usage(mut self, memory_usage: f64) -> Self {
        self.memory_usage = Some(memory_usage);
        self
    }

    pub fn with_response_time(mut self, response_time: f64) -> Self {
        self.response_time = Some(response_time);
        self
    }

    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn set_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: std::time::Duration::from_secs(60),
            reset_timeout: std::time::Duration::from_secs(30),
            half_open_max_calls: 3,
        }
    }
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            half_open_calls: 0,
        }
    }

    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.config.reset_timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.half_open_calls = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => self.half_open_calls < self.config.half_open_max_calls,
        }
    }

    pub fn record_success(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count = 0;
            }
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.half_open_max_calls {
                    self.state = CircuitBreakerState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    self.half_open_calls = 0;
                }
            }
            CircuitBreakerState::Open => {
                // Should not happen
            }
        }
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(std::time::Instant::now());

        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open;
                self.half_open_calls = 0;
                self.success_count = 0;
            }
            CircuitBreakerState::Open => {
                // Already open
            }
        }
    }

    pub fn get_state(&self) -> &CircuitBreakerState {
        &self.state
    }

    pub fn get_stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.state.clone(),
            failure_count: self.failure_count,
            success_count: self.success_count,
            half_open_calls: self.half_open_calls,
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub success_count: u32,
    pub half_open_calls: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_endpoint_builder() {
        let endpoint = ServiceEndpoint::new("test".to_string(), "node1".to_string(), false)
            .with_cpu_usage(50.0)
            .with_response_time(100.0)
            .with_metadata("version".to_string(), serde_json::json!("1.0"));

        assert_eq!(endpoint.service_name, "test");
        assert_eq!(endpoint.node_id, "node1");
        assert!(!endpoint.is_local);
        assert_eq!(endpoint.cpu_usage, Some(50.0));
        assert_eq!(endpoint.response_time, Some(100.0));
        assert!(endpoint.metadata.contains_key("version"));
    }

    #[test]
    fn test_circuit_breaker_basic() {
        let config = CircuitBreakerConfig::default();
        let mut cb = CircuitBreaker::new(config);

        // Initially closed
        assert_eq!(cb.get_state(), &CircuitBreakerState::Closed);
        assert!(cb.can_execute());

        // Record failures up to threshold
        for _ in 0..4 {
            cb.record_failure();
            assert_eq!(cb.get_state(), &CircuitBreakerState::Closed);
        }

        // One more failure should open the circuit
        cb.record_failure();
        assert_eq!(cb.get_state(), &CircuitBreakerState::Open);
        assert!(!cb.can_execute());
    }

    #[test]
    fn test_circuit_breaker_half_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            reset_timeout: std::time::Duration::from_millis(1),
            ..Default::default()
        };
        let mut cb = CircuitBreaker::new(config);

        // Open the circuit
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.get_state(), &CircuitBreakerState::Open);

        // Wait for reset timeout
        std::thread::sleep(std::time::Duration::from_millis(2));

        // Should transition to half-open
        assert!(cb.can_execute());
        assert_eq!(cb.get_state(), &CircuitBreakerState::HalfOpen);

        // Record success should close the circuit after enough successes
        for _ in 0..3 {
            cb.record_success();
        }
        assert_eq!(cb.get_state(), &CircuitBreakerState::Closed);
    }
}
