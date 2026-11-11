//! Working microservices example
//!
//! This example demonstrates basic Penoos functionality:
//! - Service creation and registration
//! - Action definitions and calls
//! - Service communication

use std::time::Duration;

use serde_json::json;
use tokio::time::sleep;

use penoos::{
    broker::ServiceBroker,
    context::Context,
    errors::{PenoosError, Result},
    service::Service,
};

/// Math service with basic operations
fn create_math_service() -> Service {
    Service::new("math")
        .action("add", |ctx: Context| {
            let params = ctx.params();
            let a = params.get("a").and_then(|v| v.as_f64()).ok_or_else(|| {
                PenoosError::ValidationError {
                    message: "Parameter 'a' is required and must be a number".to_string(),
                }
            })?;
            let b = params.get("b").and_then(|v| v.as_f64()).ok_or_else(|| {
                PenoosError::ValidationError {
                    message: "Parameter 'b' is required and must be a number".to_string(),
                }
            })?;

            Ok(json!(a + b))
        })
        .action("multiply", |ctx: Context| {
            let params = ctx.params();
            let a = params.get("a").and_then(|v| v.as_f64()).ok_or_else(|| {
                PenoosError::ValidationError {
                    message: "Parameter 'a' is required and must be a number".to_string(),
                }
            })?;
            let b = params.get("b").and_then(|v| v.as_f64()).ok_or_else(|| {
                PenoosError::ValidationError {
                    message: "Parameter 'b' is required and must be a number".to_string(),
                }
            })?;

            Ok(json!(a * b))
        })
        .action("factorial", |ctx: Context| {
            let params = ctx.params();
            let n = params.get("n").and_then(|v| v.as_u64()).ok_or_else(|| {
                PenoosError::ValidationError {
                    message: "Parameter 'n' is required and must be a positive integer".to_string(),
                }
            })?;

            if n > 20 {
                return Err(PenoosError::ValidationError {
                    message: "Number too large (max 20)".to_string(),
                });
            }

            let mut result = 1u64;
            for i in 1..=n {
                result *= i;
            }

            Ok(json!(result))
        })
}

/// String utility service
fn create_string_service() -> Service {
    Service::new("string")
        .action("uppercase", |ctx: Context| {
            let params = ctx.params();
            let text = params.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
                PenoosError::ValidationError {
                    message: "Parameter 'text' is required and must be a string".to_string(),
                }
            })?;

            Ok(json!(text.to_uppercase()))
        })
        .action("lowercase", |ctx: Context| {
            let params = ctx.params();
            let text = params.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
                PenoosError::ValidationError {
                    message: "Parameter 'text' is required and must be a string".to_string(),
                }
            })?;

            Ok(json!(text.to_lowercase()))
        })
        .action("reverse", |ctx: Context| {
            let params = ctx.params();
            let text = params.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
                PenoosError::ValidationError {
                    message: "Parameter 'text' is required and must be a string".to_string(),
                }
            })?;

            let reversed: String = text.chars().rev().collect();
            Ok(json!(reversed))
        })
        .action("length", |ctx: Context| {
            let params = ctx.params();
            let text = params.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
                PenoosError::ValidationError {
                    message: "Parameter 'text' is required and must be a string".to_string(),
                }
            })?;

            Ok(json!(text.len()))
        })
}

/// Greeter service
fn create_greeter_service() -> Service {
    Service::new("greeter")
        .action("hello", |ctx: Context| {
            let params = ctx.params();
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("World");

            Ok(json!(format!("Hello, {}!", name)))
        })
        .action("goodbye", |ctx: Context| {
            let params = ctx.params();
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("friend");

            Ok(json!(format!("Goodbye, {}!", name)))
        })
        .action("greet_many", |ctx: Context| {
            let params = ctx.params();
            let names = params
                .get("names")
                .and_then(|v| v.as_array())
                .ok_or_else(|| PenoosError::ValidationError {
                    message: "Parameter 'names' is required and must be an array".to_string(),
                })?;

            let greetings: Vec<String> = names
                .iter()
                .filter_map(|v| v.as_str())
                .map(|name| format!("Hello, {}!", name))
                .collect();

            Ok(json!({
                "greetings": greetings,
                "total": greetings.len()
            }))
        })
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Working Microservices Example");

    // Create services
    let math_service = create_math_service();
    let string_service = create_string_service();
    let greeter_service = create_greeter_service();

    // Create broker
    let mut broker = ServiceBroker::new()?;

    // Register services
    broker.create_service(math_service).await?;
    broker.create_service(string_service).await?;
    broker.create_service(greeter_service).await?;

    // Start broker
    broker.start().await?;

    println!("✅ All services started successfully!");
    println!("📋 Available services: math, string, greeter");

    // Demonstrate math operations
    println!("\n🧮 Math Operations:");

    let add_result = broker.call("math.add", json!({"a": 5, "b": 3})).await?;
    println!("  5 + 3 = {}", add_result);

    let multiply_result = broker
        .call("math.multiply", json!({"a": 4, "b": 7}))
        .await?;
    println!("  4 × 7 = {}", multiply_result);

    let factorial_result = broker.call("math.factorial", json!({"n": 5})).await?;
    println!("  5! = {}", factorial_result);

    // Demonstrate string operations
    println!("\n📝 String Operations:");

    let text = "Hello World";
    let uppercase_result = broker
        .call("string.uppercase", json!({"text": text}))
        .await?;
    println!("  '{}' → uppercase: {}", text, uppercase_result);

    let lowercase_result = broker
        .call("string.lowercase", json!({"text": text}))
        .await?;
    println!("  '{}' → lowercase: {}", text, lowercase_result);

    let reverse_result = broker.call("string.reverse", json!({"text": text})).await?;
    println!("  '{}' → reversed: {}", text, reverse_result);

    let length_result = broker.call("string.length", json!({"text": text})).await?;
    println!("  '{}' → length: {}", text, length_result);

    // Demonstrate greeter operations
    println!("\n👋 Greeting Operations:");

    let hello_result = broker
        .call("greeter.hello", json!({"name": "Alice"}))
        .await?;
    println!("  {}", hello_result.as_str().unwrap_or(""));

    let goodbye_result = broker
        .call("greeter.goodbye", json!({"name": "Bob"}))
        .await?;
    println!("  {}", goodbye_result.as_str().unwrap_or(""));

    let greet_many_result = broker
        .call(
            "greeter.greet_many",
            json!({"names": ["Alice", "Bob", "Charlie"]}),
        )
        .await?;
    println!("  Bulk greetings: {}", greet_many_result);

    // Test error handling
    println!("\n❌ Error Handling:");

    match broker.call("math.add", json!({"a": "invalid"})).await {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Expected error: {}", e),
    }

    match broker.call("unknown.action", json!({})).await {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Expected error: {}", e),
    }

    // Performance test
    println!("\n⚡ Performance Test:");
    let start = std::time::Instant::now();
    let mut total = 0.0;

    for i in 1..=100 {
        let result = broker.call("math.add", json!({"a": i, "b": i})).await?;
        if let Some(value) = result.as_f64() {
            total += value;
        }
    }

    let duration = start.elapsed();
    println!("  Performed 100 math operations in {:?}", duration);
    println!("  Total sum: {}", total);
    println!("  Avg time per call: {:?}", duration / 100);

    println!("\n⏳ Running for 3 seconds...");
    sleep(Duration::from_secs(3)).await;

    // Stop broker
    broker.stop().await?;

    println!("👋 Working microservices example completed!");

    Ok(())
}
