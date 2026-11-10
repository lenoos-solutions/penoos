//! Metrics reporters for exporting metrics to various systems

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::errors::Result;
use crate::metrics::{MetricsRegistry, MetricValue};

/// Trait for metrics reporters
#[async_trait]
pub trait MetricsReporter: Send + Sync {
    /// Name of the reporter
    fn name(&self) -> &str;
    
    /// Start the reporter
    async fn start(&self) -> Result<()>;
    
    /// Stop the reporter
    async fn stop(&self) -> Result<()>;
    
    /// Report metrics
    async fn report(&self, metrics: &HashMap<String, MetricValue>) -> Result<()>;
    
    /// Get reporter configuration
    fn config(&self) -> HashMap<String, Value>;
}

/// Console metrics reporter
pub struct ConsoleReporter {
    pub name: String,
    pub interval: Duration,
    pub format: ConsoleFormat,
    pub include_timestamp: bool,
    running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub enum ConsoleFormat {
    Text,
    Json,
    Table,
}

impl ConsoleReporter {
    pub fn new() -> Self {
        Self {
            name: "console".to_string(),
            interval: Duration::from_secs(30),
            format: ConsoleFormat::Text,
            include_timestamp: true,
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
    
    pub fn with_format(mut self, format: ConsoleFormat) -> Self {
        self.format = format;
        self
    }
    
    pub fn with_timestamp(mut self, include: bool) -> Self {
        self.include_timestamp = include;
        self
    }
    
    fn format_metrics(&self, metrics: &HashMap<String, MetricValue>) -> String {
        match self.format {
            ConsoleFormat::Text => {
                let mut output = String::new();
                if self.include_timestamp {
                    output.push_str(&format!("📊 Metrics Report - {}\n", 
                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
                    output.push_str(&"─".repeat(50));
                    output.push('\n');
                }
                
                for (name, value) in metrics {
                    match value {
                        MetricValue::Counter(v) => output.push_str(&format!("Counter  | {} = {}\n", name, v)),
                        MetricValue::Gauge(v) => output.push_str(&format!("Gauge    | {} = {:.2}\n", name, v)),
                        MetricValue::Histogram { values, .. } => {
                            let count = values.len();
                            let sum: f64 = values.iter().sum();
                            let avg = if count > 0 { sum / (count as f64) } else { 0.0 };
                            output.push_str(&format!("Histogram| {} = {:.2} (avg), {} samples\n", name, avg, count));
                        },
                        MetricValue::Info(v) => output.push_str(&format!("Info     | {} = {}\n", name, v)),
                    }
                }
                output
            },
            ConsoleFormat::Json => {
                let mut json_metrics = serde_json::Map::new();
                if self.include_timestamp {
                    json_metrics.insert("timestamp".to_string(), Value::String(
                        chrono::Utc::now().to_rfc3339()
                    ));
                }
                
                let mut metrics_obj = serde_json::Map::new();
                for (name, value) in metrics {
                    let metric_value = match value {
                        MetricValue::Counter(v) => serde_json::json!({"type": "counter", "value": v}),
                        MetricValue::Gauge(v) => serde_json::json!({"type": "gauge", "value": v}),
                        MetricValue::Histogram { values, buckets } => {
                            let count = values.len();
                            let sum: f64 = values.iter().sum();
                            serde_json::json!({
                                "type": "histogram",
                                "count": count,
                                "sum": sum,
                                "buckets": buckets
                            })
                        },
                        MetricValue::Info(v) => serde_json::json!({"type": "info", "value": v}),
                    };
                    metrics_obj.insert(name.clone(), metric_value);
                }
                json_metrics.insert("metrics".to_string(), Value::Object(metrics_obj));
                serde_json::to_string_pretty(&json_metrics).unwrap()
            },
            ConsoleFormat::Table => {
                let mut output = String::new();
                if self.include_timestamp {
                    output.push_str(&format!("📊 Metrics Report - {}\n", 
                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
                }
                
                output.push_str("┌─────────────┬──────────────────────────────────┬──────────────┐\n");
                output.push_str("│ Type        │ Name                             │ Value        │\n");
                output.push_str("├─────────────┼──────────────────────────────────┼──────────────┤\n");
                
                for (name, value) in metrics {
                    let (metric_type, metric_value) = match value {
                        MetricValue::Counter(v) => ("Counter", format!("{}", v)),
                        MetricValue::Gauge(v) => ("Gauge", format!("{:.2}", v)),
                        MetricValue::Histogram { values, .. } => {
                            let count = values.len();
                            let sum: f64 = values.iter().sum();
                            let avg = if count > 0 { sum / (count as f64) } else { 0.0 };
                            ("Histogram", format!("{:.2} ({} samples)", avg, count))
                        },
                        MetricValue::Info(v) => ("Info", v.clone()),
                    };
                    
                    output.push_str(&format!("│ {:11} │ {:32} │ {:12} │\n", 
                        metric_type, 
                        if name.len() > 32 { &name[..29] } else { name }, 
                        if metric_value.len() > 12 { &metric_value[..9] } else { &metric_value }
                    ));
                }
                
                output.push_str("└─────────────┴──────────────────────────────────┴──────────────┘\n");
                output
            }
        }
    }
}

#[async_trait]
impl MetricsReporter for ConsoleReporter {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        println!("📊 Console metrics reporter started (interval: {:?})", self.interval);
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        println!("📊 Console metrics reporter stopped");
        Ok(())
    }
    
    async fn report(&self, metrics: &HashMap<String, MetricValue>) -> Result<()> {
        if metrics.is_empty() {
            return Ok(());
        }
        
        let formatted = self.format_metrics(metrics);
        println!("{}", formatted);
        Ok(())
    }
    
    fn config(&self) -> HashMap<String, Value> {
        let mut config = HashMap::new();
        config.insert("name".to_string(), Value::String(self.name.clone()));
        config.insert("interval".to_string(), Value::Number(self.interval.as_secs().into()));
        config.insert("format".to_string(), Value::String(match self.format {
            ConsoleFormat::Text => "text".to_string(),
            ConsoleFormat::Json => "json".to_string(),
            ConsoleFormat::Table => "table".to_string(),
        }));
        config.insert("includeTimestamp".to_string(), Value::Bool(self.include_timestamp));
        config
    }
}

impl Default for ConsoleReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Prometheus metrics reporter
pub struct PrometheusReporter {
    pub name: String,
    pub endpoint: String,
    pub port: u16,
    pub path: String,
    pub basic_auth: Option<(String, String)>,
    running: Arc<RwLock<bool>>,
    metrics_cache: Arc<RwLock<HashMap<String, MetricValue>>>,
}

impl PrometheusReporter {
    pub fn new() -> Self {
        Self {
            name: "prometheus".to_string(),
            endpoint: "0.0.0.0".to_string(),
            port: 9090,
            path: "/metrics".to_string(),
            basic_auth: None,
            running: Arc::new(RwLock::new(false)),
            metrics_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = endpoint;
        self
    }
    
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    pub fn with_path(mut self, path: String) -> Self {
        self.path = path;
        self
    }
    
    pub fn with_basic_auth(mut self, username: String, password: String) -> Self {
        self.basic_auth = Some((username, password));
        self
    }
    
    #[allow(dead_code)]
    fn format_prometheus_metrics(&self, metrics: &HashMap<String, MetricValue>) -> String {
        let mut output = String::new();
        
        // Add metadata
        output.push_str("# TYPE penoos_info gauge\n");
        output.push_str(&format!("penoos_info{{version=\"{}\"}} 1\n", env!("CARGO_PKG_VERSION")));
        
        for (name, value) in metrics {
            let sanitized_name = name.replace('-', "_").replace('.', "_");
            
            match value {
                MetricValue::Counter(v) => {
                    output.push_str(&format!("# TYPE {} counter\n", sanitized_name));
                    output.push_str(&format!("{} {}\n", sanitized_name, v));
                },
                MetricValue::Gauge(v) => {
                    output.push_str(&format!("# TYPE {} gauge\n", sanitized_name));
                    output.push_str(&format!("{} {}\n", sanitized_name, v));
                },
                MetricValue::Histogram { values, buckets } => {
                    let count = values.len();
                    let sum: f64 = values.iter().sum();
                    output.push_str(&format!("# TYPE {} histogram\n", sanitized_name));
                    output.push_str(&format!("{}_count {}\n", sanitized_name, count));
                    output.push_str(&format!("{}_sum {}\n", sanitized_name, sum));
                    
                    for (_idx, le) in buckets.iter().enumerate() {
                        let bucket_count = values.iter().filter(|&&v| v <= *le).count();
                        output.push_str(&format!("{}_bucket{{le=\"{}\"}} {}\n", sanitized_name, le, bucket_count));
                    }
                    output.push_str(&format!("{}_bucket{{le=\"+Inf\"}} {}\n", sanitized_name, count));
                },
                MetricValue::Info(v) => {
                    output.push_str(&format!("# TYPE {} gauge\n", sanitized_name));
                    output.push_str(&format!("{}{{info=\"{}\"}} 1\n", sanitized_name, v));
                },
            }
            output.push('\n');
        }
        
        output
    }
}

#[async_trait]
impl MetricsReporter for PrometheusReporter {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        println!("📊 Prometheus metrics reporter started on {}:{}{}", 
                 self.endpoint, self.port, self.path);
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        println!("📊 Prometheus metrics reporter stopped");
        Ok(())
    }
    
    async fn report(&self, metrics: &HashMap<String, MetricValue>) -> Result<()> {
        let mut cache = self.metrics_cache.write().await;
        cache.clear();
        cache.extend(metrics.iter().map(|(k, v)| (k.clone(), v.clone())));
        Ok(())
    }
    
    fn config(&self) -> HashMap<String, Value> {
        let mut config = HashMap::new();
        config.insert("name".to_string(), Value::String(self.name.clone()));
        config.insert("endpoint".to_string(), Value::String(self.endpoint.clone()));
        config.insert("port".to_string(), Value::Number(self.port.into()));
        config.insert("path".to_string(), Value::String(self.path.clone()));
        config.insert("basicAuth".to_string(), Value::Bool(self.basic_auth.is_some()));
        config
    }
}

impl Default for PrometheusReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// File-based metrics reporter
pub struct FileReporter {
    pub name: String,
    pub file_path: String,
    pub format: FileFormat,
    pub rotate_size: Option<u64>,
    pub max_files: Option<u32>,
    running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub enum FileFormat {
    Json,
    Csv,
    PlainText,
}

impl FileReporter {
    pub fn new(file_path: String) -> Self {
        Self {
            name: "file".to_string(),
            file_path,
            format: FileFormat::Json,
            rotate_size: None,
            max_files: None,
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    pub fn with_format(mut self, format: FileFormat) -> Self {
        self.format = format;
        self
    }
    
    pub fn with_rotation(mut self, size_mb: u64, max_files: u32) -> Self {
        self.rotate_size = Some(size_mb * 1024 * 1024);
        self.max_files = Some(max_files);
        self
    }
    
    fn format_file_metrics(&self, metrics: &HashMap<String, MetricValue>) -> String {
        match self.format {
            FileFormat::Json => {
                let mut json_obj = serde_json::Map::new();
                json_obj.insert("timestamp".to_string(), Value::String(
                    chrono::Utc::now().to_rfc3339()
                ));
                
                let mut metrics_obj = serde_json::Map::new();
                for (name, value) in metrics {
                    let metric_value = match value {
                        MetricValue::Counter(v) => serde_json::json!({"type": "counter", "value": v}),
                        MetricValue::Gauge(v) => serde_json::json!({"type": "gauge", "value": v}),
                        MetricValue::Histogram { values, buckets } => {
                            let count = values.len();
                            let sum: f64 = values.iter().sum();
                            serde_json::json!({
                                "type": "histogram",
                                "count": count,
                                "sum": sum,
                                "buckets": buckets
                            })
                        },
                        MetricValue::Info(v) => serde_json::json!({"type": "info", "value": v}),
                    };
                    metrics_obj.insert(name.clone(), metric_value);
                }
                json_obj.insert("metrics".to_string(), Value::Object(metrics_obj));
                serde_json::to_string(&json_obj).unwrap() + "\n"
            },
            FileFormat::Csv => {
                let mut csv = String::new();
                csv.push_str("timestamp,name,type,value\n");
                
                let timestamp = chrono::Utc::now().to_rfc3339();
                for (name, value) in metrics {
                    match value {
                        MetricValue::Counter(v) => {
                            csv.push_str(&format!("{},{},counter,{}\n", timestamp, name, v));
                        },
                        MetricValue::Gauge(v) => {
                            csv.push_str(&format!("{},{},gauge,{}\n", timestamp, name, v));
                        },
                        MetricValue::Histogram { values, .. } => {
                            let count = values.len();
                            let sum: f64 = values.iter().sum();
                            let avg = if count > 0 { sum / (count as f64) } else { 0.0 };
                            csv.push_str(&format!("{},{},histogram,{}\n", timestamp, name, avg));
                        },
                        MetricValue::Info(v) => {
                            csv.push_str(&format!("{},{},info,{}\n", timestamp, name, v));
                        },
                    }
                }
                csv
            },
            FileFormat::PlainText => {
                let mut text = String::new();
                text.push_str(&format!("=== Metrics Report - {} ===\n", 
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
                
                for (name, value) in metrics {
                    match value {
                        MetricValue::Counter(v) => text.push_str(&format!("{} (counter): {}\n", name, v)),
                        MetricValue::Gauge(v) => text.push_str(&format!("{} (gauge): {:.2}\n", name, v)),
                        MetricValue::Histogram { values, .. } => {
                            let count = values.len();
                            let sum: f64 = values.iter().sum();
                            let avg = if count > 0 { sum / (count as f64) } else { 0.0 };
                            text.push_str(&format!("{} (histogram): {:.2} avg, {} samples\n", name, avg, count));
                        },
                        MetricValue::Info(v) => text.push_str(&format!("{} (info): {}\n", name, v)),
                    }
                }
                text.push_str("=====================================\n\n");
                text
            }
        }
    }
}

#[async_trait]
impl MetricsReporter for FileReporter {
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        
        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&self.file_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        println!("📊 File metrics reporter started (path: {})", self.file_path);
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        println!("📊 File metrics reporter stopped");
        Ok(())
    }
    
    async fn report(&self, metrics: &HashMap<String, MetricValue>) -> Result<()> {
        if metrics.is_empty() {
            return Ok(());
        }
        
        let formatted = self.format_file_metrics(metrics);
        
        // Check file size and rotate if needed
        if let Some(max_size) = self.rotate_size {
            if let Ok(metadata) = tokio::fs::metadata(&self.file_path).await {
                if metadata.len() > max_size {
                    // Rotate file
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let rotated_path = format!("{}.{}", self.file_path, timestamp);
                    let _ = tokio::fs::rename(&self.file_path, rotated_path).await;
                }
            }
        }
        
        // Append to file
        tokio::fs::write(&self.file_path, formatted).await?;
        Ok(())
    }
    
    fn config(&self) -> HashMap<String, Value> {
        let mut config = HashMap::new();
        config.insert("name".to_string(), Value::String(self.name.clone()));
        config.insert("filePath".to_string(), Value::String(self.file_path.clone()));
        config.insert("format".to_string(), Value::String(match self.format {
            FileFormat::Json => "json".to_string(),
            FileFormat::Csv => "csv".to_string(),
            FileFormat::PlainText => "text".to_string(),
        }));
        if let Some(size) = self.rotate_size {
            config.insert("rotateSize".to_string(), Value::Number((size / 1024 / 1024).into()));
        }
        if let Some(max) = self.max_files {
            config.insert("maxFiles".to_string(), Value::Number(max.into()));
        }
        config
    }
}

/// Metrics reporting manager
pub struct MetricsReportingManager {
    reporters: Vec<Arc<dyn MetricsReporter>>,
    registry: Arc<MetricsRegistry>,
    interval: Duration,
    running: Arc<RwLock<bool>>,
}

impl MetricsReportingManager {
    pub fn new(registry: Arc<MetricsRegistry>) -> Self {
        Self {
            reporters: Vec::new(),
            registry,
            interval: Duration::from_secs(30),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
    
    pub fn add_reporter(mut self, reporter: Arc<dyn MetricsReporter>) -> Self {
        self.reporters.push(reporter);
        self
    }
    
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        
        *running = true;
        
        // Start all reporters
        for reporter in &self.reporters {
            reporter.start().await?;
        }
        
        // Start reporting loop
        let reporters = self.reporters.clone();
        let registry = self.registry.clone();
        let interval = self.interval;
        let running_check = self.running.clone();
        
        tokio::spawn(async move {
            while *running_check.read().await {
                let metrics = registry.get_all_metrics();
                
                for reporter in &reporters {
                    if let Err(e) = reporter.report(&metrics).await {
                        eprintln!("Error reporting metrics with {}: {}", reporter.name(), e);
                    }
                }
                
                tokio::time::sleep(interval).await;
            }
        });
        
        println!("📊 Metrics reporting manager started with {} reporters", self.reporters.len());
        Ok(())
    }
    
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }
        
        *running = false;
        
        // Stop all reporters
        for reporter in &self.reporters {
            reporter.stop().await?;
        }
        
        println!("📊 Metrics reporting manager stopped");
        Ok(())
    }
    
    pub fn get_reporters(&self) -> &[Arc<dyn MetricsReporter>] {
        &self.reporters
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::MetricsRegistry;

    #[tokio::test]
    async fn test_console_reporter() {
        let reporter = ConsoleReporter::new()
            .with_format(ConsoleFormat::Json)
            .with_interval(Duration::from_secs(10));
        
        assert_eq!(reporter.name(), "console");
        
        reporter.start().await.unwrap();
        
        let mut metrics = HashMap::new();
        metrics.insert("test_counter".to_string(), MetricValue::Counter(42));
        metrics.insert("test_gauge".to_string(), MetricValue::Gauge(3.14));
        
        reporter.report(&metrics).await.unwrap();
        reporter.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_file_reporter() {
        let temp_file = "/tmp/test_metrics.json";
        let reporter = FileReporter::new(temp_file.to_string())
            .with_format(FileFormat::Json);
        
        assert_eq!(reporter.name(), "file");
        
        reporter.start().await.unwrap();
        
        let mut metrics = HashMap::new();
        metrics.insert("test_metric".to_string(), MetricValue::Counter(123));
        
        reporter.report(&metrics).await.unwrap();
        reporter.stop().await.unwrap();
        
        // Clean up
        let _ = tokio::fs::remove_file(temp_file).await;
    }

    #[tokio::test]
    async fn test_reporting_manager() {
        let registry = Arc::new(MetricsRegistry::new());
        let console_reporter = Arc::new(ConsoleReporter::default());
        
        let manager = MetricsReportingManager::new(registry.clone())
            .with_interval(Duration::from_millis(100))
            .add_reporter(console_reporter);
        
        manager.start().await.unwrap();
        
        // Let it run for a short time
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        manager.stop().await.unwrap();
    }
}
