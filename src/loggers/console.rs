//! Console logger implementation

use super::{Colors, LogEntry, LogFormat, LogLevel, Logger, LoggerConfig};
use std::io::{self, Write};

/// Console logger that outputs to stdout/stderr
pub struct ConsoleLogger {
    config: LoggerConfig,
}

impl ConsoleLogger {
    pub fn new(config: LoggerConfig) -> Self {
        Self { config }
    }

    fn format_text(&self, entry: &LogEntry) -> String {
        let mut output = String::new();

        // Timestamp
        if let Some(ref format) = self.config.timestamp_format {
            let timestamp = entry.timestamp.format(format);
            if self.config.colors {
                output.push_str(&format!("{}{}{} ", Colors::DIM, timestamp, Colors::RESET));
            } else {
                output.push_str(&format!("{} ", timestamp));
            }
        }

        // Level
        if self.config.colors {
            let level_color = Colors::for_level(entry.level);
            output.push_str(&format!(
                "{}[{}]{} ",
                level_color,
                entry.level.as_str(),
                Colors::RESET
            ));
        } else {
            output.push_str(&format!("[{}] ", entry.level.as_str()));
        }

        // Module/Service
        if let Some(ref module) = entry.module {
            if self.config.colors && self.config.module_colors {
                let module_color = Colors::for_module(module);
                output.push_str(&format!("{}{}:{} ", module_color, module, Colors::RESET));
            } else {
                output.push_str(&format!("{}: ", module));
            }
        } else if let Some(ref service) = entry.service {
            if self.config.colors && self.config.module_colors {
                let service_color = Colors::for_module(service);
                output.push_str(&format!("{}{}:{} ", service_color, service, Colors::RESET));
            } else {
                output.push_str(&format!("{}: ", service));
            }
        }

        // Message
        output.push_str(&entry.message);

        // Request ID
        if let Some(ref request_id) = entry.request_id {
            if self.config.colors {
                output.push_str(&format!(
                    " {}<req:{}>{}",
                    Colors::DIM,
                    request_id,
                    Colors::RESET
                ));
            } else {
                output.push_str(&format!(" <req:{}>", request_id));
            }
        }

        // Action
        if let Some(ref action) = entry.action {
            if self.config.colors {
                output.push_str(&format!(
                    " {}<action:{}>{}",
                    Colors::DIM,
                    action,
                    Colors::RESET
                ));
            } else {
                output.push_str(&format!(" <action:{}>", action));
            }
        }

        // Node ID
        if let Some(ref node_id) = entry.node_id {
            if self.config.colors {
                output.push_str(&format!(
                    " {}<node:{}>{}",
                    Colors::DIM,
                    node_id,
                    Colors::RESET
                ));
            } else {
                output.push_str(&format!(" <node:{}>", node_id));
            }
        }

        // Metadata
        if !entry.metadata.is_empty() {
            let metadata_str = serde_json::to_string(&entry.metadata).unwrap_or_default();
            if self.config.colors {
                output.push_str(&format!(
                    " {}{}{}",
                    Colors::DIM,
                    metadata_str,
                    Colors::RESET
                ));
            } else {
                output.push_str(&format!(" {}", metadata_str));
            }
        }

        output
    }

    fn format_simple(&self, entry: &LogEntry) -> String {
        let timestamp = if let Some(ref format) = self.config.timestamp_format {
            entry.timestamp.format(format).to_string()
        } else {
            entry.timestamp.format("%H:%M:%S").to_string()
        };

        if self.config.colors {
            let level_color = Colors::for_level(entry.level);
            format!(
                "{}{} {}[{}]{} {}",
                Colors::DIM,
                timestamp,
                level_color,
                entry.level.as_str(),
                Colors::RESET,
                entry.message
            )
        } else {
            format!("{} [{}] {}", timestamp, entry.level.as_str(), entry.message)
        }
    }

    fn format_short(&self, entry: &LogEntry) -> String {
        if self.config.colors {
            let level_color = Colors::for_level(entry.level);
            format!(
                "{}[{}]{} {}",
                level_color,
                entry.level.as_str(),
                Colors::RESET,
                entry.message
            )
        } else {
            format!("[{}] {}", entry.level.as_str(), entry.message)
        }
    }
}

impl Logger for ConsoleLogger {
    fn log(&self, entry: &LogEntry) {
        if !self.is_level_enabled(entry.level) {
            return;
        }

        let formatted = match self.config.format {
            LogFormat::Text => self.format_text(entry),
            LogFormat::Json => {
                // JSON format is handled by the json logger
                serde_json::to_string(&serde_json::json!({
                    "timestamp": entry.timestamp.to_rfc3339(),
                    "level": entry.level.as_str(),
                    "message": entry.message,
                    "module": entry.module,
                    "service": entry.service,
                    "action": entry.action,
                    "requestId": entry.request_id,
                    "nodeId": entry.node_id,
                    "metadata": entry.metadata
                }))
                .unwrap_or_default()
            }
            LogFormat::Simple => self.format_simple(entry),
            LogFormat::Short => self.format_short(entry),
        };

        // Write to stderr for errors and warnings, stdout for others
        if entry.level >= LogLevel::Warn {
            let _ = writeln!(io::stderr(), "{}", formatted);
        } else {
            let _ = writeln!(io::stdout(), "{}", formatted);
        }
    }

    fn is_level_enabled(&self, level: LogLevel) -> bool {
        level >= self.config.level
    }

    fn flush(&self) {
        let _ = io::stdout().flush();
        let _ = io::stderr().flush();
    }
}

impl Default for ConsoleLogger {
    fn default() -> Self {
        Self::new(LoggerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_console_logger_levels() {
        let config = LoggerConfig {
            level: LogLevel::Info,
            ..Default::default()
        };
        let logger = ConsoleLogger::new(config);

        assert!(!logger.is_level_enabled(LogLevel::Debug));
        assert!(logger.is_level_enabled(LogLevel::Info));
        assert!(logger.is_level_enabled(LogLevel::Error));
    }

    #[test]
    fn test_console_logger_formatting() {
        let config = LoggerConfig {
            colors: false,
            format: LogFormat::Simple,
            timestamp_format: Some("%H:%M:%S".to_string()),
            ..Default::default()
        };
        let logger = ConsoleLogger::new(config);

        let entry = LogEntry {
            timestamp: chrono::DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
            level: LogLevel::Info,
            message: "Test message".to_string(),
            module: Some("test".to_string()),
            service: None,
            action: None,
            request_id: None,
            node_id: None,
            metadata: HashMap::new(),
        };

        let formatted = logger.format_simple(&entry);
        assert!(formatted.contains("12:00:00"));
        assert!(formatted.contains("[INFO]"));
        assert!(formatted.contains("Test message"));
    }

    #[test]
    fn test_log_levels_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Fatal);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("info"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("warn"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("warning"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_colors_for_level() {
        assert_eq!(Colors::for_level(LogLevel::Error), Colors::RED);
        assert_eq!(Colors::for_level(LogLevel::Warn), Colors::YELLOW);
        assert_eq!(Colors::for_level(LogLevel::Info), Colors::GREEN);
    }
}
