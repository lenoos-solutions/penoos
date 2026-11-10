//! Error types for Penoos framework

use std::fmt;

use thiserror::Error;

/// Result type used throughout the Penoos framework
pub type Result<T> = std::result::Result<T, PenoosError>;

/// Main error type for Penoos operations
#[derive(Error, Debug)]
pub enum PenoosError {
    /// Service not found
    #[error("Service '{service}' not found")]
    ServiceNotFound { service: String },

    /// Service not available
    #[error("Service '{service}' not available")]
    ServiceNotAvailable { service: String },

    /// Action not found
    #[error("Action '{action}' not found")]
    ActionNotFound { action: String },

    /// Request timeout
    #[error("Request timeout after {timeout}ms for action '{action}'")]
    Timeout { action: String, timeout: u64 },

    /// Service already exists
    #[error("Service '{service}' already exists")]
    ServiceAlreadyExists { service: String },

    /// Broker already started
    #[error("ServiceBroker is already started")]
    AlreadyStarted,

    /// Broker not started
    #[error("ServiceBroker is not started")]
    NotStarted,

    /// Invalid parameters
    #[error("Invalid parameters: {message}")]
    InvalidParameters { message: String },

    /// Validation error
    #[error("Validation error: {message}")]
    ValidationError { message: String },

    /// Circuit breaker is open
    #[error("Circuit breaker is open for service '{service}'")]
    CircuitBreakerOpen { service: String },

    /// Transport error
    #[error("Transport error: {message}")]
    TransportError { message: String },

    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Unauthorized access
    #[error("Unauthorized: {message}")]
    UnauthorizedError { message: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {message}")]
    RateLimitError { message: String },

    /// Cache error
    #[error("Cache error: {message}")]
    CacheError { message: String },

    /// Registry error
    #[error("Registry error: {message}")]
    RegistryError { message: String },

    /// Middleware error
    #[error("Middleware error: {message}")]
    MiddlewareError { message: String },

    /// Service schema error
    #[error("Service schema error: {message}")]
    ServiceSchemaError { message: String },

    /// Max call level exceeded
    #[error("Max call level exceeded. Current: {current}, Max: {max}")]
    MaxCallLevelExceeded { current: u32, max: u32 },

    /// Request skipped
    #[error("Request skipped: {reason}")]
    RequestSkipped { reason: String },

    /// Protocol error
    #[error("Protocol error: {message}")]
    ConfigError { message: String },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic error
    #[error("Error: {message}")]
    Generic { message: String },

    /// Custom error with error code
    #[error("Error {code}: {message}")]
    Custom { code: u32, message: String, data: Option<serde_json::Value> },
}

impl PenoosError {
    /// Create a new service not found error
    pub fn service_not_found(service: &str) -> Self {
        Self::ServiceNotFound {
            service: service.to_string(),
        }
    }

    /// Create a new service not available error
    pub fn service_not_available(service: &str) -> Self {
        Self::ServiceNotAvailable {
            service: service.to_string(),
        }
    }

    /// Create a new action not found error
    pub fn action_not_found(action: &str) -> Self {
        Self::ActionNotFound {
            action: action.to_string(),
        }
    }

    /// Create a new timeout error
    pub fn timeout(action: &str, timeout: u64) -> Self {
        Self::Timeout {
            action: action.to_string(),
            timeout,
        }
    }

    /// Create a new validation error
    pub fn validation_error(message: &str) -> Self {
        Self::ValidationError {
            message: message.to_string(),
        }
    }

    /// Create a new config error
    pub fn config_error(message: &str) -> Self {
        Self::ConfigError {
            message: message.to_string(),
        }
    }

    /// Create a new runtime error (using Generic)
    pub fn runtime_error(message: &str) -> Self {
        Self::Generic {
            message: message.to_string(),
        }
    }

    /// Create a new transport error
    pub fn transport_error(message: &str) -> Self {
        Self::TransportError {
            message: message.to_string(),
        }
    }

    /// Create a new custom error
    pub fn custom(code: u32, message: &str) -> Self {
        Self::Custom {
            code,
            message: message.to_string(),
            data: None,
        }
    }

    /// Create a new custom error with data
    pub fn custom_with_data(code: u32, message: &str, data: serde_json::Value) -> Self {
        Self::Custom {
            code,
            message: message.to_string(),
            data: Some(data),
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> u32 {
        match self {
            Self::ServiceNotFound { .. } => 404,
            Self::ServiceNotAvailable { .. } => 503,
            Self::ActionNotFound { .. } => 404,
            Self::Timeout { .. } => 408,
            Self::ServiceAlreadyExists { .. } => 409,
            Self::AlreadyStarted => 400,
            Self::NotStarted => 400,
            Self::InvalidParameters { .. } => 400,
            Self::ValidationError { .. } => 422,
            Self::CircuitBreakerOpen { .. } => 503,
            Self::TransportError { .. } => 500,
            Self::SerializationError { .. } => 500,
            Self::UnauthorizedError { .. } => 401,
            Self::RateLimitError { .. } => 429,
            Self::CacheError { .. } => 500,
            Self::RegistryError { .. } => 500,
            Self::MiddlewareError { .. } => 500,
            Self::ServiceSchemaError { .. } => 400,
            Self::MaxCallLevelExceeded { .. } => 400,
            Self::RequestSkipped { .. } => 400,
            Self::ConfigError { .. } => 400,
            Self::Io(_) => 500,
            Self::Json(_) => 400,
            Self::Generic { .. } => 500,
            Self::Custom { code, .. } => *code,
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::ServiceNotAvailable { .. } => true,
            Self::Timeout { .. } => true,
            Self::CircuitBreakerOpen { .. } => false,
            Self::TransportError { .. } => true,
            Self::Io(_) => true,
            _ => false,
        }
    }

    /// Get the error type name
    pub fn error_type(&self) -> &'static str {
        match self {
            Self::ServiceNotFound { .. } => "SERVICE_NOT_FOUND",
            Self::ServiceNotAvailable { .. } => "SERVICE_NOT_AVAILABLE",
            Self::ActionNotFound { .. } => "ACTION_NOT_FOUND",
            Self::Timeout { .. } => "REQUEST_TIMEOUT",
            Self::ServiceAlreadyExists { .. } => "SERVICE_ALREADY_EXISTS",
            Self::AlreadyStarted => "ALREADY_STARTED",
            Self::NotStarted => "NOT_STARTED",
            Self::InvalidParameters { .. } => "INVALID_PARAMETERS",
            Self::ValidationError { .. } => "VALIDATION_ERROR",
            Self::CircuitBreakerOpen { .. } => "CIRCUIT_BREAKER_OPEN",
            Self::TransportError { .. } => "TRANSPORT_ERROR",
            Self::SerializationError { .. } => "SERIALIZATION_ERROR",
            Self::UnauthorizedError { .. } => "UNAUTHORIZED_ERROR",
            Self::RateLimitError { .. } => "RATE_LIMIT_EXCEEDED",
            Self::CacheError { .. } => "CACHE_ERROR",
            Self::RegistryError { .. } => "REGISTRY_ERROR",
            Self::MiddlewareError { .. } => "MIDDLEWARE_ERROR",
            Self::ServiceSchemaError { .. } => "SERVICE_SCHEMA_ERROR",
            Self::MaxCallLevelExceeded { .. } => "MAX_CALL_LEVEL_EXCEEDED",
            Self::RequestSkipped { .. } => "REQUEST_SKIPPED",
            Self::ConfigError { .. } => "CONFIG_ERROR",
            Self::Io(_) => "IO_ERROR",
            Self::Json(_) => "JSON_ERROR",
            Self::Generic { .. } => "GENERIC_ERROR",
            Self::Custom { .. } => "CUSTOM_ERROR",
        }
    }
}

/// Convert from anyhow::Error
impl From<anyhow::Error> for PenoosError {
    fn from(err: anyhow::Error) -> Self {
        Self::Generic {
            message: err.to_string(),
        }
    }
}

/// Convert from clap::Error
impl From<clap::Error> for PenoosError {
    fn from(err: clap::Error) -> Self {
        Self::ConfigError {
            message: err.to_string(),
        }
    }
}

/// Service schema error - specific error for service definition issues
#[derive(Error, Debug)]
pub struct ServiceSchemaError(pub String);

impl fmt::Display for ServiceSchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Service schema error: {}", self.0)
    }
}

/// Request skipped error - for middleware that skip requests
#[derive(Error, Debug)]
pub struct RequestSkippedError {
    pub reason: String,
}

impl fmt::Display for RequestSkippedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Request skipped: {}", self.reason)
    }
}

/// Max call level exceeded error
#[derive(Error, Debug)]
pub struct MaxCallLevelError {
    pub current: u32,
    pub max: u32,
}

impl fmt::Display for MaxCallLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Max call level exceeded. Current: {}, Max: {}", self.current, self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(PenoosError::service_not_found("test").error_code(), 404);
        assert_eq!(PenoosError::timeout("test", 1000).error_code(), 408);
        assert_eq!(PenoosError::validation_error("test").error_code(), 422);
        assert_eq!(PenoosError::custom(999, "test").error_code(), 999);
    }

    #[test]
    fn test_retryable() {
        assert!(PenoosError::service_not_available("test").is_retryable());
        assert!(PenoosError::timeout("test", 1000).is_retryable());
        assert!(!PenoosError::CircuitBreakerOpen { service: "test".to_string() }.is_retryable());
        assert!(!PenoosError::validation_error("test").is_retryable());
    }

    #[test]
    fn test_error_types() {
        assert_eq!(PenoosError::service_not_found("test").error_type(), "SERVICE_NOT_FOUND");
        assert_eq!(PenoosError::timeout("test", 1000).error_type(), "REQUEST_TIMEOUT");
        assert_eq!(PenoosError::validation_error("test").error_type(), "VALIDATION_ERROR");
    }
}
