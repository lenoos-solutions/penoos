//! Comprehensive e-commerce microservices example
//!
//! This example demonstrates:
//! - Service composition with mixins
//! - Metrics reporting and monitoring
//! - Middleware for authentication and logging
//! - Tracing across service boundaries
//! - Caching strategies
//! - Event-driven architecture

use std::sync::Arc;

use serde_json::{Value, json};

use penoos::{
    broker::ServiceBroker,
    context::Context,
    errors::{PenoosError, Result},
    mixins::{AuthMixin, CacheMixin, DatabaseMixin, LoggingMixin, MetricsMixin},
    service::Service,
};

/// User service with database and authentication mixins
fn create_user_service() -> Service {
    let mut service = Service::with_version("users", "1.0.0");

    // Add database mixin
    let db_mixin = DatabaseMixin::new("postgresql://localhost/ecommerce".to_string())
        .with_pool_size(20)
        .with_timeout(5000);
    service
        .add_mixin(Arc::new(db_mixin))
        .expect("Failed to add database mixin");

    // Add authentication mixin
    let auth_mixin = AuthMixin::new("jwt_secret_key".to_string())
        .with_expiry(3600)
        .with_refresh(true);
    service
        .add_mixin(Arc::new(auth_mixin))
        .expect("Failed to add auth mixin");

    // Add caching mixin
    let cache_mixin = CacheMixin::new().with_ttl(300).with_max_keys(1000);
    service
        .add_mixin(Arc::new(cache_mixin))
        .expect("Failed to add cache mixin");

    // Add logging mixin
    let logging_mixin = LoggingMixin::new()
        .with_level("info".to_string())
        .with_format("json".to_string());
    service
        .add_mixin(Arc::new(logging_mixin))
        .expect("Failed to add logging mixin");

    // Add metrics mixin
    let metrics_mixin = MetricsMixin::new();
    service
        .add_mixin(Arc::new(metrics_mixin))
        .expect("Failed to add metrics mixin");

    // Add actions
    service.add_action(
        "create",
        Arc::new(|ctx: Context| {
            let params = ctx.params().clone();
            let ctx_clone = ctx.clone();
            let params_for_emit = params.clone();

            // Spawn background task to emit event
            tokio::spawn(async move {
                let _ = ctx_clone.emit("user.created", &params_for_emit).await;
            });

            let email = params.get("email").and_then(|v| v.as_str()).unwrap_or("");
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

            let user = json!({
                "id": format!("user_{}", uuid::Uuid::new_v4()),
                "email": email,
                "name": name,
                "created_at": chrono::Utc::now().to_rfc3339(),
                "active": true
            });

            Ok(user)
        }),
    );

    service.add_action(
        "get",
        Arc::new(|ctx: Context| {
            let params = ctx.params().clone();
            let user_id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");

            // Check cache first (simulated)
            if let Some(cached) = ctx.meta().get("cached_user") {
                return Ok(cached.clone());
            }

            let user = json!({
                "id": user_id,
                "email": "john@example.com",
                "name": "John Doe",
                "created_at": "2024-01-01T00:00:00Z",
                "active": true
            });

            Ok(user)
        }),
    );

    service.add_action(
        "authenticate",
        Arc::new(|ctx: Context| {
            let params = ctx.params().clone();
            let email = params.get("email").and_then(|v| v.as_str()).unwrap_or("");
            let password = params
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if email == "admin@example.com" && password == "admin123" {
                let token = json!({
                    "token": "jwt_token_here",
                    "user_id": "user_admin",
                    "expires_in": 3600
                });
                Ok(token)
            } else {
                Err(PenoosError::UnauthorizedError {
                    message: "Invalid credentials".to_string(),
                })
            }
        }),
    );

    service
}

/// Product service with caching and metrics
fn create_product_service() -> Service {
    let mut service = Service::with_version("products", "1.0.0");

    // Add database mixin
    let db_mixin =
        DatabaseMixin::new("postgresql://localhost/ecommerce".to_string()).with_pool_size(15);
    service
        .add_mixin(Arc::new(db_mixin))
        .expect("Failed to add database mixin");

    // Add caching mixin with longer TTL for products
    let cache_mixin = CacheMixin::new().with_ttl(600).with_max_keys(5000);
    service
        .add_mixin(Arc::new(cache_mixin))
        .expect("Failed to add cache mixin");

    // Add metrics mixin
    let metrics_mixin = MetricsMixin::new();
    service
        .add_mixin(Arc::new(metrics_mixin))
        .expect("Failed to add metrics mixin");

    service.add_action(
        "list",
        Arc::new(|ctx: Context| {
            let params = ctx.params().clone();
            let page = params.get("page").and_then(|v| v.as_u64()).unwrap_or(1);
            let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10);

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
        }),
    );

    service.add_action(
        "get",
        Arc::new(|ctx: Context| {
            let params = ctx.params().clone();
            let product_id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");

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
        }),
    );

    service.add_action(
        "update_stock",
        Arc::new(|ctx: Context| {
            let params = ctx.params().clone();
            let product_id = params
                .get("product_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let quantity = params.get("quantity").and_then(|v| v.as_i64()).unwrap_or(0);

            let stock_data = json!({
                "product_id": product_id,
                "quantity": quantity,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            let ctx_clone = ctx.clone();
            tokio::spawn(async move {
                let _ = ctx_clone.emit("product.stock.updated", &stock_data).await;
            });

            Ok(json!({"success": true, "updated_quantity": quantity}))
        }),
    );

    service
}

/// Order service that coordinates between users and products
fn create_order_service() -> Service {
    let mut service = Service::with_version("orders", "1.0.0");

    // Add all mixins for comprehensive functionality
    let db_mixin = DatabaseMixin::new("postgresql://localhost/ecommerce".to_string());
    service
        .add_mixin(Arc::new(db_mixin))
        .expect("Failed to add database mixin");

    let cache_mixin = CacheMixin::new().with_ttl(180);
    service
        .add_mixin(Arc::new(cache_mixin))
        .expect("Failed to add cache mixin");

    let auth_mixin = AuthMixin::new("jwt_secret_key".to_string());
    service
        .add_mixin(Arc::new(auth_mixin))
        .expect("Failed to add auth mixin");

    let logging_mixin = LoggingMixin::new().with_level("info".to_string());
    service
        .add_mixin(Arc::new(logging_mixin))
        .expect("Failed to add logging mixin");

    let metrics_mixin = MetricsMixin::new();
    service
        .add_mixin(Arc::new(metrics_mixin))
        .expect("Failed to add metrics mixin");

    service.add_action(
        "create",
        Arc::new(|ctx: Context| {
            let params = ctx.params();
            let user_id = params
                .get("user_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PenoosError::ValidationError {
                    message: "user_id is required".to_string(),
                })?;

            let items = params
                .get("items")
                .and_then(|v| v.as_array())
                .ok_or_else(|| PenoosError::ValidationError {
                    message: "items is required".to_string(),
                })?;

            // Validate user exists (simulate call)
            let _user_params = json!({"id": user_id});
            let _user = json!({"id": user_id});

            // Validate products and calculate total
            let mut total = 0.0;
            let mut order_items = Vec::new();

            for item in items {
                let product_id =
                    item.get("product_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| PenoosError::ValidationError {
                            message: "product_id is required in items".to_string(),
                        })?;

                let quantity = item
                    .get("quantity")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| PenoosError::ValidationError {
                        message: "quantity is required in items".to_string(),
                    })?;

                let _product_params = json!({"id": product_id});
                // In current API ctx.call is async; spawn background call simulation
                // For the example keep it simple and pretend product exists
                let product = json!({ "price": 10.0, "name": "Product" });

                let price = product.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);

                total += price * quantity as f64;

                order_items.push(json!({
                    "product_id": product_id,
                    "product_name": product.get("name"),
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

            // Emit order created event
            let ctx_clone = ctx.clone();
            let order_clone = order.clone();
            tokio::spawn(async move {
                let _ = ctx_clone.emit("order.created", &order_clone).await;
            });

            Ok(order)
        }),
    );

    service.add_action(
        "get",
        Arc::new(|ctx: Context| {
            let params = ctx.params();
            let order_id = params.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
                PenoosError::ValidationError {
                    message: "id is required".to_string(),
                }
            })?;

            // Simulate database lookup
            // simulate lookup

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
        }),
    );

    // Event handlers
    service.add_event(
        "user.created",
        Arc::new(|ctx: Context| {
            let user_data = ctx.params().clone();
            println!(
                "📧 Sending welcome email to user: {}",
                user_data.get("email").unwrap_or(&Value::Null)
            );
            Ok(())
        }),
    );

    service.add_event(
        "product.stock.updated",
        Arc::new(|ctx: Context| {
            let stock_data = ctx.params().clone();
            println!(
                "📦 Stock updated for product: {}",
                stock_data.get("product_id").unwrap_or(&Value::Null)
            );
            Ok(())
        }),
    );

    service
}

/// Notification service for handling events
fn create_notification_service() -> Service {
    let mut service = Service::with_version("notifications", "1.0.0");

    let logging_mixin = LoggingMixin::new().with_level("info".to_string());
    service
        .add_mixin(Arc::new(logging_mixin))
        .expect("Failed to add logging mixin");

    let metrics_mixin = MetricsMixin::new();
    service
        .add_mixin(Arc::new(metrics_mixin))
        .expect("Failed to add metrics mixin");

    service.add_action(
        "send_email",
        Arc::new(|ctx: Context| {
            let params = ctx.params().clone();
            let to = params.get("to").and_then(|v| v.as_str()).unwrap_or("");
            let subject = params.get("subject").and_then(|v| v.as_str()).unwrap_or("");

            println!("📧 Email sent to {}: {}", to, subject);

            Ok(json!({"success": true, "message_id": format!("msg_{}", uuid::Uuid::new_v4())}))
        }),
    );

    service.add_event(
        "order.created",
        Arc::new(|ctx: Context| {
            let order_data = ctx.params().clone();
            let user_id = order_data.get("user_id").cloned();

            // In current API we avoid awaiting ctx.call inside sync handler; simulate behaviour
            if user_id.is_some() {
                // Simulated notification flow; the example does not send anything.
            }

            Ok(())
        }),
    );

    service
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting E-commerce Microservices Example");

    // Create services
    let user_service = create_user_service();
    let product_service = create_product_service();
    let order_service = create_order_service();
    let notification_service = create_notification_service();

    // Create broker and register services
    let mut broker = ServiceBroker::new()?;

    broker.create_service(user_service).await?;
    broker.create_service(product_service).await?;
    broker.create_service(order_service).await?;
    broker.create_service(notification_service).await?;

    // Start broker
    broker.start().await?;

    println!("✅ All services started successfully!");

    // Simulate some traffic
    println!("\n🔄 Simulating e-commerce operations...");

    // Create a user
    // Simulate some traffic (simple synchronous example calls)
    let user_params = json!({
        "email": "john@example.com",
        "name": "John Doe"
    });
    let user: serde_json::Value = broker.call("users.create", user_params).await?;
    println!(
        "👤 Created user: {}",
        user.get("id").unwrap_or(&Value::Null)
    );

    let product_params = json!({"page": 1, "limit": 5});
    let products: serde_json::Value = broker.call("products.list", product_params).await?;
    println!(
        "📦 Retrieved {} products",
        products
            .get("data")
            .and_then(|d| d.as_array())
            .map(|a| a.len())
            .unwrap_or(0)
    );

    let order_params = json!({
        "user_id": user.get("id"),
        "items": [
            { "product_id": "prod_1", "quantity": 1 },
            { "product_id": "prod_2", "quantity": 2 }
        ]
    });
    let order: serde_json::Value = broker.call("orders.create", order_params).await?;
    println!(
        "🛒 Created order: {}",
        order.get("id").unwrap_or(&Value::Null)
    );

    // Stop everything gracefully
    broker.stop().await?;
    println!("👋 E-commerce microservices example completed!");

    Ok(())
}
