//! Parameter validation for actions

use serde_json::Value;

use crate::errors::{PenoosError, Result};

/// Validator trait for validating action parameters
pub trait Validator: Send + Sync {
    /// Validate a value against a schema
    fn validate(&self, value: &Value, schema: &Value) -> Result<()>;
}

/// Simple built-in validator
pub struct SimpleValidator;

impl Validator for SimpleValidator {
    fn validate(&self, value: &Value, schema: &Value) -> Result<()> {
        validate_value(value, schema)
    }
}

/// Validate a value against a simple schema
fn validate_value(value: &Value, schema: &Value) -> Result<()> {
    match schema {
        Value::String(type_name) => {
            validate_type(value, type_name)?;
        }
        Value::Object(schema_obj) => {
            if let Some(type_val) = schema_obj.get("type")
                && let Some(type_name) = type_val.as_str()
            {
                validate_type(value, type_name)?;
            }

            // Check required fields
            if let Some(Value::Bool(true)) = schema_obj.get("required")
                && value.is_null()
            {
                return Err(PenoosError::ValidationError {
                    message: "Field is required".to_string(),
                });
            }

            // Check enum values
            if let Some(enum_values) = schema_obj.get("enum")
                && let Some(enum_array) = enum_values.as_array()
                && !enum_array.contains(value)
            {
                return Err(PenoosError::ValidationError {
                    message: format!("Value must be one of: {:?}", enum_array),
                });
            }

            // Check min/max for numbers
            if let Some(min_val) = schema_obj.get("min")
                && let (Some(num), Some(min)) = (value.as_f64(), min_val.as_f64())
                && num < min
            {
                return Err(PenoosError::ValidationError {
                    message: format!("Value must be >= {}", min),
                });
            }

            if let Some(max_val) = schema_obj.get("max")
                && let (Some(num), Some(max)) = (value.as_f64(), max_val.as_f64())
                && num > max
            {
                return Err(PenoosError::ValidationError {
                    message: format!("Value must be <= {}", max),
                });
            }

            // Check length for strings and arrays
            if let Some(min_length_val) = schema_obj.get("minLength")
                && let Some(min_length) = min_length_val.as_u64()
            {
                let length = match value {
                    Value::String(s) => s.len(),
                    Value::Array(a) => a.len(),
                    _ => 0,
                };
                if length < min_length as usize {
                    return Err(PenoosError::ValidationError {
                        message: format!("Length must be >= {}", min_length),
                    });
                }
            }

            if let Some(max_length_val) = schema_obj.get("maxLength")
                && let Some(max_length) = max_length_val.as_u64()
            {
                let length = match value {
                    Value::String(s) => s.len(),
                    Value::Array(a) => a.len(),
                    _ => 0,
                };
                if length > max_length as usize {
                    return Err(PenoosError::ValidationError {
                        message: format!("Length must be <= {}", max_length),
                    });
                }
            }

            // Validate object properties
            if let Some(properties) = schema_obj.get("properties")
                && let (Some(obj), Some(props)) = (value.as_object(), properties.as_object())
            {
                for (prop_name, prop_schema) in props {
                    if let Some(prop_value) = obj.get(prop_name) {
                        validate_value(prop_value, prop_schema)?;
                    }
                }
            }

            // Validate array items
            if let Some(items_schema) = schema_obj.get("items")
                && let Some(array) = value.as_array()
            {
                for item in array {
                    validate_value(item, items_schema)?;
                }
            }
        }
        _ => {
            return Err(PenoosError::ValidationError {
                message: "Invalid schema format".to_string(),
            });
        }
    }

    Ok(())
}

/// Validate type of a value
fn validate_type(value: &Value, type_name: &str) -> Result<()> {
    let valid = match type_name {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "null" => value.is_null(),
        "any" => true,
        _ => {
            return Err(PenoosError::ValidationError {
                message: format!("Unknown type: {}", type_name),
            });
        }
    };

    if !valid {
        return Err(PenoosError::ValidationError {
            message: format!(
                "Expected type '{}', got '{}'",
                type_name,
                get_value_type(value)
            ),
        });
    }

    Ok(())
}

/// Get the type name of a value
fn get_value_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                "integer"
            } else {
                "number"
            }
        }
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Validator registry for managing different validators
pub struct ValidatorRegistry {
    default: Box<dyn Validator>,
    validators: std::collections::HashMap<String, Box<dyn Validator>>,
}

impl ValidatorRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            default: Box::new(SimpleValidator),
            validators: std::collections::HashMap::new(),
        };

        registry.register("simple", Box::new(SimpleValidator));
        registry
    }

    pub fn register(&mut self, name: &str, validator: Box<dyn Validator>) {
        self.validators.insert(name.to_string(), validator);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Validator> {
        self.validators.get(name).map(|v| v.as_ref())
    }

    pub fn get_default(&self) -> &dyn Validator {
        self.default.as_ref()
    }

    pub fn set_default(&mut self, name: &str) -> Result<()> {
        if let Some(validator) = self.validators.remove(name) {
            let old_default = std::mem::replace(&mut self.default, validator);
            self.validators.insert("simple".to_string(), old_default);
            Ok(())
        } else {
            Err(PenoosError::ValidationError {
                message: format!("Validator '{}' not found", name),
            })
        }
    }

    pub fn list(&self) -> Vec<String> {
        self.validators.keys().cloned().collect()
    }
}

impl Default for ValidatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_type_validation() {
        let validator = SimpleValidator;

        // String validation
        assert!(
            validator
                .validate(&json!("hello"), &json!("string"))
                .is_ok()
        );
        assert!(validator.validate(&json!(42), &json!("string")).is_err());

        // Number validation
        assert!(validator.validate(&json!(42), &json!("number")).is_ok());
        assert!(validator.validate(&json!(42.5), &json!("number")).is_ok());
        assert!(
            validator
                .validate(&json!("hello"), &json!("number"))
                .is_err()
        );

        // Boolean validation
        assert!(validator.validate(&json!(true), &json!("boolean")).is_ok());
        assert!(validator.validate(&json!(false), &json!("boolean")).is_ok());
        assert!(validator.validate(&json!(1), &json!("boolean")).is_err());

        // Array validation
        assert!(
            validator
                .validate(&json!([1, 2, 3]), &json!("array"))
                .is_ok()
        );
        assert!(validator.validate(&json!({}), &json!("array")).is_err());

        // Object validation
        assert!(
            validator
                .validate(&json!({"key": "value"}), &json!("object"))
                .is_ok()
        );
        assert!(validator.validate(&json!([]), &json!("object")).is_err());
    }

    #[test]
    fn test_complex_validation() {
        let validator = SimpleValidator;

        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "minLength": 2, "maxLength": 50},
                "age": {"type": "number", "min": 0, "max": 150},
                "active": {"type": "boolean"},
                "tags": {
                    "type": "array",
                    "items": {"type": "string"}
                }
            }
        });

        let valid_data = json!({
            "name": "John Doe",
            "age": 30,
            "active": true,
            "tags": ["user", "admin"]
        });

        let invalid_data = json!({
            "name": "J", // Too short
            "age": 200, // Too high
            "active": "yes", // Wrong type
            "tags": [123, 456] // Wrong item types
        });

        assert!(validator.validate(&valid_data, &schema).is_ok());
        assert!(validator.validate(&invalid_data, &schema).is_err());
    }

    #[test]
    fn test_enum_validation() {
        let validator = SimpleValidator;

        let schema = json!({
            "type": "string",
            "enum": ["red", "green", "blue"]
        });

        assert!(validator.validate(&json!("red"), &schema).is_ok());
        assert!(validator.validate(&json!("yellow"), &schema).is_err());
    }

    #[test]
    fn test_validator_registry() {
        let registry = ValidatorRegistry::new();

        assert!(registry.get("simple").is_some());
        assert!(registry.get("nonexistent").is_none());

        let validators = registry.list();
        assert!(validators.contains(&"simple".to_string()));
    }
}
