/*
 * Penoos Rust Runner
 * Copyright (c) 2024 Penoos Rust Port
 * MIT Licensed
 */

use crate::broker::ServiceBroker;
use crate::errors::PenoosError;
use clap::{Arg, ArgMatches, Command};
use glob::glob;
use serde_json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::signal;
use toml;

/// Penoos Runner - Rust CLI for running Penoos services
#[derive(Default)]
pub struct PenoosRunner {
    /// Parsed command line flags
    flags: Option<ArgMatches>,
    /// Merged configuration
    config: Option<HashMap<String, serde_json::Value>>,
    /// Service file paths to load
    service_paths: Vec<String>,
    /// Service broker instance
    broker: Option<ServiceBroker>,
    /// Folders to watch for hot reload
    watch_folders: Vec<PathBuf>,
}

impl PenoosRunner {
    /// Create a new runner instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Process command line arguments
    /// Returns Ok(true) if processing should continue, Ok(false) if help/version shown
    pub fn process_flags(&mut self, args: Vec<String>) -> Result<bool, PenoosError> {
        let app = Command::new("penoos-runner")
            .about("Penoos microservices framework runner")
            .version("0.1.0")
            .arg(
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .value_name("FILE")
                    .help("Load the configuration from a file")
                    .action(clap::ArgAction::Set),
            )
            .arg(
                Arg::new("repl")
                    .short('r')
                    .long("repl")
                    .help("Start REPL mode")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("hot")
                    .short('H')
                    .long("hot")
                    .help("Hot reload services if changed")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("silent")
                    .short('s')
                    .long("silent")
                    .help("Silent mode. No logger")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("env")
                    .short('e')
                    .long("env")
                    .help("Load .env file from the current directory")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("envfile")
                    .short('E')
                    .long("envfile")
                    .value_name("FILE")
                    .help("Load a specified .env file")
                    .action(clap::ArgAction::Set),
            )
            .arg(
                Arg::new("mask")
                    .short('m')
                    .long("mask")
                    .value_name("PATTERN")
                    .help("Filemask for service loading")
                    .action(clap::ArgAction::Set),
            )
            .arg(
                Arg::new("services")
                    .help("Service files or directories to load")
                    .action(clap::ArgAction::Append)
                    .num_args(0..),
            );

        let matches = match app.try_get_matches_from(args) {
            Ok(matches) => matches,
            Err(err) => match err.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    print!("{}", err);
                    return Ok(false);
                }
                _ => return Err(err.into()),
            },
        };

        // Extract service paths from remaining arguments
        if let Some(services) = matches.get_many::<String>("services") {
            self.service_paths = services.cloned().collect();
        }

        self.flags = Some(matches);
        Ok(true)
    }

    /// Load environment variables from .env file
    pub fn load_env_file(&self) -> Result<(), PenoosError> {
        if let Some(flags) = &self.flags
            && (flags.get_flag("env") || flags.contains_id("envfile"))
        {
            let env_file = if let Some(envfile) = flags.get_one::<String>("envfile") {
                PathBuf::from(envfile)
            } else {
                PathBuf::from(".env")
            };

            if env_file.exists() {
                let content = fs::read_to_string(&env_file).map_err(|e| {
                    PenoosError::config_error(&format!("Failed to read .env file: {}", e))
                })?;

                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    if let Some(eq_pos) = line.find('=') {
                        let key = line[..eq_pos].trim();
                        let value = line[eq_pos + 1..].trim();
                        // SAFETY: set_var races with concurrent env reads. It runs inside the tokio runtime in `_run`, so this is not yet sound.
                        unsafe { env::set_var(key, value) };
                    }
                }
            }
        }
        Ok(())
    }

    /// Load configuration file
    ///
    /// Try to load a configuration file in order:
    /// - load file defined in MOLECULER_CONFIG env var
    /// - try to load file which is defined in CLI option with --config
    /// - try to load the `penoos.config.toml` file if exists in the cwd
    /// - try to load the `penoos.config.json` file if exists in the cwd
    pub async fn load_config_file(
        &self,
    ) -> Result<HashMap<String, serde_json::Value>, PenoosError> {
        let mut config_path: Option<PathBuf> = None;

        // Check environment variable first
        if let Ok(env_config) = env::var("MOLECULER_CONFIG") {
            config_path = Some(PathBuf::from(env_config));
        }

        // Check CLI flag
        if config_path.is_none()
            && let Some(flags) = &self.flags
            && let Some(config_file) = flags.get_one::<String>("config")
        {
            config_path = Some(PathBuf::from(config_file));
        }

        // Try default config files
        if config_path.is_none() {
            for default_config in &["penoos.config.toml", "penoos.config.json"] {
                let path = PathBuf::from(default_config);
                if path.exists() {
                    config_path = Some(path);
                    break;
                }
            }
        }

        let config = if let Some(path) = config_path {
            self.load_config_from_file(&path)?
        } else {
            HashMap::new()
        };

        Ok(config)
    }

    /// Load configuration from a specific file
    fn load_config_from_file(
        &self,
        path: &Path,
    ) -> Result<HashMap<String, serde_json::Value>, PenoosError> {
        let content = fs::read_to_string(path).map_err(|e| {
            PenoosError::config_error(&format!(
                "Failed to read config file {}: {}",
                path.display(),
                e
            ))
        })?;

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => serde_json::from_str(&content).map_err(|e| {
                PenoosError::config_error(&format!("Failed to parse JSON config: {}", e))
            }),
            Some("toml") => {
                let toml_value: toml::Value = toml::from_str(&content).map_err(|e| {
                    PenoosError::config_error(&format!("Failed to parse TOML config: {}", e))
                })?;

                // Convert TOML to JSON value for compatibility
                let json_value = serde_json::to_value(toml_value).map_err(|e| {
                    PenoosError::config_error(&format!("Failed to convert TOML to JSON: {}", e))
                })?;

                if let serde_json::Value::Object(map) = json_value {
                    Ok(map.into_iter().collect())
                } else {
                    Ok(HashMap::new())
                }
            }
            _ => Err(PenoosError::config_error(&format!(
                "Unsupported config file extension: {}",
                path.display()
            ))),
        }
    }

    /// Normalize environment value (convert strings to appropriate types)
    fn normalize_env_value(value: &str) -> serde_json::Value {
        if value.eq_ignore_ascii_case("true") {
            serde_json::Value::Bool(true)
        } else if value.eq_ignore_ascii_case("false") {
            serde_json::Value::Bool(false)
        } else if let Ok(num) = value.parse::<i64>() {
            serde_json::Value::Number(serde_json::Number::from(num))
        } else if let Ok(num) = value.parse::<f64>() {
            serde_json::Number::from_f64(num)
                .map(serde_json::Value::Number)
                .unwrap_or_else(|| serde_json::Value::String(value.to_string()))
        } else {
            serde_json::Value::String(value.to_string())
        }
    }

    /// Overwrite configuration from environment variables
    fn overwrite_from_env(
        &self,
        mut config: HashMap<String, serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
        // Process MOL_ prefixed environment variables
        for (key, value) in env::vars() {
            if let Some(suffix) = key.strip_prefix("MOL_") {
                let config_key = suffix.to_lowercase().replace('_', ".");
                let normalized_value = Self::normalize_env_value(&value);

                // Simple dot notation support - for more complex cases, you'd need a proper dot-set implementation
                if let Some(dot_pos) = config_key.find('.') {
                    let parent_key = &config_key[..dot_pos];
                    let child_key = &config_key[dot_pos + 1..];

                    if let Some(parent_value) = config.get_mut(parent_key) {
                        if let serde_json::Value::Object(obj) = parent_value {
                            obj.insert(child_key.to_string(), normalized_value);
                        }
                    } else {
                        let mut new_obj = serde_json::Map::new();
                        new_obj.insert(child_key.to_string(), normalized_value);
                        config.insert(parent_key.to_string(), serde_json::Value::Object(new_obj));
                    }
                } else {
                    config.insert(config_key, normalized_value);
                }
            }
        }

        config
    }

    /// Merge broker options from config file and environment variables
    pub async fn merge_options(
        &mut self,
        config_from_file: HashMap<String, serde_json::Value>,
    ) -> Result<(), PenoosError> {
        let mut config = config_from_file;

        // Apply environment variable overrides
        config = self.overwrite_from_env(config);

        // Apply CLI flags
        if let Some(flags) = &self.flags {
            if flags.get_flag("silent") {
                config.insert("logger".to_string(), serde_json::Value::Bool(false));
            }

            if flags.get_flag("hot") {
                config.insert("hot_reload".to_string(), serde_json::Value::Bool(true));
            }
        }

        self.config = Some(config);
        Ok(())
    }

    /// Check if path is a directory
    fn is_directory<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_dir()
    }

    /// Check if path is a service file
    fn is_service_file<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_file()
    }

    /// Load services from files or directories
    ///
    /// 1. If find `SERVICEDIR` env var and not find `SERVICES` env var, load all services from the `SERVICEDIR` directory
    /// 2. If find `SERVICEDIR` env var and `SERVICES` env var, load the specified services from the `SERVICEDIR` directory
    /// 3. If not find `SERVICEDIR` env var but find `SERVICES` env var, load the specified services from the current directory
    /// 4. check the CLI arguments. If it find filename(s), load it/them
    /// 5. If find directory(ies), load it/them
    pub async fn load_services(&mut self) -> Result<(), PenoosError> {
        self.watch_folders.clear();

        let file_mask = if let Some(flags) = &self.flags {
            flags.get_one::<String>("mask").cloned()
        } else {
            None
        }
        .unwrap_or_else(|| "**/*.rs".to_string());

        let service_dir = env::var("SERVICEDIR").unwrap_or_else(|_| ".".to_string());
        let service_dir_path = if Path::new(&service_dir).is_absolute() {
            PathBuf::from(service_dir)
        } else {
            env::current_dir()
                .map_err(|e| {
                    PenoosError::config_error(&format!("Failed to get current directory: {}", e))
                })?
                .join(service_dir)
        };

        let mut patterns = self.service_paths.clone();

        // Handle SERVICES and SERVICEDIR environment variables
        if let Ok(services_env) = env::var("SERVICES") {
            patterns = services_env
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        } else if env::var("SERVICEDIR").is_ok() && Self::is_directory(&service_dir_path) {
            // Load all services from directory
            return self
                .load_services_from_dir(&service_dir_path, &file_mask)
                .await;
        }

        if !patterns.is_empty() {
            let mut service_files: Vec<PathBuf> = Vec::new();

            for pattern in patterns {
                let pattern = pattern.trim();
                if pattern.is_empty() {
                    continue;
                }

                let skipping = pattern.starts_with('!');
                let pattern = if skipping { &pattern[1..] } else { pattern };

                if pattern.starts_with("crate:") {
                    // Load from crate module - this would need custom implementation
                    println!(
                        "Warning: crate: loading not yet implemented for pattern: {}",
                        pattern
                    );
                    continue;
                }

                let service_path = if Path::new(pattern).is_absolute() {
                    PathBuf::from(pattern)
                } else {
                    service_dir_path.join(pattern)
                };

                let mut files: Vec<PathBuf> = Vec::new();

                if Self::is_directory(&service_path) {
                    if self
                        .config
                        .as_ref()
                        .and_then(|c| c.get("hot_reload"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        self.watch_folders.push(service_path.clone());
                    }

                    // Find service files in directory
                    let glob_pattern = format!("{}/{}", service_path.display(), file_mask);
                    for path in glob(&glob_pattern)
                        .map_err(|e| {
                            PenoosError::config_error(&format!("Invalid glob pattern: {}", e))
                        })?
                        .flatten()
                    {
                        files.push(path);
                    }

                    if files.is_empty() {
                        println!(
                            "Warning: No service files found in directory: {}",
                            service_path.display()
                        );
                    }
                } else if Self::is_service_file(&service_path) {
                    files.push(service_path);
                } else {
                    // Try with .rs extension
                    let rs_path = service_path.with_extension("rs");
                    if Self::is_service_file(&rs_path) {
                        files.push(rs_path);
                    } else {
                        // Try glob pattern
                        for path in glob(pattern)
                            .map_err(|e| {
                                PenoosError::config_error(&format!("Invalid glob pattern: {}", e))
                            })?
                            .flatten()
                        {
                            files.push(path);
                        }

                        if files.is_empty() {
                            println!("Warning: No files matched pattern: {}", pattern);
                        }
                    }
                }

                if !files.is_empty() {
                    if skipping {
                        service_files.retain(|f| !files.contains(f));
                    } else {
                        service_files.extend(files);
                    }
                }
            }

            // Remove duplicates and load services
            service_files.sort();
            service_files.dedup();

            for file_path in service_files {
                self.load_service_file(&file_path).await?;
            }
        }

        Ok(())
    }

    /// Load services from a directory
    async fn load_services_from_dir(
        &mut self,
        dir: &Path,
        file_mask: &str,
    ) -> Result<(), PenoosError> {
        if self
            .config
            .as_ref()
            .and_then(|c| c.get("hot_reload"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            self.watch_folders.push(dir.to_path_buf());
        }

        let glob_pattern = format!("{}/{}", dir.display(), file_mask);
        for path in glob(&glob_pattern)
            .map_err(|e| PenoosError::config_error(&format!("Invalid glob pattern: {}", e)))?
            .flatten()
        {
            self.load_service_file(&path).await?;
        }

        Ok(())
    }

    /// Load a service from a file
    async fn load_service_file(&self, _path: &Path) -> Result<(), PenoosError> {
        // In a real implementation, this would dynamically load Rust services
        // For now, we'll just print that we're loading them
        println!("Loading service from: {}", _path.display());

        // TODO: Implement dynamic service loading
        // This is complex in Rust and would likely require:
        // 1. Dynamic library loading (.so/.dylib/.dll files)
        // 2. A plugin system with defined interfaces
        // 3. Or compilation at runtime

        Ok(())
    }

    /// Start the Penoos broker
    pub async fn start_broker(&mut self) -> Result<(), PenoosError> {
        // Create broker configuration from our config
        let _config = self.config.as_ref().cloned().unwrap_or_default();

        // Create service broker
        let mut broker = ServiceBroker::new()?;

        // Load services
        self.load_services().await?;

        // Start the broker
        broker.start().await?;

        self.broker = Some(broker);

        // TODO: Implement REPL mode if requested
        if let Some(flags) = &self.flags
            && flags.get_flag("repl")
        {
            println!("REPL mode not yet implemented");
        }

        Ok(())
    }

    /// Internal run method
    async fn _run(&mut self) -> Result<(), PenoosError> {
        self.load_env_file()?;
        let config = self.load_config_file().await?;
        self.merge_options(config).await?;
        self.start_broker().await
    }

    /// Start the runner
    pub async fn start(&mut self, args: Vec<String>) -> Result<bool, PenoosError> {
        // Returns true if we should continue running, false if help/version was shown
        let should_continue = self.process_flags(args)?;
        if !should_continue {
            return Ok(false);
        }

        // For now, we don't implement clustering like the Node.js version
        // Just run the broker directly
        self._run().await?;
        Ok(true)
    }

    /// Wait for shutdown signals
    pub async fn wait_for_shutdown(&mut self) -> Result<(), PenoosError> {
        // Wait for Ctrl+C
        signal::ctrl_c().await.map_err(|e| {
            PenoosError::runtime_error(&format!("Failed to listen for ctrl-c: {}", e))
        })?;

        println!("Received shutdown signal, stopping broker...");

        if let Some(broker) = &mut self.broker {
            broker.stop().await?;
        }

        println!("Broker stopped successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_creation() {
        let runner = PenoosRunner::new();
        assert!(runner.flags.is_none());
        assert!(runner.config.is_none());
        assert!(runner.service_paths.is_empty());
    }

    #[test]
    fn test_normalize_env_value() {
        assert_eq!(
            PenoosRunner::normalize_env_value("true"),
            serde_json::Value::Bool(true)
        );
        assert_eq!(
            PenoosRunner::normalize_env_value("false"),
            serde_json::Value::Bool(false)
        );
        assert_eq!(
            PenoosRunner::normalize_env_value("123"),
            serde_json::Value::Number(serde_json::Number::from(123))
        );
        assert_eq!(
            PenoosRunner::normalize_env_value("hello"),
            serde_json::Value::String("hello".to_string())
        );
    }

    // Test without async (proc-macro issue)
    #[test]
    fn test_process_flags_sync() {
        let mut runner = PenoosRunner::new();
        let args = vec![
            "penoos-runner".to_string(),
            "--config".to_string(),
            "test.config".to_string(),
            "--silent".to_string(),
            "service1.rs".to_string(),
            "service2.rs".to_string(),
        ];

        runner.process_flags(args).unwrap();

        assert!(runner.flags.is_some());
        let flags = runner.flags.as_ref().unwrap();
        assert_eq!(
            flags.get_one::<String>("config"),
            Some(&"test.config".to_string())
        );
        assert!(flags.get_flag("silent"));
        assert_eq!(runner.service_paths, vec!["service1.rs", "service2.rs"]);
    }
}
