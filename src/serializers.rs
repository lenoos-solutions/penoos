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
        rmp_serde::to_vec(value).map_err(|e| crate::errors::PenoosError::SerializationError {
            message: format!("MessagePack serialization error: {}", e),
        })
    }

    fn deserialize(&self, data: &[u8]) -> Result<Value> {
        rmp_serde::from_slice(data).map_err(|e| crate::errors::PenoosError::SerializationError {
            message: format!("MessagePack deserialization error: {}", e),
        })
    }

    fn content_type(&self) -> &'static str {
        "application/msgpack"
    }
}

/// Self-describing mirror of [`serde_json::Value`] for non-self-describing binary codecs.
///
/// `serde_json::Value` deserializes through `deserialize_any`, which binary formats
/// such as postcard and oxicode do not support, so values are converted to this type first.
#[cfg(any(feature = "oxicode-serializer", feature = "postcard-serializer"))]
#[derive(serde::Serialize, serde::Deserialize)]
enum WireValue {
    Null,
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    String(String),
    Array(Vec<WireValue>),
    Object(std::collections::BTreeMap<String, WireValue>),
}

#[cfg(any(feature = "oxicode-serializer", feature = "postcard-serializer"))]
impl From<&Value> for WireValue {
    fn from(value: &Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(b) => Self::Bool(*b),
            Value::Number(n) => match (n.as_i64(), n.as_u64()) {
                (Some(i), _) => Self::I64(i),
                (None, Some(u)) => Self::U64(u),
                (None, None) => Self::F64(n.as_f64().unwrap_or_default()),
            },
            Value::String(s) => Self::String(s.clone()),
            Value::Array(items) => Self::Array(items.iter().map(Self::from).collect()),
            Value::Object(map) => Self::Object(
                map.iter()
                    .map(|(k, v)| (k.clone(), Self::from(v)))
                    .collect(),
            ),
        }
    }
}

#[cfg(any(feature = "oxicode-serializer", feature = "postcard-serializer"))]
impl From<WireValue> for Value {
    fn from(wire: WireValue) -> Self {
        match wire {
            WireValue::Null => Value::Null,
            WireValue::Bool(b) => Value::Bool(b),
            WireValue::I64(i) => Value::from(i),
            WireValue::U64(u) => Value::from(u),
            WireValue::F64(f) => serde_json::Number::from_f64(f)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            WireValue::String(s) => Value::String(s),
            WireValue::Array(items) => Value::Array(items.into_iter().map(Value::from).collect()),
            WireValue::Object(map) => {
                Value::Object(map.into_iter().map(|(k, v)| (k, Value::from(v))).collect())
            }
        }
    }
}

/// Compact binary serializer using oxicode (serde integration, standard config)
#[cfg(feature = "oxicode-serializer")]
pub struct OxicodeSerializer;

#[cfg(feature = "oxicode-serializer")]
impl Serializer for OxicodeSerializer {
    fn serialize(&self, value: &Value) -> Result<Vec<u8>> {
        oxicode::serde::encode_to_vec(&WireValue::from(value), oxicode::config::standard()).map_err(
            |e| crate::errors::PenoosError::SerializationError {
                message: format!("Oxicode serialization error: {}", e),
            },
        )
    }

    fn deserialize(&self, data: &[u8]) -> Result<Value> {
        let (wire, _) =
            oxicode::serde::decode_from_slice::<WireValue, _>(data, oxicode::config::standard())
                .map_err(|e| crate::errors::PenoosError::SerializationError {
                    message: format!("Oxicode deserialization error: {}", e),
                })?;
        Ok(wire.into())
    }

    fn content_type(&self) -> &'static str {
        "application/octet-stream"
    }
}

/// Binary serializer using wincode (schema-based)
///
/// wincode has no serde integration, so the value is encoded as its JSON text
/// inside a wincode-framed string. The payload is therefore not more compact than JSON.
#[cfg(feature = "wincode-serializer")]
pub struct WincodeSerializer;

#[cfg(feature = "wincode-serializer")]
impl Serializer for WincodeSerializer {
    fn serialize(&self, value: &Value) -> Result<Vec<u8>> {
        let text = serde_json::to_string(value)?;
        wincode::serialize(&text).map_err(|e| crate::errors::PenoosError::SerializationError {
            message: format!("Wincode serialization error: {}", e),
        })
    }

    fn deserialize(&self, data: &[u8]) -> Result<Value> {
        let text: String = wincode::deserialize(data).map_err(|e| {
            crate::errors::PenoosError::SerializationError {
                message: format!("Wincode deserialization error: {}", e),
            }
        })?;
        Ok(serde_json::from_str(&text)?)
    }

    fn content_type(&self) -> &'static str {
        "application/octet-stream"
    }
}

/// Compact binary serializer using postcard
#[cfg(feature = "postcard-serializer")]
pub struct PostcardSerializer;

#[cfg(feature = "postcard-serializer")]
impl Serializer for PostcardSerializer {
    fn serialize(&self, value: &Value) -> Result<Vec<u8>> {
        postcard::to_allocvec(&WireValue::from(value)).map_err(|e| {
            crate::errors::PenoosError::SerializationError {
                message: format!("Postcard serialization error: {}", e),
            }
        })
    }

    fn deserialize(&self, data: &[u8]) -> Result<Value> {
        let wire: WireValue = postcard::from_bytes(data).map_err(|e| {
            crate::errors::PenoosError::SerializationError {
                message: format!("Postcard deserialization error: {}", e),
            }
        })?;
        Ok(wire.into())
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

        #[cfg(feature = "oxicode-serializer")]
        registry.register("oxicode", Box::new(OxicodeSerializer));

        #[cfg(feature = "wincode-serializer")]
        registry.register("wincode", Box::new(WincodeSerializer));

        #[cfg(feature = "postcard-serializer")]
        registry.register("postcard", Box::new(PostcardSerializer));

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
        let registry = SerializerRegistry::new();

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

    #[cfg(any(
        feature = "oxicode-serializer",
        feature = "wincode-serializer",
        feature = "postcard-serializer"
    ))]
    fn nested_value() -> Value {
        json!({
            "null": null,
            "bool": false,
            "negative": -7,
            "unsigned": u64::MAX,
            "float": 2.5,
            "string": "héllo",
            "array": [1, "two", [3], {"four": 4}],
            "object": {"nested": {"deep": true}}
        })
    }

    #[cfg(feature = "oxicode-serializer")]
    #[test]
    fn test_oxicode_round_trip() {
        let serializer = OxicodeSerializer;
        let value = nested_value();

        let bytes = serializer.serialize(&value).unwrap();
        assert_eq!(serializer.deserialize(&bytes).unwrap(), value);
        assert_eq!(serializer.content_type(), "application/octet-stream");
    }

    #[cfg(feature = "oxicode-serializer")]
    #[test]
    fn test_oxicode_rejects_invalid_bytes() {
        let serializer = OxicodeSerializer;
        assert!(serializer.deserialize(&[0xFF, 0xFF, 0xFF]).is_err());
    }

    #[cfg(feature = "wincode-serializer")]
    #[test]
    fn test_wincode_round_trip() {
        let serializer = WincodeSerializer;
        let value = nested_value();

        let bytes = serializer.serialize(&value).unwrap();
        assert_eq!(serializer.deserialize(&bytes).unwrap(), value);
        assert_eq!(serializer.content_type(), "application/octet-stream");
    }

    #[cfg(feature = "wincode-serializer")]
    #[test]
    fn test_wincode_rejects_invalid_bytes() {
        let serializer = WincodeSerializer;
        assert!(serializer.deserialize(&[0xFF]).is_err());
    }

    #[cfg(feature = "postcard-serializer")]
    #[test]
    fn test_postcard_round_trip() {
        let serializer = PostcardSerializer;
        let value = nested_value();

        let bytes = serializer.serialize(&value).unwrap();
        assert_eq!(serializer.deserialize(&bytes).unwrap(), value);
        assert_eq!(serializer.content_type(), "application/octet-stream");
    }

    #[cfg(feature = "postcard-serializer")]
    #[test]
    fn test_postcard_rejects_invalid_bytes() {
        let serializer = PostcardSerializer;
        assert!(serializer.deserialize(&[0xFF, 0xFF, 0xFF]).is_err());
    }

    #[cfg(feature = "postcard-serializer")]
    #[test]
    fn test_registry_exposes_selectable_binary_serializers() {
        let registry = SerializerRegistry::new();
        assert!(registry.get("postcard").is_some());
        assert!(registry.get("bincode").is_none());
    }
}
