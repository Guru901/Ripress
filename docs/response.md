# Response Object (HttpResponse)

## Overview

The `HttpResponse` object in Ripress provides methods for constructing HTTP responses with various content types, status codes, headers, and cookies. This document demonstrates common usage patterns and examples.

## Basic Usage

### Text Responses

Send plain text responses using the `.text()` method.

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
       .text("Hello, World!")
}
```

### HTML Responses

Send html responses using the `.html()` method.

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
       .html("<h1>Hello, World!</h1>")
}
```

### JSON Responses

Send JSON responses using the `.json()` method with any serializable type.

```rust
use ripress::context::{HttpRequest, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
struct User {
    name: String,
    age: u32
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user = User {
        name: "John".to_string(),
        age: 30
    };

    res.ok()
       .json(user)
}
```

## Status Codes

### Custom Status Codes

Set specific status codes using `.status()`:

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(201)
       .json(serde_json::json!({
           "message": "Resource created"
       }))
}
```

### Helper Methods

Common status codes have dedicated helper methods:

#### 200 OK

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok()
       .text("Success")
}
```

#### 400 Bad Request

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.bad_request()
       .json(serde_json::json!({
           "error": "Invalid input"
       }))
}
```

#### 404 Not Found

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.not_found()
       .text("Resource not found")
}
```

#### 401 Unauthorized

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.unauthorized()
       .text("Unauthorized")
}
```

#### 500 Internal Server Error

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.internal_server_error()
       .json(serde_json::json!({
           "error": "Internal server error"
       }))
}
```

## Headers

### Setting Headers

Add custom headers using `.set_header()`:

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_header("X-Custom-Header", "value")
       .ok()
       .text("Headers set")
}
```

### Getting Headers

Retrieve header values using `.get_header()`:

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match res.get_header("X-Custom-Header") {
        Ok(value) => res.ok().text(format!("Header value: {}", value)),
        Err(_) => res.not_found().text("Header not found")
    }
}
```

Returns `Result<String, HttpResponseError>`.

## Streaming Responses

### Stream Response

Send streaming responses using the `.write()` method with any Stream that produces `Result<Bytes, E>`.

```rust
use ripress::context::{HttpRequest, HttpResponse};
use bytes::Bytes;
use futures::stream;
use tokio::time;
use std::time::Duration;

async fn stream_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let stream = stream::unfold(0, |state| async move {
        if state < 500 {
            time::sleep(Duration::from_millis(10)).await;
            Some((
                Ok::<Bytes, std::io::Error>(Bytes::from(format!("Chunk {}\n", state))),
                state + 1,
            ))
        } else {
            None
        }
    });

    res.write(stream)
}
```

The `.write()` method:

- Accepts any `Stream` that implements `Stream<Item = Result<Bytes, E>>`
- Automatically sets the content type to `text/event-stream`
- Maintains a keep-alive connection
- Streams the data chunks to the client

## Cookies

### Setting Cookies

Set cookies using `.set_cookie()`:

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_cookie("session", "abc123")
       .ok()
       .text("Cookie set")
}
```

### Removing Cookies

Remove cookies using `.clear_cookie()`:

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.clear_cookie("session")
       .ok()
       .text("Cookie removed")
}
```

## Content Type

The content type is automatically set based on the response method used (`.json()`, `.text()`), but can be manually set:

```rust
use ripress::context::{HttpRequest, HttpResponse};
use ripress::types::ResponseContentType;

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_content_type(ResponseContentType::JSON)
       .ok()
       .json(serde_json::json!({
           "message": "Custom content type"
       }))
}
```

## Method Chaining

All response methods support chaining for a fluent API:

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_header("X-Custom", "value")
       .set_cookie("session", "abc123")
       .ok()
       .json(serde_json::json!({
           "status": "success"
       }))
}
```
