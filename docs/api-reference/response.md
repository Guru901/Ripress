# Ripress HttpResponse API Reference

## Overview

The `HttpResponse` object in Ripress provides methods for constructing HTTP responses with various content types, status codes, headers, and cookies. This document demonstrates common usage patterns and examples.

## Basic Usage

### Text Responses

Send plain text responses using the `.text()` method.

```rust
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::types::RouterFns;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || println!("Server running on port 3000"))
        .await
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello, World!")
}
```

### HTML Responses

Send HTML responses using the `.html()` method.

```rust
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::types::RouterFns;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || println!("Server running on port 3000"))
        .await
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().html("<h1>Hello, World!</h1>")
}
```

### Sending a file response

To return a file response, use `.send_file()` with the path to the file.

```rust
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::types::RouterFns;

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

### JSON Responses

Send JSON responses using the `.json()` method with any serializable type.

```rust
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::types::RouterFns;
use serde::Serialize;

#[derive(Serialize)]
struct User {
    name: String,
    age: u32,
}

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || println!("Server running on port 3000"))
        .await
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user = User {
        name: "John".to_string(),
        age: 30,
    };

    res.ok().json(user)
}
```

## Status Codes

### Custom Status Codes

Set specific status codes using `.status()`:

```rust
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::types::RouterFns;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || println!("Server running on port 3000"))
        .await
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(201).json(serde_json::json!({
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

## Headers

### Setting Headers

Add custom headers using `.set_header()`:

```rust
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::types::RouterFns;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || println!("Server running on port 3000"))
        .await
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_header("X-Custom-Header", "value")
        .ok()
        .text("Headers set")
}
```

### Getting Headers

Retrieve header values using `.get_header()`:

```rust
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::types::RouterFns;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || println!("Server running on port 3000"))
        .await
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let headers = res.headers.clone();

    if let Some(value) = headers.get("X-Custom-Header") {
        res.ok().text(format!("Header value: {}", value))
    } else {
        res.not_found().text("Header not found")
    }
}
```

Returns `Option<&str>`.

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
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::res::{CookieOptions, CookieSameSiteOptions};
use ripress::types::RouterFns;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || println!("Server running on port 3000"))
        .await
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_cookie(
        "session",
        "abc123",
        CookieOptions {
            http_only: true,
            secure: true,
            same_site: CookieSameSiteOptions::Strict,
            path: Some("/"),
            domain: Some(""),
            max_age: None,
            expires: None,
        },
    )
    .ok()
    .text("Cookie set")
}
```

### Removing Cookies

Remove cookies using `.clear_cookie()`:

```rust
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};
use ripress::types::RouterFns;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || println!("Server running on port 3000"))
        .await
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.clear_cookie("session").ok().text("Cookie removed")
}
```

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
- `.not_found()` - 404 Not Found (implied from example)

### Headers

- `.set_header(name, value)` - Set custom header
- `.get_header(name)` - Get header value (returns `Option<&str>`)

### Cookies

- `.set_cookie(name, value, options)` - Set cookie with options
- `.clear_cookie(name)` - Remove cookie

All methods support fluent chaining for building complex responses.
