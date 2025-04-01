# App Object (App)

## Overview

The `App` struct is the core of Ripress, providing a simple interface for creating HTTP servers and handling requests. It follows an Express-like pattern for route handling.

## Creating an App Object

Creates a new App instance:

```rust
use ripress::app::App;

let mut app = App::new();
```

## Route Handling Methods

### Basic Route Handler Pattern

All route handlers follow this pattern:

```rust
async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello, World!")
}
```

### Adding Routes That Match All HTTP Methods

Use `.all()` to handle any HTTP method:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.all("/hello", handler);

    app.listen(3000, || {}).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello from any method!")
}
```

## Serving Static Files

The `static_files` method provides a simple way to serve static assets (such as HTML, CSS, JavaScript, images, etc.) from a local directory. It maps a URL path prefix to a directory on your file system.

### Example

```rust
use ripress::app::App;

let mut app = App::new();

// Serve files from the "./public" directory when requests come to "/public"
app.static_files("/public", "./public");

app.get("/", |req, res| async { res.ok().text("Hello, World!") });

app.listen(3000, || {
    println!("Listening on port 3000");
})
.await;
```

### Usage Details

- **URL Path Prefix:**  
  The first argument is the URL path prefix (e.g., `/public`). Requests starting with this prefix will be treated as requests for static files.

- **Directory Path:**  
  The second argument is the local file system directory that contains your static assets (e.g., `"./public"`). Ensure that the path is correct relative to the project root.

This integration allows your application to serve both dynamic routes and static content easily.

### HTTP Method-Specific Routes

#### GET Requests

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

async fn get_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("GET request received")
}

let mut app = App::new();
app.get("/hello", get_handler);
```

#### POST Requests

```rust
async fn post_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("POST request received")
}

let mut app = App::new();
app.post("/submit", post_handler);
```

#### POST Requests

```rust
async fn post_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("HEAD request received")
}

let mut app = App::new();
app.head("/submit", post_handler);
```

#### PATCH Requests

```rust
async fn patch_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("POST request received")
}

let mut app = App::new();
app.patch("/submit", post_handler);
```

#### PUT Requests

```rust
async fn put_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("PUT request received")
}

let mut app = App::new();
app.put("/update", put_handler);
```

#### DELETE Requests

```rust
async fn delete_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("DELETE request received")
}

let mut app = App::new();
app.delete("/remove", delete_handler);
```

## Middlewares

Middleware provides a powerful way to process HTTP requests and responses in a modular, reusable manner.

### Adding Middleware

Use the `.use_middleware()` method to add middleware to your application:

```rust
let mut app = App::new();

app.use_middleware("/api/", |req, res, next| {
    println!("here");
    Box::pin(async move { next.run(req, res).await })
});
```

### Order Matters

Middleware is executed in the order it's added.
And they are applied to all routes.

The middleware will be applied to /api/\* in this case

## Dynamic Route Parameters

Routes can include dynamic parameters using `{paramName}` syntax:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};
use serde_json::json;

async fn user_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.get_params("id").unwrap_or("unknown".to_string());
    res.ok().json(json!({
        "userId": user_id,
        "message": "User details retrieved"
    }))
}

let mut app = App::new();
app.get("/user/{id}", user_handler);
```

## Starting the Server

Use the `.listen()` method to start the server:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Add your routes here
    app.get("/", home_handler);

    // Start the server
    println!("Server starting...");
    app.listen(3000, || {}).await;
}

async fn home_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Welcome to Ripress!")
}
```

All route handlers must be async functions that take `HttpRequest` and `HttpResponse` parameters and return `HttpResponse`. The server will automatically parse URL parameters, query strings, and request bodies based on the content type.
