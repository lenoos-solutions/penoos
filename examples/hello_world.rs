//! Hello World example for Penoos Rust
//! 
//! This is the simplest possible Penoos application.
//! Run with: cargo run --example hello_world

use penoos::{ServiceBroker, Service, Context, Result};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    println!("🚀 Starting Penoos Hello World example...");

    // Create a service broker
    let mut broker = ServiceBroker::new()?;

    // Create a simple greeter service
    let greeter_service = Service::new("greeter")
        .action("hello", |ctx: Context| {
            let name = ctx.params()
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("World");
            
            println!("👋 Hello action called with name: {}", name);
            Ok(json!(format!("Hello, {}! 🌟", name)))
        })
        .action("welcome", |ctx: Context| {
            let name = ctx.params()
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Guest");
            
            println!("🎉 Welcome action called with name: {}", name);
            Ok(json!({
                "message": format!("Welcome to Penoos, {}!", name),
                "timestamp": chrono::Utc::now().timestamp(),
                "service": "greeter"
            }))
        });

    // Register the service
    broker.create_service(greeter_service).await?;

    // Start the broker
    broker.start().await?;
    println!("✅ Broker started successfully!");

    // Call the hello action
    let result = broker.call("greeter.hello", json!({ "name": "Rust Developer" })).await?;
    println!("📞 Called greeter.hello: {}", result);

    // Call the welcome action
    let result = broker.call("greeter.welcome", json!({ "name": "Rustacean" })).await?;
    println!("📞 Called greeter.welcome: {}", result);

    // Call without parameters
    let result = broker.call("greeter.hello", json!({})).await?;
    println!("📞 Called greeter.hello (no params): {}", result);

    // Stop the broker
    broker.stop().await?;
    println!("🛑 Broker stopped. Goodbye!");

    Ok(())
}
