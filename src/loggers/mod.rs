//! Logging implementations for Penoos

pub mod console;
pub mod file;
pub mod json;

use serde_json::Value;
use std::collections::HashMap;

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    // Returns Option, not the FromStr Result, so it intentionally does not implement the trait.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(level: &str) -> Option<Self> {
        match level.to_lowercase().as_str() {
            "trace" => Some(LogLevel::Trace),
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warn" | "warning" => Some(LogLevel::Warn),
            "error" => Some(LogLevel::Error),
            "fatal" => Some(LogLevel::Fatal),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }
}

/// Log entry structure
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub message: String,
    pub module: Option<String>,
    pub service: Option<String>,
    pub action: Option<String>,
    pub request_id: Option<String>,
    pub node_id: Option<String>,
    pub metadata: HashMap<String, Value>,
}

/// Logger trait
pub trait Logger: Send + Sync {
    fn log(&self, entry: &LogEntry);
    fn is_level_enabled(&self, level: LogLevel) -> bool;
    fn flush(&self);
}

/// Logger configuration
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub level: LogLevel,
    pub format: LogFormat,
    pub colors: bool,
    pub module_colors: bool,
    pub timestamp_format: Option<String>,
    pub object_printer: Option<ObjectPrinter>,
}

/// Log formatting options
#[derive(Debug, Clone)]
pub enum LogFormat {
    Text,
    Json,
    Simple,
    Short,
}

/// Object printing options
#[derive(Debug, Clone)]
pub enum ObjectPrinter {
    Json,
    Pretty,
    Yaml,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Text,
            colors: true,
            module_colors: true,
            timestamp_format: Some("%Y-%m-%d %H:%M:%S%.3f".to_string()),
            object_printer: Some(ObjectPrinter::Json),
        }
    }
}

impl LogEntry {
    pub fn new(level: LogLevel, message: String) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            level,
            message,
            module: None,
            service: None,
            action: None,
            request_id: None,
            node_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_module(mut self, module: String) -> Self {
        self.module = Some(module);
        self
    }

    pub fn with_service(mut self, service: String) -> Self {
        self.service = Some(service);
        self
    }

    pub fn with_action(mut self, action: String) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    pub fn with_node_id(mut self, node_id: String) -> Self {
        self.node_id = Some(node_id);
        self
    }

    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Color codes for terminal output
pub struct Colors;

impl Colors {
    pub const RESET: &'static str = "\x1b[0m";
    pub const BOLD: &'static str = "\x1b[1m";
    pub const DIM: &'static str = "\x1b[2m";

    // Foreground colors
    pub const BLACK: &'static str = "\x1b[30m";
    pub const RED: &'static str = "\x1b[31m";
    pub const GREEN: &'static str = "\x1b[32m";
    pub const YELLOW: &'static str = "\x1b[33m";
    pub const BLUE: &'static str = "\x1b[34m";
    pub const MAGENTA: &'static str = "\x1b[35m";
    pub const CYAN: &'static str = "\x1b[36m";
    pub const WHITE: &'static str = "\x1b[37m";

    // Bright foreground colors
    pub const BRIGHT_BLACK: &'static str = "\x1b[90m";
    pub const BRIGHT_RED: &'static str = "\x1b[91m";
    pub const BRIGHT_GREEN: &'static str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &'static str = "\x1b[93m";
    pub const BRIGHT_BLUE: &'static str = "\x1b[94m";
    pub const BRIGHT_MAGENTA: &'static str = "\x1b[95m";
    pub const BRIGHT_CYAN: &'static str = "\x1b[96m";
    pub const BRIGHT_WHITE: &'static str = "\x1b[97m";

    pub fn for_level(level: LogLevel) -> &'static str {
        match level {
            LogLevel::Trace => Self::DIM,
            LogLevel::Debug => Self::BRIGHT_BLACK,
            LogLevel::Info => Self::GREEN,
            LogLevel::Warn => Self::YELLOW,
            LogLevel::Error => Self::RED,
            LogLevel::Fatal => Self::BRIGHT_RED,
        }
    }

    pub fn for_module(module: &str) -> &'static str {
        // Generate a consistent color for each module name
        let hash = module
            .chars()
            .fold(0u32, |acc, c| acc.wrapping_add(c as u32));
        let colors = [
            Self::BLUE,
            Self::CYAN,
            Self::MAGENTA,
            Self::YELLOW,
            Self::BRIGHT_BLUE,
            Self::BRIGHT_CYAN,
            Self::BRIGHT_MAGENTA,
        ];
        colors[hash as usize % colors.len()]
    }
}
