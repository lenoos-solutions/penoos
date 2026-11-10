//! API Gateway example demonstrating HTTP-like service routing
//! 
//! This example shows how to create a gateway service that routes
//! requests to different backend services based on URL patterns.

use penoos::{ServiceBroker, Service, Context};
use serde_json::json;

#[tokio::main]
async fn main() -> penoos::Result<()> {
    env_logger::init();
    
    println!("🌐 Starting API Gateway example...");

    let mut broker = ServiceBroker::new()?;

    // Create users service
    let users_service = Service::new("users")
        .action("list", |_ctx: Context| {
            Ok(json!({
                "users": [
                    {"id": 1, "name": "John Doe", "email": "john@example.com"},
                    {"id": 2, "name": "Jane Smith", "email": "jane@example.com"}
                ]
            }))
        })
        .action("get", |ctx: Context| {
            let id = ctx.params().get("id").unwrap().as_u64().unwrap();
            Ok(json!({
                "user": {
                    "id": id,
                    "name": format!("User {}", id),
                    "email": format!("user{}@example.com", id)
                }
            }))
        })
        .action("create", |ctx: Context| {
            let name = ctx.params().get("name").unwrap().as_str().unwrap();
            let email = ctx.params().get("email").unwrap().as_str().unwrap();
            Ok(json!({
                "user": {
                    "id": 123,
                    "name": name,
                    "email": email,
                    "created": true
                }
            }))
        });

    // Create posts service
    let posts_service = Service::new("posts")
        .action("list", |_ctx: Context| {
            Ok(json!({
                "posts": [
                    {"id": 1, "title": "First Post", "author": "John Doe"},
                    {"id": 2, "title": "Second Post", "author": "Jane Smith"}
                ]
            }))
        })
        .action("get", |ctx: Context| {
            let id = ctx.params().get("id").unwrap().as_u64().unwrap();
            Ok(json!({
                "post": {
                    "id": id,
                    "title": format!("Post {}", id),
                    "content": format!("This is the content of post {}", id),
                    "author": "Unknown Author"
                }
            }))
        });

    // Create API Gateway service
    let gateway_service = Service::new("gateway")
        .action("route", |ctx: Context| {
            let path = ctx.params().get("path").unwrap().as_str().unwrap();
            let method = ctx.params().get("method").unwrap().as_str().unwrap();
            let query = ctx.params().get("query").cloned().unwrap_or(json!({}));
            let body = ctx.params().get("body").cloned().unwrap_or(json!({}));
            
            // Route based on path
            let (service, action, params) = match (method, path) {
                ("GET", "/users") => ("users", "list", json!({})),
                ("GET", path) if path.starts_with("/users/") => {
                    let id = path.strip_prefix("/users/").unwrap().parse::<u64>().unwrap_or(0);
                    ("users", "get", json!({"id": id}))
                },
                ("POST", "/users") => ("users", "create", body),
                ("GET", "/posts") => ("posts", "list", json!({})),
                ("GET", path) if path.starts_with("/posts/") => {
                    let id = path.strip_prefix("/posts/").unwrap().parse::<u64>().unwrap_or(0);
                    ("posts", "get", json!({"id": id}))
                },
                _ => return Ok(json!({
                    "error": "Not Found",
                    "code": 404,
                    "message": format!("Route {} {} not found", method, path)
                }))
            };

            // Merge query parameters into params
            let mut final_params = params.as_object().unwrap().clone();
            if let Some(query_obj) = query.as_object() {
                for (key, value) in query_obj {
                    final_params.insert(key.clone(), value.clone());
                }
            }

            Ok(json!({
                "route": {
                    "service": service,
                    "action": action,
                    "params": final_params
                }
            }))
        });

    // Register services
    broker.create_service(users_service).await?;
    broker.create_service(posts_service).await?;
    broker.create_service(gateway_service).await?;

    broker.start().await?;
    println!("✅ API Gateway started successfully!");

    // Simulate API requests
    println!("\n🌐 Simulating API requests...\n");

    // GET /users
    let route_result = broker.call("gateway.route", json!({
        "method": "GET",
        "path": "/users",
        "query": {},
        "body": {}
    })).await?;
    
    let route_info = route_result.get("route").unwrap();
    let users_result = broker.call(
        &format!("{}.{}", route_info.get("service").unwrap().as_str().unwrap(), route_info.get("action").unwrap().as_str().unwrap()),
        route_info.get("params").unwrap().clone()
    ).await?;
    
    println!("📡 GET /users");
    println!("Route: {}", serde_json::to_string_pretty(&route_result)?);
    println!("Response: {}", serde_json::to_string_pretty(&users_result)?);

    // GET /users/1
    let route_result = broker.call("gateway.route", json!({
        "method": "GET",
        "path": "/users/1",
        "query": {},
        "body": {}
    })).await?;
    
    let route_info = route_result.get("route").unwrap();
    let user_result = broker.call(
        &format!("{}.{}", route_info.get("service").unwrap().as_str().unwrap(), route_info.get("action").unwrap().as_str().unwrap()),
        route_info.get("params").unwrap().clone()
    ).await?;
    
    println!("\n📡 GET /users/1");
    println!("Route: {}", serde_json::to_string_pretty(&route_result)?);
    println!("Response: {}", serde_json::to_string_pretty(&user_result)?);

    // POST /users
    let route_result = broker.call("gateway.route", json!({
        "method": "POST",
        "path": "/users",
        "query": {},
        "body": {
            "name": "Alice Johnson",
            "email": "alice@example.com"
        }
    })).await?;
    
    let route_info = route_result.get("route").unwrap();
    let create_result = broker.call(
        &format!("{}.{}", route_info.get("service").unwrap().as_str().unwrap(), route_info.get("action").unwrap().as_str().unwrap()),
        route_info.get("params").unwrap().clone()
    ).await?;
    
    println!("\n📡 POST /users");
    println!("Route: {}", serde_json::to_string_pretty(&route_result)?);
    println!("Response: {}", serde_json::to_string_pretty(&create_result)?);

    // Test 404
    let not_found = broker.call("gateway.route", json!({
        "method": "GET",
        "path": "/invalid",
        "query": {},
        "body": {}
    })).await?;
    
    println!("\n📡 GET /invalid (404 test)");
    println!("Response: {}", serde_json::to_string_pretty(&not_found)?);

    broker.stop().await?;
    println!("\n🛑 API Gateway stopped. Goodbye!");

    Ok(())
}
