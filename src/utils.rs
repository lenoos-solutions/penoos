//! Utility functions and helpers

use std::time::{SystemTime, UNIX_EPOCH};

use hostname;
use uuid::Uuid;

/// Generate a unique node ID
pub fn generate_node_id() -> String {
    let hostname = hostname::get()
        .unwrap_or_else(|_| std::ffi::OsString::from("unknown"))
        .to_string_lossy()
        .to_string();
    
    let process_id = std::process::id();
    let random: u32 = rand::random();
    
    format!("{}-{}-{}", hostname, process_id, random)
}

/// Generate a unique identifier
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Get current timestamp in milliseconds
pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Convert milliseconds to human-readable duration
pub fn humanize_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.2}s", ms as f64 / 1000.0)
    } else if ms < 3_600_000 {
        format!("{:.2}m", ms as f64 / 60_000.0)
    } else {
        format!("{:.2}h", ms as f64 / 3_600_000.0)
    }
}

/// Parse service name into name and version parts
pub fn parse_service_name(full_name: &str) -> (String, Option<String>) {
    if full_name.starts_with('v') {
        // Find the last dot to separate version from service name
        if let Some(dot_pos) = full_name.rfind('.') {
            let version_part = &full_name[1..dot_pos]; // Skip 'v' and get version
            let name = &full_name[dot_pos + 1..]; // Skip '.' and get name
            (name.to_string(), Some(version_part.to_string()))
        } else {
            (full_name.to_string(), None)
        }
    } else {
        (full_name.to_string(), None)
    }
}

/// Create a versioned service name
pub fn create_versioned_name(name: &str, version: Option<&str>) -> String {
    if let Some(version) = version {
        format!("v{}.{}", version, name)
    } else {
        name.to_string()
    }
}

/// Check if a string matches a pattern with wildcards
pub fn match_pattern(text: &str, pattern: &str) -> bool {
    // Simple pattern matching with * wildcard
    if pattern == "*" {
        return true;
    }
    
    if pattern.contains('*') {
        let regex_pattern = pattern.replace('*', ".*");
        if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            return regex.is_match(text);
        }
    }
    
    text == pattern
}

/// Flatten nested values in a JSON object
pub fn flatten_object(obj: &serde_json::Value, prefix: &str) -> std::collections::HashMap<String, serde_json::Value> {
    let mut result = std::collections::HashMap::new();
    
    match obj {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                let new_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                
                match value {
                    serde_json::Value::Object(_) => {
                        let nested = flatten_object(value, &new_key);
                        result.extend(nested);
                    }
                    _ => {
                        result.insert(new_key, value.clone());
                    }
                }
            }
        }
        _ => {
            result.insert(prefix.to_string(), obj.clone());
        }
    }
    
    result
}

/// Remove duplicates from a vector while preserving order
pub fn remove_duplicates<T: PartialEq + Clone>(vec: Vec<T>) -> Vec<T> {
    let mut result = Vec::new();
    for item in vec {
        if !result.contains(&item) {
            result.push(item);
        }
    }
    result
}

/// Sanitize a string for use as an identifier
pub fn sanitize_identifier(input: &str) -> String {
    input
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect()
}

/// Calculate CPU usage percentage (placeholder implementation)
pub fn get_cpu_usage() -> Option<f64> {
    // This is a placeholder - in a real implementation you'd use
    // a crate like `sysinfo` to get actual CPU usage
    Some(rand::random_range(0.0..100.0))
}

/// Calculate memory usage percentage (placeholder implementation)
pub fn get_memory_usage() -> Option<f64> {
    // This is a placeholder - in a real implementation you'd use
    // a crate like `sysinfo` to get actual memory usage
    Some(rand::random_range(0.0..100.0))
}

/// Deep merge two JSON values
pub fn deep_merge(base: &mut serde_json::Value, other: &serde_json::Value) {
    match (base, other) {
        (serde_json::Value::Object(base_map), serde_json::Value::Object(other_map)) => {
            for (key, value) in other_map {
                if let Some(base_value) = base_map.get_mut(key) {
                    deep_merge(base_value, value);
                } else {
                    base_map.insert(key.clone(), value.clone());
                }
            }
        }
        (base, other) => {
            *base = other.clone();
        }
    }
}

/// Check if a value is empty (null, empty string, empty array, empty object)
pub fn is_empty_value(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => true,
        serde_json::Value::String(s) => s.is_empty(),
        serde_json::Value::Array(a) => a.is_empty(),
        serde_json::Value::Object(o) => o.is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generate_node_id() {
        let id = generate_node_id();
        assert!(!id.is_empty());
        assert!(id.contains('-'));
        
        // Should be unique
        let id2 = generate_node_id();
        assert_ne!(id, id2);
    }

    #[test]
    fn test_humanize_duration() {
        assert_eq!(humanize_duration(500), "500ms");
        assert_eq!(humanize_duration(1500), "1.50s");
        assert_eq!(humanize_duration(75000), "1.25m");
        assert_eq!(humanize_duration(7200000), "2.00h");
    }

    #[test]
    fn test_parse_service_name() {
        assert_eq!(parse_service_name("service"), ("service".to_string(), None));
        assert_eq!(parse_service_name("v1.service"), ("service".to_string(), Some("1".to_string())));
        assert_eq!(parse_service_name("v2.0.service"), ("service".to_string(), Some("2.0".to_string())));
    }

    #[test]
    fn test_create_versioned_name() {
        assert_eq!(create_versioned_name("service", None), "service");
        assert_eq!(create_versioned_name("service", Some("1")), "v1.service");
        assert_eq!(create_versioned_name("service", Some("2.0")), "v2.0.service");
    }

    #[test]
    fn test_match_pattern() {
        assert!(match_pattern("anything", "*"));
        assert!(match_pattern("user.created", "user.*"));
        assert!(match_pattern("user.updated", "user.*"));
        assert!(!match_pattern("post.created", "user.*"));
        assert!(match_pattern("exact.match", "exact.match"));
    }

    #[test]
    fn test_flatten_object() {
        let obj = json!({
            "a": 1,
            "b": {
                "c": 2,
                "d": {
                    "e": 3
                }
            }
        });

        let flattened = flatten_object(&obj, "");
        assert_eq!(flattened.get("a"), Some(&json!(1)));
        assert_eq!(flattened.get("b.c"), Some(&json!(2)));
        assert_eq!(flattened.get("b.d.e"), Some(&json!(3)));
    }

    #[test]
    fn test_remove_duplicates() {
        let vec = vec![1, 2, 3, 2, 4, 1, 5];
        let result = remove_duplicates(vec);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("hello-world"), "hello-world");
        assert_eq!(sanitize_identifier("hello world!"), "hello_world_");
        assert_eq!(sanitize_identifier("test@123"), "test_123");
    }

    #[test]
    fn test_deep_merge() {
        let mut base = json!({
            "a": 1,
            "b": {
                "c": 2
            }
        });

        let other = json!({
            "b": {
                "d": 3
            },
            "e": 4
        });

        deep_merge(&mut base, &other);

        assert_eq!(base, json!({
            "a": 1,
            "b": {
                "c": 2,
                "d": 3
            },
            "e": 4
        }));
    }

    #[test]
    fn test_is_empty_value() {
        assert!(is_empty_value(&json!(null)));
        assert!(is_empty_value(&json!("")));
        assert!(is_empty_value(&json!([])));
        assert!(is_empty_value(&json!({})));
        assert!(!is_empty_value(&json!("hello")));
        assert!(!is_empty_value(&json!(42)));
        assert!(!is_empty_value(&json!([1, 2, 3])));
        assert!(!is_empty_value(&json!({"a": 1})));
    }
}
