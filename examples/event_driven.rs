//! Event-driven microservices example with pub/sub messaging
//! 
//! This example demonstrates how services can communicate through events,
//! implementing a simple order processing system with multiple services.

use penoos::{ServiceBroker, Service, Context};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> penoos::Result<()> {
    env_logger::init();
    
    println!("📡 Starting Event-Driven Microservices example...");

    let mut broker = ServiceBroker::new()?;

    // Orders service - manages order lifecycle
    let orders_service = Service::new("orders")
        .action("create", |ctx: Context| {
            let order_id = ctx.params().get("orderId").unwrap().as_str().unwrap();
            let user_id = ctx.params().get("userId").unwrap().as_str().unwrap();
            let items = ctx.params().get("items").unwrap().clone();
            
            println!("📋 Order {} created for user {}", order_id, user_id);
            
            Ok(json!({
                "order": {
                    "id": order_id,
                    "userId": user_id,
                    "items": items,
                    "status": "created",
                    "timestamp": chrono::Utc::now().timestamp()
                }
            }))
        })
        .action("update_status", |ctx: Context| {
            let order_id = ctx.params().get("orderId").unwrap().as_str().unwrap();
            let status = ctx.params().get("status").unwrap().as_str().unwrap();
            
            println!("📋 Order {} status updated to: {}", order_id, status);
            
            Ok(json!({
                "order": {
                    "id": order_id,
                    "status": status,
                    "updatedAt": chrono::Utc::now().timestamp()
                }
            }))
        });

    // Inventory service - manages stock
    let inventory_service = Service::new("inventory")
        .action("check_stock", |ctx: Context| {
            let items = ctx.params().get("items").unwrap().as_array().unwrap();
            let mut available = true;
            
            for item in items {
                let product_id = item.get("productId").unwrap().as_str().unwrap();
                let quantity = item.get("quantity").unwrap().as_u64().unwrap();
                println!("📦 Checking stock for product {} (qty: {})", product_id, quantity);
                
                // Simulate stock check (always available for demo)
                if quantity > 10 {
                    available = false;
                    break;
                }
            }
            
            Ok(json!({
                "available": available,
                "checkedAt": chrono::Utc::now().timestamp()
            }))
        })
        .action("reserve", |ctx: Context| {
            let order_id = ctx.params().get("orderId").unwrap().as_str().unwrap();
            let items = ctx.params().get("items").unwrap().clone();
            
            println!("📦 Reserved inventory for order {}", order_id);
            
            Ok(json!({
                "reserved": true,
                "orderId": order_id,
                "items": items,
                "reservedAt": chrono::Utc::now().timestamp()
            }))
        });

    // Payment service - processes payments
    let payment_service = Service::new("payment")
        .action("process", |ctx: Context| {
            let order_id = ctx.params().get("orderId").unwrap().as_str().unwrap();
            let amount = ctx.params().get("amount").unwrap().as_f64().unwrap();
            let user_id = ctx.params().get("userId").unwrap().as_str().unwrap();
            
            println!("💳 Processing payment for order {} (${:.2})", order_id, amount);
            
            // Simulate payment processing (always successful for demo)
            let success = amount > 0.0;
            
            Ok(json!({
                "success": success,
                "orderId": order_id,
                "userId": user_id,
                "amount": amount,
                "transactionId": format!("txn_{}", chrono::Utc::now().timestamp()),
                "processedAt": chrono::Utc::now().timestamp()
            }))
        });

    // Notification service - sends notifications
    let notification_service = Service::new("notification")
        .action("send_email", |ctx: Context| {
            let user_id = ctx.params().get("userId").unwrap().as_str().unwrap();
            let subject = ctx.params().get("subject").unwrap().as_str().unwrap();
            let message = ctx.params().get("message").unwrap().as_str().unwrap();
            
            println!("📧 Sending email to user {}: {}", user_id, subject);
            println!("   Message: {}", message);
            
            Ok(json!({
                "sent": true,
                "userId": user_id,
                "subject": subject,
                "sentAt": chrono::Utc::now().timestamp()
            }))
        })
        .action("send_sms", |ctx: Context| {
            let user_id = ctx.params().get("userId").unwrap().as_str().unwrap();
            let message = ctx.params().get("message").unwrap().as_str().unwrap();
            
            println!("📱 Sending SMS to user {}: {}", user_id, message);
            
            Ok(json!({
                "sent": true,
                "userId": user_id,
                "message": message,
                "sentAt": chrono::Utc::now().timestamp()
            }))
        });

    // Order orchestrator service - coordinates the order flow
    let orchestrator_service = Service::new("orchestrator")
        .action("process_order", |ctx: Context| {
            let order_data = ctx.params().clone();
            let order_id = order_data.get("orderId").unwrap().as_str().unwrap();
            
            println!("🎼 Starting order orchestration for order {}", order_id);
            
            Ok(json!({
                "orchestration": {
                    "orderId": order_id,
                    "status": "started",
                    "startedAt": chrono::Utc::now().timestamp()
                }
            }))
        });

    // Register all services
    broker.create_service(orders_service).await?;
    broker.create_service(inventory_service).await?;
    broker.create_service(payment_service).await?;
    broker.create_service(notification_service).await?;
    broker.create_service(orchestrator_service).await?;

    broker.start().await?;
    println!("✅ All services started successfully!");

    // Simulate order processing workflow
    println!("\n🛒 Simulating order processing workflow...\n");

    let order_data = json!({
        "orderId": "ORD-2025-001",
        "userId": "user123",
        "items": [
            {"productId": "PROD-001", "quantity": 2, "price": 29.99},
            {"productId": "PROD-002", "quantity": 1, "price": 49.99}
        ],
        "totalAmount": 109.97
    });

    // Step 1: Create order
    println!("Step 1: Creating order...");
    let order_result = broker.call("orders.create", order_data.clone()).await?;
    println!("✅ Order created: {}", serde_json::to_string_pretty(&order_result)?);
    sleep(Duration::from_millis(500)).await;

    // Step 2: Check inventory
    println!("\nStep 2: Checking inventory...");
    let inventory_result = broker.call("inventory.check_stock", json!({
        "items": order_data.get("items").unwrap()
    })).await?;
    println!("✅ Inventory check: {}", serde_json::to_string_pretty(&inventory_result)?);
    sleep(Duration::from_millis(500)).await;

    if inventory_result.get("available").unwrap().as_bool().unwrap() {
        // Step 3: Reserve inventory
        println!("\nStep 3: Reserving inventory...");
        let reserve_result = broker.call("inventory.reserve", json!({
            "orderId": order_data.get("orderId").unwrap(),
            "items": order_data.get("items").unwrap()
        })).await?;
        println!("✅ Inventory reserved: {}", serde_json::to_string_pretty(&reserve_result)?);
        sleep(Duration::from_millis(500)).await;

        // Step 4: Process payment
        println!("\nStep 4: Processing payment...");
        let payment_result = broker.call("payment.process", json!({
            "orderId": order_data.get("orderId").unwrap(),
            "userId": order_data.get("userId").unwrap(),
            "amount": order_data.get("totalAmount").unwrap()
        })).await?;
        println!("✅ Payment processed: {}", serde_json::to_string_pretty(&payment_result)?);
        sleep(Duration::from_millis(500)).await;

        if payment_result.get("success").unwrap().as_bool().unwrap() {
            // Step 5: Update order status
            println!("\nStep 5: Updating order status...");
            let status_result = broker.call("orders.update_status", json!({
                "orderId": order_data.get("orderId").unwrap(),
                "status": "confirmed"
            })).await?;
            println!("✅ Order status updated: {}", serde_json::to_string_pretty(&status_result)?);
            sleep(Duration::from_millis(500)).await;

            // Step 6: Send notifications
            println!("\nStep 6: Sending notifications...");
            
            // Send confirmation email
            let email_result = broker.call("notification.send_email", json!({
                "userId": order_data.get("userId").unwrap(),
                "subject": "Order Confirmation",
                "message": format!("Your order {} has been confirmed and will be processed soon.", order_data.get("orderId").unwrap())
            })).await?;
            println!("✅ Email sent: {}", serde_json::to_string_pretty(&email_result)?);
            
            // Send SMS notification
            let sms_result = broker.call("notification.send_sms", json!({
                "userId": order_data.get("userId").unwrap(),
                "message": format!("Order {} confirmed! Total: ${:.2}", order_data.get("orderId").unwrap(), order_data.get("totalAmount").unwrap())
            })).await?;
            println!("✅ SMS sent: {}", serde_json::to_string_pretty(&sms_result)?);

            println!("\n🎉 Order processing completed successfully!");
        } else {
            println!("\n❌ Payment failed, order cancelled");
        }
    } else {
        println!("\n❌ Insufficient inventory, order cancelled");
    }

    sleep(Duration::from_secs(1)).await;
    broker.stop().await?;
    println!("\n🛑 All services stopped. Goodbye!");

    Ok(())
}
