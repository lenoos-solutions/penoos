//! Serializers for data transformation

use serde_json::Value;

use crate::errors::Result;

/// Serializer trait for converting data between formats
pub trait Serializer: Send + Sync {
    /// Serialize a value to bytes
    fn serialize(&self, value: &Value) -> Result<Vec<u8>>;
    
    /// Deserialize bytes to a value
    fn deserialize(&self, data: &[u8]) -> Result<Value>;
    
    /// Get the content type for this serializer
    fn content_type(&self) -> &'static str;
}

/// JSON serializer (default)
pub struct JsonSerializer;

impl Serializer for JsonSerializer {
    fn serialize(&self, value: &Value) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(value)?)
    }

    fn deserialize(&self, data: &[u8]) -> Result<Value> {
        Ok(serde_json::from_slice(data)?)
    }

    fn content_type(&self) -> &'static str {
        "application/json"
    }
}

/// MessagePack serializer
#[cfg(feature = "msgpack")]
pub struct MessagePackSerializer;

#[cfg(feature = "msgpack")]
impl Serializer for MessagePackSerializer {
    fn serialize(&self, value: &Value) -> Result<Vec<u8>> {
        use rmp_serde;
        rmp_serde::to_vec(value).map_err(|e| {
            crate::errors::PenoosError::SerializationError {
                message: format!("MessagePack serialization error: {}", e),
            }
        })
    }

    fn deserialize(&self, data: &[u8]) -> Result<Value> {
        use rmp_serde;
        rmp_serde::from_slice(data).map_err(|e| {
            crate::errors::PenoosError::SerializationError {
                message: format!("MessagePack deserialization error: {}", e),
            }
        })
    }

    fn content_type(&self) -> &'static str {
        "application/msgpack"
    }
}

/// Binary serializer using bincode
#[cfg(feature = "bincode-serializer")]
pub struct BincodeSerializer;

#[cfg(feature = "bincode-serializer")]
impl Serializer for BincodeSerializer {
    fn serialize(&self, value: &Value) -> Result<Vec<u8>> {
        bincode::serialize(value).map_err(|e| {
            crate::errors::PenoosError::SerializationError {
                message: format!("Bincode serialization error: {}", e),
            }
        })
    }

    fn deserialize(&self, data: &[u8]) -> Result<Value> {
        bincode::deserialize(data).map_err(|e| {
            crate::errors::PenoosError::SerializationError {
                message: format!("Bincode deserialization error: {}", e),
            }
        })
    }

    fn content_type(&self) -> &'static str {
        "application/octet-stream"
    }
}

/// Serializer registry for managing different serializers
pub struct SerializerRegistry {
    default: Box<dyn Serializer>,
    serializers: std::collections::HashMap<String, Box<dyn Serializer>>,
}

impl SerializerRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            default: Box::new(JsonSerializer),
            serializers: std::collections::HashMap::new(),
        };
        
        registry.register("json", Box::new(JsonSerializer));
        
        #[cfg(feature = "msgpack")]
        registry.register("msgpack", Box::new(MessagePackSerializer));
        
        #[cfg(feature = "bincode-serializer")]
        registry.register("bincode", Box::new(BincodeSerializer));
        
        registry
    }

    pub fn register(&mut self, name: &str, serializer: Box<dyn Serializer>) {
        self.serializers.insert(name.to_string(), serializer);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Serializer> {
        self.serializers.get(name).map(|s| s.as_ref())
    }

    pub fn get_default(&self) -> &dyn Serializer {
        self.default.as_ref()
    }

    pub fn set_default(&mut self, name: &str) -> Result<()> {
        if let Some(serializer) = self.serializers.remove(name) {
            let old_default = std::mem::replace(&mut self.default, serializer);
            self.serializers.insert("json".to_string(), old_default);
            Ok(())
        } else {
            Err(crate::errors::PenoosError::SerializationError {
                message: format!("Serializer '{}' not found", name),
            })
        }
    }

    pub fn list(&self) -> Vec<String> {
        self.serializers.keys().cloned().collect()
    }
}

impl Default for SerializerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_serializer() {
        let serializer = JsonSerializer;
        let value = json!({"name": "test", "age": 30});

        let serialized = serializer.serialize(&value).unwrap();
        let deserialized = serializer.deserialize(&serialized).unwrap();

        assert_eq!(value, deserialized);
        assert_eq!(serializer.content_type(), "application/json");
    }

    #[test]
    fn test_serializer_registry() {
        let mut registry = SerializerRegistry::new();
        
        assert!(registry.get("json").is_some());
        assert!(registry.get("nonexistent").is_none());
        
        let serializers = registry.list();
        assert!(serializers.contains(&"json".to_string()));
    }

    #[test]
    fn test_serializer_round_trip() {
        let registry = SerializerRegistry::new();
        let json_serializer = registry.get("json").unwrap();
        
        let original = json!({
            "string": "hello",
            "number": 42,
            "boolean": true,
            "array": [1, 2, 3],
            "object": {"nested": "value"}
        });

        let serialized = json_serializer.serialize(&original).unwrap();
        let deserialized = json_serializer.deserialize(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }
}
