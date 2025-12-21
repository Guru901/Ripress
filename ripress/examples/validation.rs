//! Validation Example
//!
//! This example demonstrates how to use JSON validation with the `validator` crate
//! in Ripress. It shows:
//! - Basic validation rules (email, length, range)
//! - Handling validation errors
//! - Custom error responses
//! - Complex nested validation

use ripress::{
    app::App,
    req::{body::json_data::JsonBodyValidated, HttpRequest},
    res::HttpResponse,
    types::RouterFns,
};
use ripress_derive::FromJson;
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

// User registration with validation
#[derive(Debug, Deserialize, Serialize, Validate, FromJson)]
struct RegisterUser {
    #[validate(length(
        min = 3,
        max = 20,
        message = "Username must be between 3 and 20 characters"
    ))]
    username: String,

    #[validate(email(message = "Invalid email address"))]
    email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    password: String,

    #[validate(range(min = 18, max = 120, message = "Age must be between 18 and 120"))]
    age: u8,
}

// Product with validation
#[derive(Debug, Deserialize, Serialize, Validate, FromJson)]
struct CreateProduct {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Product name is required and must be less than 100 characters"
    ))]
    name: String,

    #[validate(range(min = 0.01, message = "Price must be greater than 0"))]
    price: f64,

    #[validate(length(max = 500, message = "Description must be less than 500 characters"))]
    description: Option<String>,

    #[validate(range(min = 0, message = "Stock cannot be negative"))]
    stock: i32,
}

// Blog post with nested validation
#[derive(Debug, Deserialize, Serialize, Validate)]
struct Author {
    #[validate(length(min = 2, max = 50))]
    name: String,

    #[validate(email)]
    email: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, FromJson)]
struct BlogPost {
    #[validate(length(min = 5, max = 200))]
    title: String,

    #[validate(length(min = 10, max = 5000))]
    content: String,

    author: Author,

    #[validate(length(max = 10))]
    tags: Vec<String>,
}

// Order with multiple items
#[derive(Debug, Deserialize, Serialize, Validate, FromJson)]
struct OrderItem {
    #[validate(length(min = 1))]
    product_id: String,

    #[validate(range(min = 1, max = 1000))]
    quantity: u32,
}

#[derive(Debug, Deserialize, Serialize, Validate, FromJson)]
struct CreateOrder {
    #[validate(length(min = 1, message = "Order must contain at least one item"))]
    items: Vec<OrderItem>,

    #[validate(length(min = 5, max = 200))]
    shipping_address: String,

    #[validate(email)]
    customer_email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // User registration endpoint
    app.post(
        "/api/users/register",
        |body: JsonBodyValidated<RegisterUser>, res: HttpResponse| async move {
            // If we reach here, validation has passed
            println!("Registering user: {:?}", body.username);

            res.status(201).json(json!({
                "success": true,
                "message": "User registered successfully",
                "username": body.username,
                "email": body.email
            }))
        },
    );

    // Product creation endpoint
    app.post(
        "/api/products",
        |body: JsonBodyValidated<CreateProduct>, res: HttpResponse| async move {
            println!("Creating product: {:?}", body.name);

            res.status(201).json(json!({
                "success": true,
                "message": "Product created successfully",
                "product": {
                    "name": body.name,
                    "price": body.price,
                    "stock": body.stock
                }
            }))
        },
    );

    // Blog post creation endpoint with nested validation
    app.post(
        "/api/posts",
        |body: JsonBodyValidated<BlogPost>, res: HttpResponse| async move {
            println!("Creating blog post: {:?}", body.title);

            res.status(201).json(json!({
                "success": true,
                "message": "Blog post created successfully",
                "post": {
                    "title": body.title,
                    "author": body.author.name,
                    "tags": body.tags
                }
            }))
        },
    );

    // Order creation endpoint
    app.post(
        "/api/orders",
        |body: JsonBodyValidated<CreateOrder>, res: HttpResponse| async move {
            println!("Creating order with {} items", body.items.len());

            res.status(201).json(json!({
                "success": true,
                "message": "Order created successfully",
                "order": {
                    "items_count": body.items.len(),
                    "customer_email": body.customer_email
                }
            }))
        },
    );

    // Health check endpoint (no validation)
    app.get("/health", |_: HttpRequest, res| async move {
        res.json(json!({
            "status": "healthy",
            "validation_enabled": cfg!(feature = "validation")
        }))
    });

    println!("🚀 Validation example server starting on http://127.0.0.1:3000");
    println!("\nTry these curl commands:\n");
    println!("✅ Valid user registration:");
    println!(r#"curl -X POST http://127.0.0.1:3000/api/users/register \"#);
    println!(r#"  -H "Content-Type: application/json" \"#);
    println!(
        r#"  -d '{{"username":"john_doe","email":"john@example.com","password":"securepass123","age":25}}'"#
    );
    println!("\n❌ Invalid user (short username):");
    println!(r#"curl -X POST http://127.0.0.1:3000/api/users/register \"#);
    println!(r#"  -H "Content-Type: application/json" \"#);
    println!(
        r#"  -d '{{"username":"ab","email":"john@example.com","password":"securepass123","age":25}}'"#
    );
    println!("\n✅ Valid product:");
    println!(r#"curl -X POST http://127.0.0.1:3000/api/products \"#);
    println!(r#"  -H "Content-Type: application/json" \"#);
    println!(
        r#"  -d '{{"name":"Laptop","price":999.99,"description":"Gaming laptop","stock":50}}'"#
    );
    println!("\n❌ Invalid product (negative stock):");
    println!(r#"curl -X POST http://127.0.0.1:3000/api/products \"#);
    println!(r#"  -H "Content-Type: application/json" \"#);
    println!(r#"  -d '{{"name":"Mouse","price":29.99,"stock":-5}}'"#);

    app.listen(3000, || {}).await;
    Ok(())
}
