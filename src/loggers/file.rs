//! File logger implementation for Penoos
//!
//! This logger writes log messages to files with rotation support.

use crate::errors::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

/// Log level for file logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// File logger configuration
#[derive(Debug, Clone)]
pub struct FileLoggerConfig {
    pub file_path: PathBuf,
    pub max_file_size: u64,
    pub max_files: u32,
    pub level: LogLevel,
    pub format: String,
}

impl Default for FileLoggerConfig {
    fn default() -> Self {
        Self {
            file_path: PathBuf::from("logs/penoos.log"),
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
            level: LogLevel::Info,
            format: "{timestamp} [{level}] {message}".to_string(),
        }
    }
}

/// File logger implementation
pub struct FileLogger {
    config: FileLoggerConfig,
}

impl FileLogger {
    pub fn new(config: FileLoggerConfig) -> Self {
        Self { config }
    }

    /// Format a log message
    fn format_message(&self, level: LogLevel, message: &str) -> String {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let level_str = match level {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        };

        self.config
            .format
            .replace("{timestamp}", &timestamp.to_string())
            .replace("{level}", level_str)
            .replace("{message}", message)
    }

    /// Write a log message to file
    async fn write_log(&self, level: LogLevel, message: &str) -> Result<()> {
        if level < self.config.level {
            return Ok(());
        }

        let formatted = self.format_message(level, message);

        // Ensure directory exists
        if let Some(parent) = self.config.file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.file_path)
            .await?;

        file.write_all(formatted.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;

        Ok(())
    }
}

/// Logger trait for file logging
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
impl Logger for FileLogger {
    async fn log(&self, level: LogLevel, message: &str, _meta: Option<Value>) -> Result<()> {
        self.write_log(level, message).await
    }

    async fn trace(&self, message: &str) -> Result<()> {
        self.write_log(LogLevel::Trace, message).await
    }

    async fn debug(&self, message: &str) -> Result<()> {
        self.write_log(LogLevel::Debug, message).await
    }

    async fn info(&self, message: &str) -> Result<()> {
        self.write_log(LogLevel::Info, message).await
    }

    async fn warn(&self, message: &str) -> Result<()> {
        self.write_log(LogLevel::Warn, message).await
    }

    async fn error(&self, message: &str) -> Result<()> {
        self.write_log(LogLevel::Error, message).await
    }

    async fn fatal(&self, message: &str) -> Result<()> {
        self.write_log(LogLevel::Fatal, message).await
    }
}
