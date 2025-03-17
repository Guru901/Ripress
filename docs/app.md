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

    app.listen("127.0.0.1:3000").await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello from any method!")
}
```

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
    app.listen("127.0.0.1:3000").await;
}

async fn home_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Welcome to Ripress!")
}
```

All route handlers must be async functions that take `HttpRequest` and `HttpResponse` parameters and return `HttpResponse`. The server will automatically parse URL parameters, query strings, and request bodies based on the content type.
