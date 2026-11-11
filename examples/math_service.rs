//! Math Service example for Penoos Rust
//!
//! This example demonstrates a math service with multiple actions,
//! parameter validation, and error handling.
//! Run with: cargo run --example math_service

use penoos::{Context, PenoosError, Result, Service, ServiceBroker};
use serde_json::{Value, json};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    println!("🧮 Starting Penoos Math Service example...");

    // Create a service broker
    let mut broker = ServiceBroker::new()?;

    // Create a math service
    let mut settings = HashMap::new();
    settings.insert("precision".to_string(), Value::Number(2.into()));

    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), Value::String("1.0.0".to_string()));
    metadata.insert(
        "author".to_string(),
        Value::String("Penoos Team".to_string()),
    );

    let math_service = Service::new("math")
        .description("Mathematical operations service")
        .settings(settings)
        .metadata(metadata)
        // Addition action
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

            let result = a + b;
            println!("➕ Addition: {} + {} = {}", a, b, result);

            Ok(json!({
                "result": result,
                "operation": "addition",
                "operands": [a, b]
            }))
        })
        // Subtraction action
        .action("sub", |ctx: Context| {
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

            let result = a - b;
            println!("➖ Subtraction: {} - {} = {}", a, b, result);

            Ok(json!({
                "result": result,
                "operation": "subtraction",
                "operands": [a, b]
            }))
        })
        // Multiplication action
        .action("mult", |ctx: Context| {
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

            let result = a * b;
            println!("✖️ Multiplication: {} * {} = {}", a, b, result);

            Ok(json!({
                "result": result,
                "operation": "multiplication",
                "operands": [a, b]
            }))
        })
        // Division action with error handling
        .action("div", |ctx: Context| {
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

            if b == 0.0 {
                return Err(PenoosError::Custom {
                    code: 422,
                    message: "Division by zero is not allowed".to_string(),
                    data: Some(json!({ "a": a, "b": b })),
                });
            }

            let result = a / b;
            println!("➗ Division: {} / {} = {}", a, b, result);

            Ok(json!({
                "result": result,
                "operation": "division",
                "operands": [a, b]
            }))
        })
        // Power action
        .action("pow", |ctx: Context| {
            let params = ctx.params();

            let base = params.get("base").and_then(|v| v.as_f64()).ok_or_else(|| {
                PenoosError::ValidationError {
                    message: "Parameter 'base' is required and must be a number".to_string(),
                }
            })?;

            let exponent = params
                .get("exponent")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| PenoosError::ValidationError {
                    message: "Parameter 'exponent' is required and must be a number".to_string(),
                })?;

            let result = base.powf(exponent);
            println!("🔢 Power: {} ^ {} = {}", base, exponent, result);

            Ok(json!({
                "result": result,
                "operation": "power",
                "base": base,
                "exponent": exponent
            }))
        })
        // Calculate factorial
        .action("factorial", |ctx: Context| {
            let params = ctx.params();

            let n = params.get("n").and_then(|v| v.as_u64()).ok_or_else(|| {
                PenoosError::ValidationError {
                    message: "Parameter 'n' is required and must be a positive integer".to_string(),
                }
            })?;

            if n > 20 {
                return Err(PenoosError::ValidationError {
                    message: "Factorial calculation is limited to n <= 20 to prevent overflow"
                        .to_string(),
                });
            }

            let mut result = 1u64;
            for i in 1..=n {
                result *= i;
            }

            println!("🔢 Factorial: {}! = {}", n, result);

            Ok(json!({
                "result": result,
                "operation": "factorial",
                "operand": n
            }))
        })
        // Service lifecycle hooks
        .started(|| {
            println!("🟢 Math service started and ready for calculations!");
            Ok(())
        })
        .stopped(|| {
            println!("🔴 Math service stopped");
            Ok(())
        });

    // Register the service
    broker.create_service(math_service).await?;

    // Start the broker
    broker.start().await?;
    println!("✅ Broker started successfully!");

    // Demonstrate various math operations
    println!("\n🧮 Running math operations...\n");

    // Basic arithmetic
    let result = broker.call("math.add", json!({ "a": 5, "b": 3 })).await?;
    println!("Result: {}\n", result);

    let result = broker.call("math.sub", json!({ "a": 10, "b": 4 })).await?;
    println!("Result: {}\n", result);

    let result = broker.call("math.mult", json!({ "a": 6, "b": 7 })).await?;
    println!("Result: {}\n", result);

    let result = broker.call("math.div", json!({ "a": 15, "b": 3 })).await?;
    println!("Result: {}\n", result);

    // Power operation
    let result = broker
        .call("math.pow", json!({ "base": 2, "exponent": 8 }))
        .await?;
    println!("Result: {}\n", result);

    // Factorial
    let result = broker.call("math.factorial", json!({ "n": 5 })).await?;
    println!("Result: {}\n", result);

    // Test error handling
    println!("🚨 Testing error handling...\n");

    // Division by zero
    match broker.call("math.div", json!({ "a": 10, "b": 0 })).await {
        Ok(result) => println!("Unexpected success: {}", result),
        Err(e) => println!("Expected error: {}\n", e),
    }

    // Missing parameter
    match broker.call("math.add", json!({ "a": 5 })).await {
        Ok(result) => println!("Unexpected success: {}", result),
        Err(e) => println!("Expected error: {}\n", e),
    }

    // Invalid factorial input
    match broker.call("math.factorial", json!({ "n": 25 })).await {
        Ok(result) => println!("Unexpected success: {}", result),
        Err(e) => println!("Expected error: {}\n", e),
    }

    // Stop the broker
    broker.stop().await?;
    println!("🛑 Broker stopped. Goodbye!");

    Ok(())
}
