# App

## Overview

The `App` struct is the core component of the Ripress web framework, providing an Express.js-inspired interface for building HTTP servers in Rust. It handles routing, middleware, static file serving, and request/response processing in an asynchronous environment.

The App follows a builder pattern where you configure routes, middleware, and static files before starting the server with the `listen()` method.

## Table of Contents

1. [Creating an App Instance](#creating-an-app-instance)
2. [Route Handling](#route-handling)
3. [HTTP Method-Specific Routes](#http-method-specific-routes)
4. [Dynamic Route Parameters](#dynamic-route-parameters)
5. [Query Parameters and Request Body](#query-parameters-and-request-body)
6. [Middleware System](#middleware-system)
7. [Static File Serving](#static-file-serving)
8. [WebSocket Support](#websocket-support)
9. [Error Handling](#error-handling)
10. [Starting the Server](#starting-the-server)
11. [Complete Examples](#complete-examples)

## Creating an App Instance

### Basic Usage

```rust
use ripress::app::App;

let mut app = App::new();
```

The `App::new()` constructor creates a fresh App instance with:

- Empty route table
- No middleware configured
- No static file mappings
- Default error handling

### App Structure

The App internally maintains:

- `routes`: HashMap storing route paths and their associated HTTP method handlers
- `middlewares`: Vector of middleware functions with their mount paths
- `static_files`: HashMap for static file serving configuration

## Route Handling

### Handler Function Signature

All route handlers in Ripress must follow this exact signature:

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // Handler logic here
    res.ok().text("Response content")
}
```

**Important Requirements:**

- Must be `async` functions
- Must accept exactly two parameters: `HttpRequest` and `HttpResponse`
- Must return `HttpResponse`
- Must be `Send + 'static` for thread safety

### Handler Registration

Routes are registered using HTTP method-specific functions:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || {}).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Response content")
}
```

### Closure Handlers

You can also use closures as handlers:

```rust
use ripress::{
    app::App
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/inline", |req, res| async move {
        let message = format!("Request path: {}", req.path);
        res.ok().text(message)
    });

    app.listen(3000, || {}).await;
}
```

## HTTP Method-Specific Routes

### GET Requests

Used for retrieving data. GET requests should be idempotent and safe.

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};
use serde_json::json;

async fn get_users(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // Fetch users from database
    let users = vec!["Alice", "Bob", "Charlie"];

    res.ok().json(json!({
        "users": users,
        "count": users.len()
    }))
}

#[tokio::main]
async fn main() {
    let mut app = App::new();
    app.get("/users", get_users);

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
    })
    .await;
}
```

### POST Requests

Used for creating new resources or submitting data.

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};
use serde_json::{Value, json};

#[tokio::main]
async fn main() {
    let mut app = App::new();
    app.post("/users", create_user);

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
    })
    .await;
}

async fn create_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // Parse JSON body
    if let Ok(body) = req.json::<Value>() {
        let name = body
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        // Simulate user creation
        let new_user = json!({
            "id": 123,
            "name": name,
            "created_at": "2024-01-01T00:00:00Z"
        });

        res.created().json(new_user)
    } else {
        res.bad_request().json(json!({
            "error": "Invalid JSON body"
        }))
    }
}
```

### PUT Requests

Used for updating existing resources (full replacement).

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};
use serde_json::{Value, json};

#[tokio::main]
async fn main() {
    let mut app = App::new();
    app.put("/users/{id}", update_user);

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
    })
    .await;
}

async fn update_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.params.get("id").unwrap_or("0");

    if let Ok(body) = req.json::<Value>() {
        // Update user logic here
        res.ok().json(json!({
            "id": user_id,
            "message": "User updated successfully"
        }))
    } else {
        res.bad_request().json(json!({
            "error": "Invalid request body"
        }))
    }
}
```

### PATCH Requests

Used for partial updates to existing resources.

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    app.patch("/users/{id}", patch_user);

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
    })
    .await;
}

async fn patch_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.params.get("id").unwrap_or("0");

    res.ok().json(json!({
        "id": user_id,
        "message": "User partially updated"
    }))
}
```

### DELETE Requests

Used for removing resources.

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    app.delete("/users/{id}", delete_user);

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
    })
    .await;
}

async fn delete_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.params.get("id").unwrap_or("0");

    // Delete user logic here
    res.ok().json(json!({
        "message": format!("User {} deleted successfully", user_id)
    }))
}
```

### OPTIONS Requests

Used for CORS preflight requests and discovering allowed methods.

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};

#[tokio::main]
async fn main() {
    let mut app = App::new();
    app.options("/api/*", options_handler);

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
    })
    .await;
}

async fn options_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
        .set_header("Allow", "GET, POST, PUT, DELETE, OPTIONS")
        .set_header(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        )
        .set_header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )
        .text("OK")
}
```

## Dynamic Route Parameters

### Basic Parameters

Use `{parameter_name}` syntax to define dynamic route segments:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    app.get("/users/{id}/profile", get_user_profile);

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
    })
    .await;
}

async fn get_user_profile(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.params.get("id").unwrap_or("unknown");

    res.ok().json(json!({
        "user_id": user_id,
        "profile": {
            "name": format!("User {}", user_id),
            "email": format!("user{}@example.com", user_id)
        }
    }))
}
```

### Multiple Parameters

Routes can have multiple dynamic segments:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    app.get("/users/{user_id}/posts/{post_id}", get_user_post);

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
    })
    .await;
}

async fn get_user_post(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.params.get("user_id").unwrap_or("0");
    let post_id = req.params.get("post_id").unwrap_or("0");

    res.ok().json(json!({
        "user_id": user_id,
        "post_id": post_id,
        "post": {
            "title": format!("Post {} by User {}", post_id, user_id),
            "content": "Sample content..."
        }
    }))
}
```

### Parameter Validation

Always validate and sanitize route parameters:

```rust
use serde_json::json;

async fn get_user_safe(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.params.get("id") {
        Some(id) => {
            match id.parse::<u32>() {
                Ok(user_id) if user_id > 0 => {
                    res.ok().json(json!({
                        "user_id": user_id,
                        "message": "Valid user ID"
                    }))
                }
                _ => {
                    res.bad_request().json(json!({
                        "error": "Invalid user ID format"
                    }))
                }
            }
        }
        None => {
            res.bad_request().json(json!({
                "error": "User ID is required"
            }))
        }
    }
}
```

## Query Parameters and Request Body

### Accessing Query Parameters

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Usage: GET /search?q=john&limit=5
    app.get("/search", search_users);

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
    })
    .await;
}

async fn search_users(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let query = req.query.get("q").unwrap_or("");
    let limit = req
        .query
        .get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(10);

    // Perform search logic
    res.ok().json(json!({
        "query": query,
        "limit": limit,
        "results": []
    }))
}
```

### Processing Request Bodies

#### JSON Bodies

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize)]
struct CreateUserRequest {
    name: String,
    email: String,
    age: Option<u32>,
}

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.post("/create-user", create_user_typed);

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
    })
    .await;
}

async fn create_user_typed(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.json::<CreateUserRequest>() {
        Ok(user_data) => {
            // Process the structured data
            res.created().json(json!({
                "message": "User created",
                "user": user_data
            }))
        }
        Err(e) => res.bad_request().json(json!({
            "error": format!("Invalid JSON: {}", e)
        })),
    }
}
```

#### Form Data

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.post("/handle-form", handle_form);

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
    })
    .await;
}

async fn handle_form(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.form_data() {
        Ok(form_data) => {
            if let Some(name) = form_data.get("name") {
                res.ok().text(format!("Hello, {}!", name))
            } else {
                res.bad_request().text("Name field is required")
            }
        }
        Err(e) => res.bad_request().json(json!({
            "error": format!("Form parsing error: {}", e)
        })),
    }
}
```

## Middleware System

### Understanding Middleware

Ripress supports two types of middleware:

**Pre-middleware** - Executes before route handlers, allowing you to:

- Authenticate requests
- Add CORS headers
- Validate input
- Transform requests
- Handle errors

**Post-middleware** - Executes after route handlers, allowing you to:

- Log requests/responses
- Transform responses
- Add response headers
- Clean up resources
- Monitor performance

### Middleware Function Signature

```rust
|req: HttpRequest, res: HttpResponse| -> impl Future<Output = (HttpRequest, Option<HttpResponse>)>
```

**Return Values:**

- `(request, None)`: Continue to next middleware/handler
- `(request, Some(response))`: Short-circuit and return response immediately

### Pre-Middleware Example

```rust
app.use_pre_middleware("/api/", |req, res| async {
    println!("Pre-processing API request: {} {}", req.method, req.path);

    // Continue processing
    (req, None)

    // Or block the request:
    // (req, Some(res.unauthorized().text("Access denied")))
});
```

### Post-Middleware Example

```rust
app.use_post_middleware("/api/", |req, res| async {
    println!("Post-processing API response: {} {}", req.method, req.path);

    // Continue processing
    (req, None)

    // Or modify the response:
    // (req, Some(res.set_header("X-Processed-By", "Ripress")))
});
```

### Authentication Middleware

```rust
use serde_json::json;

app.use_pre_middleware("/protected/", |req, res| async {
    match req.headers.get("Authorization") {
        Some(auth_header) if auth_header.starts_with("Bearer ") => {
            let token = &auth_header[7..]; // Remove "Bearer " prefix

            if validate_token(token) {
                (req, None) // Valid token, continue
            } else {
                (
                    req,
                    Some(res.unauthorized().json(json!({
                        "error": "Invalid token"
                    }))),
                )
            }
        }
        _ => (
            req,
            Some(res.unauthorized().json(json!({
                "error": "Authentication required"
            }))),
        ),
    }
});

fn validate_token(token: &str) -> bool {
    // Implement your token validation logic
    token == "valid-secret-token"
}
```

### CORS Middleware

```rust
app.use_cors(None);
```

### Request Logging Middleware

```rust
app.use_logger(None);
```

### Middleware Execution Order

Middleware executes in a specific order based on type and registration order:

**Execution Flow:**

1. **Pre-middleware** (in registration order)
2. **Route handler**
3. **Post-middleware** (in registration order)

```rust
// Pre-middleware runs first (in registration order)
app.use_pre_middleware("/", |req, res| async {
    println!("First pre-middleware");
    (req, None)
});

app.use_pre_middleware("/api/", |req, res| async {
    println!("API pre-middleware");
    (req, None)
});

// Route handler executes here
app.get("/api/users", |req, res| async {
    res.ok().text("Users data")
});

// Post-middleware runs after route handler (in registration order)
app.use_post_middleware("/api/", |req, res| async {
    println!("API post-middleware");
    (req, None)
});

app.use_post_middleware("/", |req, res| async {
    println!("Last post-middleware");
    (req, None)
});
```

**For a request to `/api/users`, the execution order would be:**

1. "First pre-middleware" (matches "/")
2. "API pre-middleware" (matches "/api/")
3. Route handler (returns "Users data")
4. "API post-middleware" (matches "/api/")
5. "Last post-middleware" (matches "/")

### File Upload Middleware

The built-in file upload middleware processes binary file uploads and saves them to a configurable directory. It automatically detects file types, generates unique filenames, and supports both raw binary uploads and `multipart/form-data` from browsers.

#### Basic Usage

```rust
use ripress::middlewares::file_upload::{file_upload, FileUploadConfiguration};

// Use default upload directory ("uploads")
app.use_pre_middleware("/upload", file_upload(None));

// Or specify a custom upload directory
app.use_pre_middleware("/upload", file_upload(Some(FileUploadConfiguration {
    upload_dir: "user_files",
    ..Default::default();
})));
```

#### How It Works

The file upload middleware:

1. **Processes binary requests** - Works with `RequestBodyType::BINARY` content
2. **Supports multipart forms** - Extracts ALL file parts and text fields from `multipart/form-data`
3. **Detects file extensions** - Uses the `infer` crate for automatic type detection
4. **Generates unique filenames** - Creates UUID-based names to prevent conflicts
5. **Saves files** - Writes uploaded content to the specified directory
6. **Sets comprehensive request data** - Adds file information and form field mappings

#### Route Handler Example

```rust
app.post("/upload", |req, res| async move {
    // Check if files were uploaded successfully
    if let Some(count_str) = req.get_data("uploaded_file_count") {
        let count: usize = count_str.parse().unwrap_or(0);
        if count > 0 {
            if let Some(files_json) = req.get_data("uploaded_files") {
                res.ok().json(json!({
                    "success": true,
                    "count": count,
                    "files": serde_json::from_str::<serde_json::Value>(&files_json).unwrap(),
                    "message": format!("Successfully uploaded {} files", count)
                }))
            } else {
                res.ok().json(json!({
                    "success": true,
                    "count": count,
                    "message": format!("Successfully uploaded {} files", count)
                }))
            }
        } else {
            res.ok().json(json!({
                "success": false,
                "message": "No files were uploaded"
            }))
        }
    } else {
        res.ok().json(json!({
            "success": false,
            "message": "No files were uploaded"
        }))
    }
});
```

## Static File Serving

### Basic Static File Configuration

```rust
let mut app = App::new();

// Serve files from "./public" directory at "/static" URL path
app.static_files("/static", "./public");

// Now files are accessible:
// ./public/index.html -> http://localhost:3000/static/index.html
// ./public/css/style.css -> http://localhost:3000/static/css/style.css
```

### Directory Structure Example

```
project/
├── src/
│   └── main.rs
└── public/
    ├── index.html
    ├── css/
    │   └── style.css
    ├── js/
    │   └── app.js
    └── images/
        └── logo.png
```

### Advanced Static File Serving

The static file server automatically:

- Sets appropriate `Content-Type` headers based on file extensions
- Handles conditional requests with `If-None-Match` headers
- Returns `304 Not Modified` for cached files
- Sets `Cache-Control: public, max-age=86400` (24 hours)
- Adds custom `X-Served-By: hyper-staticfile` header

### Multiple Static Directories

```rust
// Currently, Ripress supports one static file mapping per app
// For multiple directories, you can:

// Option 1: Use different path prefixes with symbolic links or file organization
app.static_files("/assets", "./static");

// Option 2: Create multiple App instances (not recommended for simple cases)
```

### Static File Security

The static file server:

- Prevents directory traversal attacks (`../` attempts)
- Only serves files within the specified directory
- Respects file system permissions
- Returns appropriate HTTP status codes (404, 403, etc.)

## WebSocket Support

Ripress provides built-in WebSocket support through the `wynd` crate when the "with-wynd" feature is enabled. This allows you to create real-time applications alongside your HTTP routes.

### Prerequisites

Ensure you have the WebSocket dependencies in your `Cargo.toml`:

```toml
[dependencies]
ripress = { version = "1", features = ["with-wynd"] }  # Enable WebSocket support
wynd = "0.4"  # WebSocket library
```

### Basic WebSocket Setup

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let mut wynd = Wynd::new();

    // HTTP routes
    app.get("/", |_, res| async move {
        res.ok().text("Hello, World!")
    });

    // WebSocket connection handler
    wynd.on_connection(|conn| async move {
        conn.on_text(|event, _| async move {
            println!("Received message: {}", event.data);
        });
    });

    // Mount WebSocket at /ws path
    app.use_wynd("/ws", wynd.handler());

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
        println!("WebSocket available at ws://localhost:3000/ws");
    })
    .await;
}
```

### WebSocket Event Handlers

The `wynd` crate provides several event handlers for different WebSocket events:

#### Text Message Handler

```rust
wynd.on_connection(|conn| async move {
    conn.on_text(|event, _| async move {
        println!("Text message: {}", event.data);

        // Echo the message back
        if let Err(e) = event.handle.send_text(&format!("Echo: {}", event.data)).await {
            eprintln!("Failed to send echo: {}", e);
        }
    });
});
```

#### Binary Message Handler

```rust
wynd.on_connection(|conn| async move {
    conn.on_binary(|event, _| async move {
        println!("Binary message received, size: {} bytes", event.data.len());

        // Echo binary data back
        if let Err(e) = event.handle.send_binary(&event.data).await {
            eprintln!("Failed to send binary echo: {}", e);
        }
    });
});
```

#### Connection Close Handler

```rust
wynd.on_connection(|conn| async move {
    conn.on_close(|event, _| async move {
        println!("Connection closed: {:?}", event.code);
    });
});
```

#### Error Handler

```rust
wynd.on_connection(|conn| async move {
    conn.on_error(|event, _| async move {
        eprintln!("WebSocket error: {:?}", event.error);
    });
});
```

### Advanced WebSocket Example

Here's a more comprehensive example with multiple event handlers and state management:

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::Wynd;
use std::sync::Arc;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let mut wynd = Wynd::new();

    // Create a broadcast channel for chat messages
    let (tx, _rx) = broadcast::channel::<String>(100);
    let tx = Arc::new(tx);

    // HTTP route for the chat page
    app.get("/", |_, res| async move {
        res.ok().html(r#"
            <!DOCTYPE html>
            <html>
            <head><title>WebSocket Chat</title></head>
            <body>
                <h1>WebSocket Chat</h1>
                <div id="messages"></div>
                <input type="text" id="message" placeholder="Type a message...">
                <button onclick="sendMessage()">Send</button>
                <script>
                    const ws = new WebSocket('ws://localhost:3000/ws');
                    const messages = document.getElementById('messages');
                    const input = document.getElementById('message');

                    ws.onmessage = function(event) {
                        const div = document.createElement('div');
                        div.textContent = event.data;
                        messages.appendChild(div);
                    };

                    function sendMessage() {
                        if (input.value.trim()) {
                            ws.send(input.value);
                            input.value = '';
                        }
                    }

                    input.addEventListener('keypress', function(e) {
                        if (e.key === 'Enter') sendMessage();
                    });
                </script>
            </body>
            </html>
        "#)
    });

    // WebSocket connection handler with chat functionality
    wynd.on_connection(|conn| async move {
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // Handle incoming text messages
        conn.on_text(|event, _| async move {
            let message = format!("User: {}", event.data);
            println!("{}", message);

            // Broadcast message to all connected clients
            if let Err(e) = tx.send(message.clone()) {
                eprintln!("Failed to broadcast message: {}", e);
            }

            // Send confirmation back to sender
            if let Err(e) = event.handle.send_text("Message sent!").await {
                eprintln!("Failed to send confirmation: {}", e);
            }
        });

        // Handle connection close
        conn.on_close(|event, _| async move {
            println!("Client disconnected: {:?}", event.code);
        });

        // Handle errors
        conn.on_error(|event, _| async move {
            eprintln!("WebSocket error: {:?}", event.error);
        });
    });

    // Mount WebSocket at /ws path
    app.use_wynd("/ws", wynd.handler());

    app.listen(3000, || {
        println!("🚀 Chat server running on http://localhost:3000");
        println!("🔌 WebSocket available at ws://localhost:3000/ws");
    })
    .await;
}
```

### WebSocket Best Practices

1. **Error Handling**: Always handle WebSocket errors gracefully
2. **Connection Management**: Track active connections for cleanup
3. **Message Validation**: Validate incoming messages before processing
4. **Rate Limiting**: Implement rate limiting for WebSocket messages
5. **Security**: Validate WebSocket upgrade requests and implement authentication if needed
6. **Graceful Shutdown**: Handle connection closures properly

### Testing WebSocket Connections

You can test WebSocket connections using various tools:

```bash
# Using wscat (install with: npm install -g wscat)
wscat -c ws://localhost:3000/ws

# Using curl (for WebSocket upgrade testing)
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" \
     -H "Sec-WebSocket-Version: 13" -H "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
     http://localhost:3000/ws
```

### WebSocket with Middleware

WebSocket connections can coexist with HTTP middleware. The WebSocket middleware is applied before other middleware, ensuring proper WebSocket upgrade handling:

```rust
let mut app = App::new();
let mut wynd = Wynd::new();

// Add HTTP middleware
app.use_pre_middleware("/", |req, res| async {
    println!("HTTP request: {} {}", req.method, req.path);
    (req, None)
});

// WebSocket setup
wynd.on_connection(|conn| async move {
    conn.on_text(|event, _| async move {
        println!("WebSocket message: {}", event.data);
    });
});

app.use_wynd("/ws", wynd.handler());
```

## Error Handling

### Built-in Error Handling

Ripress provides automatic error handling for:

- Invalid JSON in request bodies
- Route parameter parsing errors
- File serving errors
- Internal server errors

### Custom Error Responses

```rust
use ripress::context::{HttpRequest, HttpResponse};
use serde_json::json;

async fn error_prone_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match risky_operation() {
        Ok(data) => res.ok().json(data),
        Err(e) => {
            eprintln!("Error in handler: {}", e);
            res.internal_server_error().json(json!({
                "error": "Internal server error",
                "message": "Something went wrong"
            }))
        }
    }
}

fn risky_operation() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Some operation that might fail
    Ok(json!({"status": "success"}))
}
```

### Validation Errors

```rust
use ripress::context::{HttpRequest, HttpResponse};
use serde_json::json;

async fn validate_input(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let mut errors = Vec::new();

    // Validate required fields
    if !req.query.contains("email") {
        errors.push("Email is required");
    }

    if let Some(age) = req.query.get("age") {
        if age.parse::<u32>().is_err() {
            errors.push("Age must be a valid number");
        }
    }

    if !errors.is_empty() {
        return res.bad_request().json(json!({
            "error": "Validation failed",
            "details": errors
        }));
    }

    res.ok().json(json!({"status": "validated"}))
}
```

## Starting the Server

### Basic Server Startup

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Configure your app...
    app.get("/", home_handler);

    // Start the server
    app.listen(3000, || {
        println!("🚀 Server running on http://localhost:3000");
    })
    .await;
}

async fn home_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Welcome to Ripress!")
}
```

### Server Configuration

The `listen` method:

- Binds to `127.0.0.1` (localhost) by default
- Accepts a port number (1-65535)
- Takes a closure that executes once when the server starts
- Returns a `Future` that runs indefinitely
- Handles graceful shutdown on process termination

### Production Considerations

```rust
use ripress::app::App;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Add production middleware
    app.use_shield(None);

    // Configure routes...

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    app.listen(port, || {
        println!("Production server running on port {}", port);
    })
    .await;
}
```

## Complete Examples

### RESTful API Example

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

type UserStore = Arc<Mutex<HashMap<u32, User>>>;

async fn get_users(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // In a real app, you'd access the store from app state
    res.ok().json(json!({
        "users": [],
        "total": 0
    }))
}

async fn create_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.json::<User>() {
        Ok(user_data) => {
            res.created().json(json!({
                "message": "User created",
                "user": user_data
            }))
        }
        Err(e) => {
            res.bad_request().json(json!({
                "error": format!("Invalid user data: {}", e)
            }))
        }
    }
}

async fn get_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.params.get("id").unwrap_or("0");

    res.ok().json(json!({
        "user": {
            "id": user_id,
            "name": "John Doe",
            "email": "john@example.com"
        }
    }))
}

async fn update_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.params.get("id").unwrap_or("0");

    match req.json::<User>() {
        Ok(_user_data) => {
            res.ok().json(json!({
                "message": format!("User {} updated", user_id)
            }))
        }
        Err(e) => {
            res.bad_request().json(json!({
                "error": format!("Invalid user data: {}", e)
            }))
        }
    }
}

async fn delete_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.params.get("id").unwrap_or("0");

    res.ok().json(json!({
        "message": format!("User {} deleted", user_id)
    }))
}

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // API routes
    app.get("/api/users", get_users);
    app.post("/api/users", create_user);
    app.get("/api/users/{id}", get_user);
    app.put("/api/users/{id}", update_user);
    app.delete("/api/users/{id}", delete_user);

    // Serve static files
    app.static_files("/", "./public");

    app.listen(3000, || {
        println!("🚀 API server running on http://localhost:3000");
        println!("📖 API endpoints:");
        println!("   GET    /api/users");
        println!("   POST   /api/users");
        println!("   GET    /api/users/:id");
        println!("   PUT    /api/users/:id");
        println!("   DELETE /api/users/:id");
    }).await;
}
```

This comprehensive documentation covers all major aspects of the Ripress App object, providing detailed examples and explanations for building robust web applications.
