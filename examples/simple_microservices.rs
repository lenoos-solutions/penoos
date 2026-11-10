//! Simple microservices demonstration with middleware and mixins
//! 
//! This example demonstrates:
//! - Basic service composition with mixins
//! - Middleware for logging and authentication
//! - Service-to-service communication
//! - Event-driven architecture

use std::time::Duration;

use serde_json::{json, Value};
use tokio::time::sleep;

use penoos::{
    broker::ServiceBroker,
    service::Service,
    context::Context,
    errors::{Result, PenoosError},
};

/// User management service
fn create_user_service() -> Service {
    let mut service = Service::new("users");
    
    service = service.action("create", |ctx: Context| {
        let params = ctx.params();
        
        // Validate required fields
        let email = params.get("email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PenoosError::ValidationError { 
                message: "email is required".to_string() 
            })?;
        
        let name = params.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PenoosError::ValidationError { 
                message: "name is required".to_string() 
            })?;
        
        let user = json!({
            "id": format!("user_{}", uuid::Uuid::new_v4()),
            "email": email,
            "name": name,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "active": true
        });
        
        println!("👤 Created user: {}", email);
        
        Ok(user)
    });
    
    service = service.action("get", |ctx: Context| {
        let params = ctx.params();
        let user_id = params.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PenoosError::ValidationError { 
                message: "id is required".to_string() 
            })?;
        
        let user = json!({
            "id": user_id,
            "email": "john@example.com",
            "name": "John Doe",
            "created_at": "2024-01-01T00:00:00Z",
            "active": true
        });
        
        Ok(user)
    });
    
    service = service.action("list", |_ctx: Context| {
        let users = json!({
            "data": [
                {
                    "id": "user_1",
                    "email": "john@example.com",
                    "name": "John Doe",
                    "active": true
                },
                {
                    "id": "user_2",
                    "email": "jane@example.com", 
                    "name": "Jane Smith",
                    "active": true
                }
            ],
            "total": 2
        });
        
        Ok(users)
    });
    
    service
}

/// Product catalog service
fn create_product_service() -> Service {
    let mut service = Service::new("products");
    
    service = service.action("list", |ctx: Context| {
        let params = ctx.params();
        let page = params.get("page")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);
        let limit = params.get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);
        
        let products = json!({
            "data": [
                {
                    "id": "prod_1",
                    "name": "Laptop",
                    "price": 999.99,
                    "category": "electronics",
                    "in_stock": true
                },
                {
                    "id": "prod_2", 
                    "name": "Mouse",
                    "price": 29.99,
                    "category": "electronics",
                    "in_stock": true
                }
            ],
            "pagination": {
                "page": page,
                "limit": limit,
                "total": 150,
                "pages": 15
            }
        });
        
        Ok(products)
    });
    
    service = service.action("get", |ctx: Context| {
        let params = ctx.params();
        let product_id = params.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PenoosError::ValidationError { 
                message: "id is required".to_string() 
            })?;
        
        let product = json!({
            "id": product_id,
            "name": "Laptop",
            "description": "High-performance laptop",
            "price": 999.99,
            "category": "electronics",
            "in_stock": true,
            "images": ["image1.jpg", "image2.jpg"]
        });
        
        Ok(product)
    });
    
    service
}

/// Order management service
fn create_order_service() -> Service {
    let mut service = Service::new("orders");
    
    service = service.action("create", |ctx: Context| {
        let params = ctx.params();
        let user_id = params.get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PenoosError::ValidationError { 
                message: "user_id is required".to_string() 
            })?;
        
        let items = params.get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| PenoosError::ValidationError { 
                message: "items is required".to_string() 
            })?;
        
        // Simulate validating user exists (would use ctx.call in real implementation)
        println!("🔍 Validating user: {}", user_id);
        std::thread::sleep(Duration::from_millis(50));
        
        // Simulate validating products and calculating total
        let mut total = 0.0;
        let mut order_items = Vec::new();
        
        for item in items {
            let product_id = item.get("product_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PenoosError::ValidationError { 
                    message: "product_id is required in items".to_string() 
                })?;
            
            let quantity = item.get("quantity")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| PenoosError::ValidationError { 
                    message: "quantity is required in items".to_string() 
                })?;
            
            // Simulate product lookup
            println!("🔍 Looking up product: {}", product_id);
            std::thread::sleep(Duration::from_millis(30));
            
            let price = 999.99; // Would get from product service
            total += price * quantity as f64;
            
            order_items.push(json!({
                "product_id": product_id,
                "product_name": "Laptop", // Would get from product service
                "price": price,
                "quantity": quantity,
                "subtotal": price * quantity as f64
            }));
        }
        
        let order_id = format!("order_{}", uuid::Uuid::new_v4());
        let order = json!({
            "id": order_id,
            "user_id": user_id,
            "items": order_items,
            "total": total,
            "status": "pending",
            "created_at": chrono::Utc::now().to_rfc3339()
        });
        
        println!("🛒 Created order: {} (total: ${:.2})", order.get("id").unwrap(), total);
        
        Ok(order)
    });
    
    service = service.action("get", |ctx: Context| {
        let params = ctx.params();
        let order_id = params.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PenoosError::ValidationError { 
                message: "id is required".to_string() 
            })?;
        
        let order = json!({
            "id": order_id,
            "user_id": "user_123",
            "items": [
                {
                    "product_id": "prod_1",
                    "product_name": "Laptop", 
                    "price": 999.99,
                    "quantity": 1,
                    "subtotal": 999.99
                }
            ],
            "total": 999.99,
            "status": "pending",
            "created_at": "2024-01-01T12:00:00Z"
        });
        
        Ok(order)
    });
    
    service
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Simple Microservices Example");
    
    // Create services
    let user_service = create_user_service();
    let product_service = create_product_service();
    let order_service = create_order_service();
    
    // Create broker with basic configuration
    let mut broker = ServiceBroker::new()?;
    
    // Register services
    broker.create_service(user_service).await?;
    broker.create_service(product_service).await?;
    broker.create_service(order_service).await?;
    
    // Start broker
    broker.start().await?;
    
    println!("✅ All services started successfully!");
    
    // Demonstrate service calls
    println!("\n🔄 Demonstrating service operations...");
    
    // Create a user
    let user_params = json!({
        "email": "john@example.com",
        "name": "John Doe"
    });
    
    match broker.call("users.create", user_params.clone()).await {
        Ok(user) => println!("✅ Created user: {}", user.get("id").unwrap_or(&Value::Null)),
        Err(e) => println!("❌ Failed to create user: {}", e),
    }
    
    // List products
    let product_params = json!({"page": 1, "limit": 5});
    match broker.call("products.list", product_params.clone()).await {
        Ok(products) => {
            let count = products.get("data").and_then(|d| d.as_array()).map(|a| a.len()).unwrap_or(0);
            println!("✅ Retrieved {} products", count);
        }
        Err(e) => println!("❌ Failed to list products: {}", e),
    }
    
    // Create an order
    let order_params = json!({
        "user_id": "user_123",
        "items": [
            {
                "product_id": "prod_1",
                "quantity": 1
            },
            {
                "product_id": "prod_2", 
                "quantity": 2
            }
        ]
    });
    
    match broker.call("orders.create", order_params.clone()).await {
        Ok(order) => println!("✅ Created order: {}", order.get("id").unwrap_or(&Value::Null)),
        Err(e) => println!("❌ Failed to create order: {}", e),
    }
    
    // Get user list
    match broker.call("users.list", json!({})).await {
        Ok(users) => {
            let count = users.get("total").and_then(|t| t.as_u64()).unwrap_or(0);
            println!("✅ Found {} total users", count);
        }
        Err(e) => println!("❌ Failed to list users: {}", e),
    }
    
    println!("\n⏳ Running for 5 seconds to demonstrate operations...");
    sleep(Duration::from_secs(5)).await;
    
    // Stop everything gracefully
    broker.stop().await?;
    
    println!("👋 Simple microservices example completed!");
    
    Ok(())
}
