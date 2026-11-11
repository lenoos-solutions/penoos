//! Metrics collection and reporting

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;

/// Metrics registry for collecting and managing metrics
pub struct MetricsRegistry {
    metrics: Arc<DashMap<String, MetricValue>>,
    enabled: bool,
}

/// Metric value types
#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram { values: Vec<f64>, buckets: Vec<f64> },
    Info(String),
}

/// Metric types
#[derive(Debug, Clone)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Info,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(DashMap::new()),
            enabled: true,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn register_counter(&self, name: &str, description: &str) {
        if !self.enabled {
            return;
        }
        self.metrics
            .insert(name.to_string(), MetricValue::Counter(0));
        log::debug!("Registered counter metric: {} - {}", name, description);
    }

    pub fn register_gauge(&self, name: &str, description: &str) {
        if !self.enabled {
            return;
        }
        self.metrics
            .insert(name.to_string(), MetricValue::Gauge(0.0));
        log::debug!("Registered gauge metric: {} - {}", name, description);
    }

    pub fn register_histogram(&self, name: &str, description: &str, buckets: Vec<f64>) {
        if !self.enabled {
            return;
        }
        self.metrics.insert(
            name.to_string(),
            MetricValue::Histogram {
                values: Vec::new(),
                buckets,
            },
        );
        log::debug!("Registered histogram metric: {} - {}", name, description);
    }

    pub fn register_info(&self, name: &str, description: &str) {
        if !self.enabled {
            return;
        }
        self.metrics
            .insert(name.to_string(), MetricValue::Info(String::new()));
        log::debug!("Registered info metric: {} - {}", name, description);
    }

    pub fn increment_counter(&self, name: &str, value: u64) {
        if !self.enabled {
            return;
        }

        if let Some(mut metric) = self.metrics.get_mut(name)
            && let MetricValue::Counter(counter) = metric.value_mut()
        {
            *counter += value;
        }
    }

    pub fn set_gauge(&self, name: &str, value: f64) {
        if !self.enabled {
            return;
        }

        if let Some(mut metric) = self.metrics.get_mut(name)
            && let MetricValue::Gauge(gauge) = metric.value_mut()
        {
            *gauge = value;
        }
    }

    pub fn observe_histogram(&self, name: &str, value: f64) {
        if !self.enabled {
            return;
        }

        if let Some(mut metric) = self.metrics.get_mut(name)
            && let MetricValue::Histogram { values, .. } = metric.value_mut()
        {
            values.push(value);
        }
    }

    pub fn set_info(&self, name: &str, value: &str) {
        if !self.enabled {
            return;
        }

        if let Some(mut metric) = self.metrics.get_mut(name)
            && let MetricValue::Info(info) = metric.value_mut()
        {
            *info = value.to_string();
        }
    }

    pub fn get_metric(&self, name: &str) -> Option<MetricValue> {
        self.metrics.get(name).map(|entry| entry.value().clone())
    }

    pub fn get_all_metrics(&self) -> HashMap<String, MetricValue> {
        self.metrics
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    pub fn clear(&self) {
        self.metrics.clear();
    }

    pub fn remove_metric(&self, name: &str) -> bool {
        self.metrics.remove(name).is_some()
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer for measuring execution time
pub struct Timer {
    start: Instant,
    name: String,
    registry: Arc<MetricsRegistry>,
}

impl Timer {
    pub fn new(name: &str, registry: Arc<MetricsRegistry>) -> Self {
        Self {
            start: Instant::now(),
            name: name.to_string(),
            registry,
        }
    }

    pub fn stop(self) -> Duration {
        let duration = self.start.elapsed();
        self.registry
            .observe_histogram(&self.name, duration.as_secs_f64());
        duration
    }
}

/// Metrics constants
pub mod constants {
    pub const MOLECULER_BROKER_STARTED: &str = "penoos.broker.started";
    pub const MOLECULER_BROKER_LOCAL_SERVICES_TOTAL: &str = "penoos.broker.local.services.total";
    pub const MOLECULER_BROKER_MIDDLEWARES_TOTAL: &str = "penoos.broker.middlewares.total";
    pub const MOLECULER_REQUEST_TOTAL: &str = "penoos.request.total";
    pub const MOLECULER_REQUEST_ACTIVE: &str = "penoos.request.active";
    pub const MOLECULER_REQUEST_TIME: &str = "penoos.request.time";
    pub const MOLECULER_REQUEST_LEVELS: &str = "penoos.request.levels";
    pub const MOLECULER_EVENT_EMIT_TOTAL: &str = "penoos.event.emit.total";
    pub const MOLECULER_EVENT_BROADCAST_TOTAL: &str = "penoos.event.broadcast.total";
    pub const MOLECULER_TRANSIT_PUBLISH_TOTAL: &str = "penoos.transit.publish.total";
    pub const MOLECULER_TRANSIT_RECEIVE_TOTAL: &str = "penoos.transit.receive.total";
    pub const MOLECULER_TRANSIT_REQUESTS_ACTIVE: &str = "penoos.transit.requests.active";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_metric() {
        let registry = MetricsRegistry::new();
        registry.register_counter("test.counter", "Test counter");

        registry.increment_counter("test.counter", 1);
        registry.increment_counter("test.counter", 5);

        if let Some(MetricValue::Counter(value)) = registry.get_metric("test.counter") {
            assert_eq!(value, 6);
        } else {
            panic!("Expected counter metric");
        }
    }

    #[test]
    fn test_gauge_metric() {
        let registry = MetricsRegistry::new();
        registry.register_gauge("test.gauge", "Test gauge");

        registry.set_gauge("test.gauge", 42.5);

        if let Some(MetricValue::Gauge(value)) = registry.get_metric("test.gauge") {
            assert_eq!(value, 42.5);
        } else {
            panic!("Expected gauge metric");
        }
    }

    #[test]
    fn test_histogram_metric() {
        let registry = MetricsRegistry::new();
        let buckets = vec![0.1, 0.5, 1.0, 2.0, 5.0];
        registry.register_histogram("test.histogram", "Test histogram", buckets);

        registry.observe_histogram("test.histogram", 0.3);
        registry.observe_histogram("test.histogram", 1.5);

        if let Some(MetricValue::Histogram { values, .. }) = registry.get_metric("test.histogram") {
            assert_eq!(values.len(), 2);
            assert!(values.contains(&0.3));
            assert!(values.contains(&1.5));
        } else {
            panic!("Expected histogram metric");
        }
    }

    #[test]
    fn test_info_metric() {
        let registry = MetricsRegistry::new();
        registry.register_info("test.info", "Test info");

        registry.set_info("test.info", "test_value");

        if let Some(MetricValue::Info(value)) = registry.get_metric("test.info") {
            assert_eq!(value, "test_value");
        } else {
            panic!("Expected info metric");
        }
    }

    #[test]
    fn test_disabled_registry() {
        let mut registry = MetricsRegistry::new();
        registry.disable();

        registry.register_counter("test.counter", "Test counter");
        registry.increment_counter("test.counter", 1);

        assert!(registry.get_metric("test.counter").is_none());
    }
}
