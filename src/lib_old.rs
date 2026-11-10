//! # Penoos - Microservices Framework for Rust
//! 
//! Penoos is a fast, modern and powerful microservices framework for Rust.
//! It helps you to build efficient, reliable & scalable services.
//! 
//! ## Core Features
//! 
//! - **Async/await support** - Built on top of Tokio runtime
//! - **Service Registry** - Dynamic service discovery
//! - **Load Balancing** - Multiple strategies (round-robin, random, etc.)
//! - **Fault Tolerance** - Circuit breaker, retry, timeout patterns
//! - **Event-driven Architecture** - Pub/Sub messaging
//! - **Pluggable Transporters** - NATS, Redis, TCP support
//! - **Caching** - Built-in caching with multiple backends
//! - **Metrics & Tracing** - Observability support
//! 
//! ## Quick Start
//! 
//! ```rust
//! use penoos::{ServiceBroker, Service, Context, Result};
//! use serde_json::json;
//! 
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let mut broker = ServiceBroker::new()?;
//!     
//!     let service = Service::new("math")
//!         .action("add", |ctx: Context| async move {
//!             let a: f64 = ctx.params().get("a").unwrap().as_f64().unwrap();
//!             let b: f64 = ctx.params().get("b").unwrap().as_f64().unwrap();

// Core modules
pub mod broker;
pub mod service;
pub mod context;
pub mod errors;
pub mod constants;
pub mod utils;
pub mod validators;
//!             Ok(json!(a + b))
//!         });
//!     
//!     broker.create_service(service).await?;
//!     broker.start().await?;
//!     
//!     let result = broker.call("math.add", json!({"a": 5, "b": 3})).await?;
//!     println!("5 + 3 = {}", result);
//!     
//!     Ok(())
//! }
//! ```

pub mod broker;
pub mod service;
pub mod context;
pub mod registry;
pub mod errors;
pub mod transport;
pub mod cache;
pub mod metrics;
pub mod tracing;
pub mod utils;
pub mod middleware;
pub mod serializers;
pub mod validators;
pub mod constants;

// Re-exports for convenience
pub use broker::ServiceBroker;
pub use service::{Service, ServiceSchema, ActionHandler};
pub use context::Context;
pub use errors::{PenoosError, Result};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PROTOCOL_VERSION: &str = "5";

/// Default configuration for Penoos
#[derive(Debug, Clone)]
pub struct PenoosConfig {
    pub node_id: Option<String>,
    pub namespace: String,
    pub logger_level: String,
    pub request_timeout: u64,
    pub retry_policy: RetryPolicy,
    pub heartbeat_interval: u64,
    pub heartbeat_timeout: u64,
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub enabled: bool,
    pub retries: u32,
    pub delay: u64,
    pub max_delay: u64,
    pub factor: f64,
}

impl Default for PenoosConfig {
    fn default() -> Self {
        Self {
            node_id: None,
            namespace: String::new(),
            logger_level: "info".to_string(),
            request_timeout: 10000, // 10 seconds
            retry_policy: RetryPolicy {
                enabled: false,
                retries: 5,
                delay: 100,
                max_delay: 1000,
                factor: 2.0,
            },
            heartbeat_interval: 10,
            heartbeat_timeout: 30,
            metrics_enabled: true,
            tracing_enabled: true,
        }
    }
}
