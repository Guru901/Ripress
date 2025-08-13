# Basic Routing example

```rust
use ripress::{app::App, types::RouterFns};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Home route
    app.get("/", |_, res| async {
        res.json(json!({
            "message": "Welcome to the Ripress API example!"
        }))
    });

    // Basic user routes
    app.get("/users/:id", |req, res| async move {
        let user_id = req.params.get("id").unwrap_or("unknown");
        res.json(json!({
            "user_id": user_id,
            "message": format!("User details for ID: {}", user_id)
        }))
    });

    app.post("/users", |_, res| async {
        res.status(201).json(json!({
            "message": "User created successfully",
            "id": "new-user-123"
        }))
    });

    app.put("/users/:id", |req, res| async move {
        let user_id = req.params.get("id").unwrap_or("unknown");
        res.json(json!({
            "message": format!("User {} updated successfully", user_id)
        }))
    });

    app.delete("/users/:id", |req, res| async move {
        let user_id = req.params.get("id").unwrap_or("unknown");
        res.status(204).json(json!({
            "message": format!("User {} deleted successfully", user_id)
        }))
    });

    // Posts for a user
    app.get("/users/:user_id/posts", |req, res| async move {
        let user_id = req.params.get("user_id").unwrap_or("unknown");
        res.json(json!({
            "user_id": user_id,
            "posts": [
                {"id": 1, "title": "First Post"},
                {"id": 2, "title": "Second Post"}
            ]
        }))
    });

    app.post("/users/:user_id/posts", |req, res| async move {
        let user_id = req.params.get("user_id").unwrap_or("unknown");
        res.status(201).json(json!({
            "message": format!("Post created for user {}", user_id),
            "post_id": "new-post-456"
        }))
    });

    // Query parameter example
    app.get("/search", |req, res| async move {
        let query = req.query.get("q").unwrap_or("");
        let limit = req.query.get("limit").unwrap_or("10");
        let page = req.query.get("page").unwrap_or("1");
        res.json(json!({
            "query": query,
            "limit": limit.parse::<i32>().unwrap_or(10),
            "page": page.parse::<i32>().unwrap_or(1),
            "results": []
        }))
    });

    // Wildcard route example
    app.get("/files/*path", |req, res| async move {
        let file_path = req.params.get("path").unwrap_or("");
        res.json(json!({
            "message": format!("Serving file: {}", file_path),
            "path": file_path
        }))
    });

    // CORS preflight
    app.options("/*path", |_, res| async {
        res.set_header("Access-Control-Allow-Origin", "*")
            .set_header(
                "Access-Control-Allow-Methods",
                "GET, POST, PUT, DELETE, OPTIONS",
            )
            .set_header(
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization",
            )
            .status(200)
            .text("")
    });

    // Error route
    app.get("/error", |_, res| async {
        res.status(500).json(json!({
            "error": "Internal Server Error",
            "message": "Something went wrong"
        }))
    });

    // Protected route example
    app.get("/protected", |req, res| async move {
        let auth_header = req.headers.get("Authorization");
        match auth_header {
            Some(token) if token.starts_with("Bearer ") => res.json(json!({
                "message": "Access granted to protected resource"
            })),
            _ => res.status(401).json(json!({
                "error": "Unauthorized",
                "message": "Valid bearer token required"
            })),
        }
    });

    // 404 handler
    app.get("*", |req, res| async move {
        res.status(404).json(json!({
            "error": "Not Found",
            "message": format!("Route {} not found", req.path)
        }))
    });

    println!("🚀 Ripress server running on http://localhost:3000");
    println!("Try these routes:");
    println!("  GET    /");
    println!("  GET    /users/:id");
    println!("  POST   /users");
    println!("  PUT    /users/:id");
    println!("  DELETE /users/:id");
    println!("  GET    /users/:user_id/posts");
    println!("  POST   /users/:user_id/posts");
    println!("  GET    /search?q=term");
    println!("  GET    /files/*path");
    println!("  GET    /protected");

    app.listen(3000, || {
        println!("✨ Server is ready!");
    })
    .await;
}
```
