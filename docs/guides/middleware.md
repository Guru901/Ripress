# Middleware Guide

## Overview

Middleware in Ripress are functions that execute before route handlers, allowing you to modify requests, add functionality, or short-circuit processing. They follow the same pattern as Express.js middleware and are executed in the order they're added.

## Middleware Function Signature

All middleware functions must follow this signature:

```rust
|req: HttpRequest, res: HttpResponse| -> impl Future<Output = (HttpRequest, Option<HttpResponse>)>
```

**Return Values:**

- `(request, None)`: Continue to next middleware/handler
- `(request, Some(response))`: Short-circuit and return response immediately

## Built-in Middleware

Ripress comes with several built-in middleware functions that you can use out of the box.

### CORS Middleware

The CORS middleware adds Cross-Origin Resource Sharing headers to your responses.

```rust
use ripress::middlewares::cors::cors;

// Basic CORS with default settings
app.use_middleware("/", cors(None));

// Custom CORS configuration
use ripress::middlewares::cors::CorsConfig;

let config = CorsConfig {
    allowed_origins: vec!["http://localhost:3000".to_string()],
    allowed_methods: vec!["GET".to_string(), "POST".to_string()],
    allowed_headers: vec!["Content-Type".to_string()],
    ..Default::default()
};

app.use_middleware("/", cors(Some(config)));
```

### Logger Middleware

The logger middleware provides request logging functionality.

```rust
use ripress::middlewares::logger::logger;

// Basic logging
app.use_middleware("/", logger(None));

// Custom logging configuration
use ripress::middlewares::logger::LoggerConfig;

let config = LoggerConfig {
    log_requests: true,
    log_responses: true,
    log_headers: false,
    ..Default::default()
};

app.use_middleware("/", logger(Some(config)));
```

### File Upload Middleware

The file upload middleware processes binary file uploads and saves them to a configurable directory.

#### Basic Usage

```rust
use ripress::middlewares::file_upload::file_upload;

// Use default upload directory ("uploads")
app.use_middleware("/upload", file_upload(None));

// Specify a custom upload directory
app.use_middleware("/upload", file_upload(Some("custom_uploads")));
```

#### How It Works

The file upload middleware:

1. **Processes binary requests** - Only works with `RequestBodyType::BINARY`
2. **Detects file extensions** - Uses the `infer` crate for automatic type detection
3. **Generates unique filenames** - Creates UUID-based names to prevent conflicts
4. **Saves files** - Writes uploaded content to the specified directory
5. **Sets request data** - Adds `uploaded_file` and `uploaded_file_path` to the request

#### Route Handler Example

```rust
app.post("/upload", |req, res| async move {
    // Check if file was uploaded successfully
    if let Some(uploaded_file) = req.get_data("uploaded_file") {
        let uploaded_file_path = req.get_data("uploaded_file_path").unwrap_or_default();

        res.ok().json(serde_json::json!({
            "success": true,
            "filename": uploaded_file,
            "path": uploaded_file_path,
            "message": "File uploaded successfully"
        }))
    } else {
        // No file was uploaded, but request continues normally
        res.ok().json(serde_json::json!({
            "success": false,
            "message": "No file uploaded or upload failed"
        }))
    }
});
```

#### Middleware Behavior

- **Binary requests**: Files are processed and saved
- **Non-binary requests**: Middleware logs the content type mismatch but continues processing
- **Upload failures**: Errors are logged but don't block the request
- **File data**: Available via `req.get_data("uploaded_file")` and `req.get_data("uploaded_file_path")`

#### Configuration Options

```rust
// Default behavior - saves to "uploads" directory
app.use_middleware("/upload", file_upload(None));

// Custom upload directory
app.use_middleware("/upload", file_upload(Some("user_uploads")));

// Nested directory structure
app.use_middleware("/upload", file_upload(Some("uploads/images")));
```

## Custom Middleware

You can create your own middleware functions for specific needs.

### Authentication Middleware

```rust
app.use_middleware("/protected/", |req, res| async move {
    match req.headers.get("Authorization") {
        Some(auth_header) if auth_header.starts_with("Bearer ") => {
            let token = &auth_header[7..]; // Remove "Bearer " prefix

            if validate_token(token) {
                // Add user data to request
                let mut req = req.clone();
                req.set_data("user_id", "12345");
                req.set_data("user_role", "admin");

                (req, None) // Valid token, continue
            } else {
                (
                    req,
                    Some(res.unauthorized().json(serde_json::json!({
                        "error": "Invalid token"
                    }))),
                )
            }
        }
        _ => (
            req,
            Some(res.unauthorized().json(serde_json::json!({
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

### Request Logging Middleware

```rust
use chrono::Utc;

app.use_middleware("/", |req, res| async move {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    println!(
        "[{}] {} {} - {}",
        timestamp,
        req.method,
        req.path,
        req.headers.get("User-Agent").unwrap_or("Unknown")
    );

    (req, None)
});
```

### Rate Limiting Middleware

```rust
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

lazy_static! {
    static ref REQUEST_COUNTS: Mutex<HashMap<String, (u32, Instant)>> = Mutex::new(HashMap::new());
}

app.use_middleware("/api/", |req, res| async move {
    let client_ip = req.headers.get("X-Forwarded-For")
        .or_else(|| req.headers.get("X-Real-IP"))
        .unwrap_or("unknown")
        .to_string();

    let mut counts = REQUEST_COUNTS.lock().unwrap();
    let now = Instant::now();

    if let Some((count, last_request)) = counts.get_mut(&client_ip) {
        if now.duration_since(*last_request) < Duration::from_secs(60) {
            *count += 1;
            if *count > 100 { // 100 requests per minute
                return (
                    req,
                    Some(res.too_many_requests().json(serde_json::json!({
                        "error": "Rate limit exceeded"
                    }))),
                );
            }
        } else {
            *count = 1;
        }
        *last_request = now;
    } else {
        counts.insert(client_ip.clone(), (1, now));
    }

    (req, None)
});
```

## Middleware Execution Order

Middleware executes in the order it's added to the application:

```rust
// This runs first
app.use_middleware("/", |req, res| async move {
    println!("First middleware");
    (req, None)
});

// This runs second
app.use_middleware("/api/", |req, res| async move {
    println!("API middleware");
    (req, None)
});

// This runs third (only for /api/ routes)
app.use_middleware("/api/users/", |req, res| async move {
    println!("Users API middleware");
    (req, None)
});
```

## Middleware Best Practices

### 1. Order Matters

Place general middleware (like CORS, logging) before specific middleware (like authentication, rate limiting).

### 2. Error Handling

Always handle potential errors in your middleware and return appropriate HTTP status codes.

### 3. Performance

Keep middleware lightweight and avoid blocking operations. Use async/await for I/O operations.

### 4. Data Sharing

Use `req.set_data()` to share information between middleware and route handlers.

### 5. Short-circuiting

Use `(req, Some(response))` to stop processing when necessary (e.g., authentication failures).

## Complete Example

Here's a complete example showing multiple middleware working together:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    middlewares::{cors::cors, logger::logger, file_upload::file_upload},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Global middleware (runs for all routes)
    app.use_middleware("/", cors(None));
    app.use_middleware("/", logger(None));

    // API-specific middleware
    app.use_middleware("/api/", |req, res| async move {
        // Add API version to request
        let mut req = req.clone();
        req.set_data("api_version", "v1");
        (req, None)
    });

    // File upload middleware
    app.use_middleware("/upload", file_upload(None));

    // Routes
    app.get("/api/status", |req, res| async move {
        let version = req.get_data("api_version").unwrap_or_default();
        res.ok().json(serde_json::json!({
            "status": "ok",
            "version": version
        }))
    });

    app.post("/upload", |req, res| async move {
        if let Some(filename) = req.get_data("uploaded_file") {
            res.ok().json(serde_json::json!({
                "success": true,
                "filename": filename
            }))
        } else {
            res.ok().json(serde_json::json!({
                "success": false,
                "message": "No file uploaded"
            }))
        }
    });

    app.listen(3000, || {
        println!("Server running on port 3000");
    }).await;
}
```

This setup provides:

- CORS support for all routes
- Request/response logging
- API version tracking for API routes
- File upload handling for upload routes
- Clean separation of concerns
