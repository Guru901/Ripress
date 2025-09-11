# Ripress HttpResponse API Reference

## Overview

The `HttpResponse` object in Ripress provides various methods for handling responses, including sending text, JSON, status codes, and cookies. This document demonstrates different response-handling scenarios with practical examples.

## Basic Responses

### Sending a Plain Text Response

Send text responses using the `.text()` method.

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

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello, World!")
}
```

### Sending an HTML Response

Send HTML responses using the `.html()` method.

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

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().html("<h1>Hello, World!</h1>")
}
```

### Sending a file response

To return a file response, use `.send_file()` with the path to the file.

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

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.send_file("./public/index.html").await
}
```

### Sending a JSON Response

To return a JSON response, use `.json()` with a serializable Rust struct.

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};
use serde::Serialize;

#[derive(Serialize)]
struct Message {
    message: String,
    code: i32,
}

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let response_body = Message {
        message: "Success".to_string(),
        code: 200,
    };

    res.ok().json(response_body)
}
```

## Status Codes

### Setting a Custom Status Code

You can manually set any status code using `.status()`.

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn custom_status(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(201)  // Created
       .json(serde_json::json!({
           "message": "Resource created",
           "id": "123"
       }))
}

// Fun example
async fn teapot(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(418)  // I'm a teapot
       .text("Sorry, I'm a teapot, I cannot brew coffee!")
}
```

### Status Code Helpers

Ripress provides convenient helper methods for common status codes:

#### Success Responses

```rust
use ripress::context::{HttpRequest, HttpResponse};

// 200 OK
async fn ok_response(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().json(serde_json::json!({
        "message": "Request successful"
    }))
}
```

#### Error Responses

```rust
use ripress::context::{HttpRequest, HttpResponse};

// 400 Bad Request
async fn bad_request(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.bad_request().json(serde_json::json!({
        "error": "Invalid input provided"
    }))
}

// 404 Not Found
async fn not_found(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.not_found().json(serde_json::json!({
        "error": "Resource not found"
    }))
}

// 401 Unauthorized
async fn unauthorized(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(401).json(serde_json::json!({
        "error": "Authentication required"
    }))
}

// 500 Internal Server Error
async fn internal_error(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(500).json(serde_json::json!({
        "error": "Internal server error"
    }))
}
```

## Headers and Cookies

### Working with Headers

```rust
use ripress::context::{HttpRequest, HttpResponse};
use ripress::{app::App, types::RouterFns};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);
    app.get("/check-headers", check_headers);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_header("X-Request-ID", "abc-123")
        .set_header("X-Custom-Header", "custom-value")
        .ok()
        .json(serde_json::json!({ "status": "success" }))
}

async fn check_headers(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.headers.get("X-Custom-Header") {
        Some(value) => res.ok().json(serde_json::json!({ "header": value })),
        None => res.bad_request().text("Missing required header"),
    }
}
```

### Managing Cookies

```rust
use ripress::context::{HttpRequest, HttpResponse};
use ripress::res::CookieOptions;
use ripress::{app::App, types::RouterFns};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/login", set_session);
    app.get("/logout", logout);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn set_session(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_cookie("session_id", "abc123", None)
        .set_cookie("user_id", "user_123", None)
        .ok()
        .json(serde_json::json!({
            "message": "Session started"
        }))
}

// Removing cookies (logout example)
async fn logout(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.clear_cookie("session_id")
        .clear_cookie("user_id")
        .ok()
        .json(serde_json::json!({
            "message": "Logged out successfully"
        }))
}
```

## Complete Examples

### Authentication Response

```rust
use ripress::context::{HttpRequest, HttpResponse};
use ripress::res::CookieOptions;
use ripress::{app::App, types::RouterFns};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.post("/login", login);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn login(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_cookie("session_id", "abc123", None)
        .set_header("X-Auth-Token", "jwt_token_here")
        .ok()
        .json(serde_json::json!({
            "status": "success",
            "user": {
                "id": 1,
                "name": "John Doe",
                "role": "admin"
            }
        }))
}
```

### Error Response with Details

```rust
use ripress::context::{HttpRequest, HttpResponse};
use ripress::{app::App, types::RouterFns};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.post("/validate", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.bad_request()
        .set_header("X-Error-Code", "VALIDATION_ERROR")
        .json(serde_json::json!({
            "error": "Validation failed",
            "code": "VALIDATION_ERROR",
            "details": {
                "fields": [
                    {"field": "email", "error": "Invalid email format"},
                    {"field": "age", "error": "Must be over 18"}
                ]
            }
        }))
}
```

## Streaming Responses

### Basic Stream Response

Here's a basic example of streaming numbers:

```rust
use bytes::Bytes;
use futures::StreamExt;
use futures::stream;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::{app::App, types::RouterFns};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/stream", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let stream = stream::iter(0..5)
        .map(|n| Ok::<Bytes, std::io::Error>(Bytes::from(format!("Number: {}\n", n))));

    res.write(stream)
}
```

### Real-time Updates Stream

Here's an example of streaming real-time updates with delays:

```rust
use std::time::Duration;
use bytes::Bytes;
use futures::stream;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::{app::App, types::RouterFns};
use tokio::time;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/updates", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let stream = stream::unfold(0, |state| async move {
        if state < 100 {
            // Simulate some processing time
            time::sleep(Duration::from_millis(100)).await;

            let data = format!("Update {}\n", state);

            Some((Ok::<Bytes, std::io::Error>(Bytes::from(data)), state + 1))
        } else {
            None
        }
    });

    res.write(stream)
}
```

### File Streaming Example

Here's an example of streaming a large file:

```rust
use bytes::Bytes;
use futures::stream;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::{app::App, types::RouterFns};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/download", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let file = match File::open("large_file.txt").await {
        Ok(f) => f,
        Err(e) => {
            return res
                .status(500)
                .text(format!("Failed to open file: {}", e));
        }
    };

    let reader = BufReader::new(file);

    let stream = stream::unfold(reader, |mut reader| async move {
        let mut buffer = vec![0; 1024];
        match reader.read(&mut buffer).await {
            Ok(n) if n > 0 => {
                buffer.truncate(n);
                Some((Ok::<Bytes, std::io::Error>(Bytes::from(buffer)), reader))
            }
            _ => None,
        }
    });

    res.set_header("Content-Type", "text/plain").write(stream)
}
```

The `.write()` method:

- Accepts any `Stream` that implements `Stream<Item = Result<Bytes, E>>`
- Automatically sets the content type to `text/event-stream`
- Maintains a keep-alive connection
- Streams the data chunks to the client

## Method Chaining

All response methods support chaining for a fluent API:

```rust
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::res::CookieOptions;
use ripress::types::RouterFns;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || println!("Server running on port 3000"))
        .await
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_header("X-Custom", "value")
        .set_cookie("session", "abc123", CookieOptions::default())
        .ok()
        .json(serde_json::json!({
            "status": "success"
        }))
}
```

## Quick Reference

### Response Types

- `.text(content)` - Plain text response
- `.html(content)` - HTML response
- `.json(data)` - JSON response
- `.write(stream)` - Streaming response

### Status Code Methods

- `.ok()` - 200 OK
- `.status(code)` - Custom status code
- `.bad_request()` - 400 Bad Request
- `.not_found()` - 404 Not Found

### Headers and Cookies

- `.set_header(name, value)` - Set custom header
- `.set_cookie(name, value, options)` - Set cookie with options
- `.clear_cookie(name)` - Remove cookie

### Streaming

- `.write(stream)` - Stream data to client
- Supports any `Stream<Item = Result<Bytes, E>>`
- Automatically sets appropriate headers

All methods support fluent chaining for building complex responses.
