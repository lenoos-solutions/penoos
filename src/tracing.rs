//! Tracing and observability support

use std::collections::HashMap;
use std::time::Instant;

/// Tracing context for distributed tracing
#[derive(Debug, Clone)]
pub struct TracingContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub baggage: HashMap<String, String>,
}

/// Span for tracing
#[derive(Debug)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: Instant,
    pub tags: HashMap<String, String>,
    pub logs: Vec<LogEntry>,
}

/// Log entry within a span
#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: Instant,
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, String>,
}

/// Log levels for tracing
#[derive(Debug, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Tracer for creating and managing spans
pub struct Tracer {
    enabled: bool,
}

impl Tracer {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn start_span(&self, operation_name: &str) -> Option<Span> {
        if !self.enabled {
            return None;
        }

        Some(Span {
            trace_id: crate::utils::generate_id(),
            span_id: crate::utils::generate_id(),
            parent_span_id: None,
            operation_name: operation_name.to_string(),
            start_time: Instant::now(),
            tags: HashMap::new(),
            logs: Vec::new(),
        })
    }

    pub fn child_span(&self, parent: &Span, operation_name: &str) -> Option<Span> {
        if !self.enabled {
            return None;
        }

        Some(Span {
            trace_id: parent.trace_id.clone(),
            span_id: crate::utils::generate_id(),
            parent_span_id: Some(parent.span_id.clone()),
            operation_name: operation_name.to_string(),
            start_time: Instant::now(),
            tags: HashMap::new(),
            logs: Vec::new(),
        })
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new(true)
    }
}

impl Span {
    pub fn set_tag(&mut self, key: &str, value: &str) {
        self.tags.insert(key.to_string(), value.to_string());
    }

    pub fn log(&mut self, level: LogLevel, message: &str) {
        self.logs.push(LogEntry {
            timestamp: Instant::now(),
            level,
            message: message.to_string(),
            fields: HashMap::new(),
        });
    }

    pub fn finish(self) {
        let duration = self.start_time.elapsed();
        log::debug!("Span '{}' finished in {:?}", self.operation_name, duration);
    }
}
