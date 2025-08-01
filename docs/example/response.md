# Response Examples

The `HttpResponse` object in Ripress provides various methods for handling responses, including sending text, JSON, status codes, and cookies. This document demonstrates different response-handling scenarios.

## Basic Responses

### Sending a Plain Text Response

Send text responses using the `.text()` method.

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn text_response(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
       .text("Hello, World!")
}
```

### Sending an HTML Responses

Send html responses using the `.html()` method.

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
       .html("<h1>Hello, World!</h1>")
}
```

### Sending a JSON Response

To return a JSON response, use `.json()` with a serializable Rust struct.

```rust
use ripress::context::{HttpRequest, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
struct Message {
    message: String,
    code: i32,
}

async fn json_response(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let response_body = Message {
        message: "Success".to_string(),
        code: 200,
    };

    res.ok()
       .json(response_body)
}

async fn quick_json(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
       .json(serde_json::json!({
           "message": "Success",
           "code": 200
       }))
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
       .json(serde_json::json!({
           "status": "success",
           "data": { "id": 1, "name": "John" }
       }))
}
```

#### Error Responses

```rust
use ripress::context::{HttpRequest, HttpResponse};

// 400 Bad Request
async fn bad_request(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.bad_request().json(serde_json::json!({
        "error": "Invalid input",
        "details": ["name is required", "age must be positive"]
    }))
}

// 404 Not Found
async fn not_found(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.not_found().json(serde_json::json!({
        "error": "Resource not found",
        "resource": "user/123"
    }))
}

// 401 Unauthorized
async fn unauthorized(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.unauthorized().text("Unauthorized")
}

// 500 Internal Server Error
async fn internal_error(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.internal_server_error().json(serde_json::json!({
        "error": "Internal server error",
        "request_id": "abc-123"
    }))
}
```

## Headers and Cookies

### Working with Headers

```rust
use ripress::context::{HttpRequest, HttpResponse};

// Setting multiple headers
async fn set_headers(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_header("X-Request-ID", "abc-123")
       .set_header("X-Custom-Header", "custom-value")
       .ok()
       .json(serde_json::json!({ "status": "success" }))
}

// Reading headers
async fn check_headers(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match res.get_header("X-Custom-Header") {
        Ok(value) => res.ok()
                       .json(serde_json::json!({ "header": value })),
        Err(_) => res.bad_request()
                     .text("Missing required header")
    }
}
```

### Managing Cookies

```rust
use ripress::{
    context::{HttpRequest, HttpResponse},
    res::CookieOptions,
};

// Setting cookies
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
use ripress::{
    context::{HttpRequest, HttpResponse},
    res::CookieOptions,
};

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

async fn validation_error(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
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
use ripress::context::{HttpRequest, HttpResponse};
use bytes::Bytes;
use futures::stream;

async fn basic_stream(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let stream = stream::iter(0..5)
        .map(|n| Ok::<Bytes, std::io::Error>(Bytes::from(format!("Number: {}\n", n))));

    res.write(stream)
}
```

### Real-time Updates Stream

Here's an example of streaming real-time updates with delays:

```rust
use ripress::context::{HttpRequest, HttpResponse};
use bytes::Bytes;
use futures::stream;
use tokio::time;
use std::time::Duration;

async fn realtime_updates(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let stream = stream::unfold(0, |state| async move {
        if state < 100 {
            // Simulate some processing time
            time::sleep(Duration::from_millis(100)).await;

            let data = format!("Update {}: {}\n",
                state,
                chrono::Local::now().format("%H:%M:%S")
            );

            Some((
                Ok::<Bytes, std::io::Error>(Bytes::from(data)),
                state + 1,
            ))
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
use ripress::context::{HttpRequest, HttpResponse};
use bytes::Bytes;
use futures::stream;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};

async fn stream_file(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let file = File::open("large_file.txt").await.unwrap();
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

    res.set_header("Content-Type", "text/plain")
       .write(stream)
}
```

```

These examples demonstrate different use cases for streaming responses, from simple number sequences to real-time updates and file streaming.
```
