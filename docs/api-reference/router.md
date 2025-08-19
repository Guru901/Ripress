# Ripress Router API Reference

## Overview

The `Router` struct provides a simple way to group and manage routes under a common base path. It allows you to define routes for various HTTP methods (GET, POST, PUT, DELETE, and PATCH) and then register those routes with an `App` instance.

## Creating a Router Instance

To create a new router, use the `Router::new` method and specify the base path. For example:

```rust
use ripress::{app::App, router::Router};

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let router = Router::new("/api");

    router.register(&mut app);
}
```

## Defining Routes

The Router offers methods corresponding to each HTTP method. Each method takes a static path and a handler function. The handler must be an async function that accepts an `HttpRequest` and an `HttpResponse`, then returns an `HttpResponse`.

### Basic Route Definition

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    router::Router,
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let mut router = Router::new("/api");

    router.get("/hello", get_handler);
    router.post("/hello", post_handler);
    // The route will be /api/hello

    router.register(&mut app);
}

async fn get_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("GET request handled")
}

async fn post_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("POST request handled")
}
```

### All HTTP Methods

The router supports all common HTTP methods:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    router::Router,
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let mut router = Router::new("/api");

    // Define handlers for all HTTP methods
    router.get("/users", get_users);
    router.post("/users", create_user);
    router.put("/users/:id", update_user);
    router.delete("/users/:id", delete_user);
    router.patch("/users/:id", patch_user);

    router.register(&mut app);

    app.listen(3000, || println!("Server running on port 3000")).await
}

async fn get_users(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(serde_json::json!([
        {"id": 1, "name": "John Doe"},
        {"id": 2, "name": "Jane Smith"}
    ]))
}

async fn create_user(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(201).json(serde_json::json!({
        "message": "User created successfully"
    }))
}

async fn update_user(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(serde_json::json!({
        "message": "User updated successfully"
    }))
}

async fn delete_user(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(serde_json::json!({
        "message": "User deleted successfully"
    }))
}

async fn patch_user(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(serde_json::json!({
        "message": "User partially updated"
    }))
}
```

## Registering the Router with an App

After defining the routes using your router, register the router with an `App` instance. This will add all the router's routes to the application with their full path (combining the router's base path and the route's defined path):

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    router::Router,
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
        res.ok().text("Hello from Router!")
    }

    let mut router = Router::new("/api");

    // Define routes on the router
    router.get("/hello", handler);

    // Create an App instance and register the router
    let mut app = App::new();
    router.register(&mut app);

    app.listen(3000, || println!("Server running on port 3000")).await
}
```

## Multiple Routers

You can create and register multiple routers for better organization:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    router::Router,
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // User routes
    let mut user_router = Router::new("/api/users");
    user_router.get("/", list_users);
    user_router.post("/", create_user);
    user_router.get("/:id", get_user);

    // Product routes
    let mut product_router = Router::new("/api/products");
    product_router.get("/", list_products);
    product_router.post("/", create_product);
    product_router.get("/:id", get_product);

    // Register both routers
    user_router.register(&mut app);
    product_router.register(&mut app);

    app.listen(3000, || println!("Server running on port 3000")).await
}

// Handler implementations...
async fn list_users(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(serde_json::json!({"users": []}))
}

async fn create_user(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(201).json(serde_json::json!({"message": "User created"}))
}

async fn get_user(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(serde_json::json!({"user": {}}))
}

async fn list_products(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(serde_json::json!({"products": []}))
}

async fn create_product(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(201).json(serde_json::json!({"message": "Product created"}))
}

async fn get_product(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(serde_json::json!({"product": {}}))
}
```

## Nested Path Structure

Routers automatically combine the base path with route paths:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    router::Router,
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let mut router = Router::new("/api/v1");

    router.get("/health", health_check);        // Results in: /api/v1/health
    router.get("/users", get_users);            // Results in: /api/v1/users
    router.post("/users", create_user);         // Results in: /api/v1/users
    router.get("/users/:id", get_user_by_id);   // Results in: /api/v1/users/:id

    router.register(&mut app);

    app.listen(3000, || println!("Server running on port 3000")).await
}

async fn health_check(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(serde_json::json!({"status": "healthy"}))
}

async fn get_users(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(serde_json::json!({"users": []}))
}

async fn create_user(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(201).json(serde_json::json!({"message": "User created"}))
}

async fn get_user_by_id(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(serde_json::json!({"user": {}}))
}
```

## How It Works

- **Route Storage:**  
  The router stores routes internally using a `HashMap`, with each route path mapping to a set of HTTP methods and their associated handlers.

- **Method Handlers:**  
  The helper methods (`get`, `post`, `put`, `delete`, and `patch`) wrap the provided handler functions and associate them with the corresponding HTTP method.

- **Registration:**  
  Calling `register` on the router iterates through all the defined routes, prepends the router's base path to each route, and then adds them to the provided `App` instance.

This modular design simplifies the management of routes when building larger applications by keeping related routes together under a shared base path.

## Quick Reference

### Creating a Router

```rust
let mut router = Router::new("/api");
```

### HTTP Method Handlers

- `.get(path, handler)` - Handle GET requests
- `.post(path, handler)` - Handle POST requests
- `.put(path, handler)` - Handle PUT requests
- `.delete(path, handler)` - Handle DELETE requests
- `.patch(path, handler)` - Handle PATCH requests

### Registration

```rust
router.register(&mut app);
```

### Handler Signature

```rust
async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // Handler implementation
}
```

### Path Combination

- Router base path: `/api`
- Route path: `/users`
- Final path: `/api/users`
