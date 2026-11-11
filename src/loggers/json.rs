//! JSON logger implementation for Penoos
//!
//! This logger outputs structured JSON log messages.

use crate::errors::Result;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::io::{self, Write};

/// Log level for JSON logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Fatal => "fatal",
        }
    }
}

/// JSON logger configuration
#[derive(Debug, Clone)]
pub struct JsonLoggerConfig {
    pub level: LogLevel,
    pub pretty: bool,
    pub include_timestamp: bool,
    pub include_hostname: bool,
}

impl Default for JsonLoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            pretty: false,
            include_timestamp: true,
            include_hostname: true,
        }
    }
}

/// JSON logger implementation
pub struct JsonLogger {
    config: JsonLoggerConfig,
}

impl JsonLogger {
    pub fn new(config: JsonLoggerConfig) -> Self {
        Self { config }
    }

    /// Create a log entry as JSON
    fn create_log_entry(&self, level: LogLevel, message: &str, meta: Option<Value>) -> Value {
        let mut entry = json!({
            "level": level.as_str(),
            "message": message,
        });

        if self.config.include_timestamp {
            entry["timestamp"] = json!(chrono::Utc::now().to_rfc3339());
        }

        if self.config.include_hostname
            && let Ok(hostname) = hostname::get()
        {
            entry["hostname"] = json!(hostname.to_string_lossy());
        }

        if let Some(meta) = meta {
            entry["meta"] = meta;
        }

        entry
    }

    /// Write a log entry to stdout
    fn write_log(&self, level: LogLevel, message: &str, meta: Option<Value>) -> Result<()> {
        if level < self.config.level {
            return Ok(());
        }

        let entry = self.create_log_entry(level, message, meta);

        let output = if self.config.pretty {
            serde_json::to_string_pretty(&entry)?
        } else {
            serde_json::to_string(&entry)?
        };

        println!("{}", output);
        io::stdout().flush()?;

        Ok(())
    }
}

/// Logger trait for JSON logging
#[async_trait]
pub trait Logger: Send + Sync {
    async fn log(&self, level: LogLevel, message: &str, meta: Option<Value>) -> Result<()>;
    async fn trace(&self, message: &str) -> Result<()>;
    async fn debug(&self, message: &str) -> Result<()>;
    async fn info(&self, message: &str) -> Result<()>;
    async fn warn(&self, message: &str) -> Result<()>;
    async fn error(&self, message: &str) -> Result<()>;
    async fn fatal(&self, message: &str) -> Result<()>;
}

#[async_trait]
impl Logger for JsonLogger {
    async fn log(&self, level: LogLevel, message: &str, meta: Option<Value>) -> Result<()> {
        self.write_log(level, message, meta)
    }

    async fn trace(&self, message: &str) -> Result<()> {
        self.write_log(LogLevel::Trace, message, None)
    }

    async fn debug(&self, message: &str) -> Result<()> {
        self.write_log(LogLevel::Debug, message, None)
    }

    async fn info(&self, message: &str) -> Result<()> {
        self.write_log(LogLevel::Info, message, None)
    }

    async fn warn(&self, message: &str) -> Result<()> {
        self.write_log(LogLevel::Warn, message, None)
    }

    async fn error(&self, message: &str) -> Result<()> {
        self.write_log(LogLevel::Error, message, None)
    }

    async fn fatal(&self, message: &str) -> Result<()> {
        self.write_log(LogLevel::Fatal, message, None)
    }
}
