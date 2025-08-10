# Response Examples

The `HttpResponse` object in Ripress provides various methods for handling responses, including sending text, JSON, status codes, and cookies. This document demonstrates different response-handling scenarios.

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

    app.listen(3000, || {}).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello, World!")
}
```

### Sending an HTML Responses

Send html responses using the `.html()` method.

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
    res.ok().html("<h1>Hello, World!</h1>")
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

    app.listen(3000, || {}).await;
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
    res.ok()
}
```

#### Error Responses

```rust
use ripress::context::{HttpRequest, HttpResponse};

// 400 Bad Request
async fn bad_request(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.bad_request()
}

// 404 Not Found
async fn not_found(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.not_found()
}

// 401 Unauthorized
async fn unauthorized(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.unauthorized()
}

// 500 Internal Server Error
async fn internal_error(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.internal_server_error()
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

    app.listen(3000, || {}).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_header("X-Request-ID", "abc-123")
        .set_header("X-Custom-Header", "custom-value")
        .ok()
        .json(serde_json::json!({ "status": "success" }))
}

async fn check_headers(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let headers = res.headers.clone();
    match headers.get("X-Custom-Header") {
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

    app.get("/", set_session);
    app.get("/logout", logout);

    app.listen(3000, || {}).await;
}

async fn set_session(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_cookie("session_id", "abc123", CookieOptions::default())
        .set_cookie("user_id", "user_123", CookieOptions::default())
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

    app.get("/login", login);

    app.listen(3000, || {}).await;
}

async fn login(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_cookie("session_id", "abc123", CookieOptions::default())
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
use ripress::res::CookieOptions;
use ripress::{app::App, types::RouterFns};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || {}).await;
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

    app.get("/", handler);

    app.listen(3000, || {}).await;
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

    app.get("/", handler);

    app.listen(3000, || {}).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let stream = stream::unfold(0, |state| async move {
        if state < 100 {
            // Simulate some processing time
            time::sleep(Duration::from_millis(100)).await;

            let data = format!("Update {}\n", state,);

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

    app.get("/", handler);

    app.listen(3000, || {}).await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let file = match File::open("large_file.txt").await {
        Ok(f) => f,
        Err(e) => {
            return res
                .internal_server_error()
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

These examples demonstrate different use cases for streaming responses, from simple number sequences to real-time updates and file streaming.
