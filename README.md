# Ripress

### An express-inspired Rust-based web framework

Please star the repo if you like it, so that I know someone is using it.

## Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Installation](#installation)
- [Examples](#basic-example)
- [Documentation](#documentation)
- [Changelog](#changelog)

---

## Overview

Ripress is a web framework inspired by Express.js.

## Features

- **Express-like routing** with `App` and `Router`
- **Async handler support** built on `tokio`
- **Built-in middleware** including CORS, logging, and file uploads
- **Request/response objects** with JSON, text, and form parsing
- **Type-safe handler signatures** for better developer experience
- **Extensible architecture** via custom middleware

## Goals

- Provide an intuitive and simple API like Express.js
- Focus on developer experience first; performance optimizations will come later
- Prioritize ease of use over low-level control initially
- Include built-in middleware for common web development needs

---

## Installation

You can add `ripress` to your project using Cargo:

```sh
cargo add ripress
cargo add tokio --features macros,rt-multi-thread
```

## Basic Example

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

    app.listen(3000, || {
        println!("Server is running on port 3000");
    })
    .await;
}

async fn handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(200)
        .json(json!({"message": "Welcome to Ripress!"}))
}
```

View more basic examples in [Examples](./docs/example/) dir.

View full blown code examples [here](https://github.com/Guru901/ripress-examples).

## Middleware Example

```rust
use ripress::{
    app::App,
    middlewares::{cors::cors, file_upload::file_upload},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Add CORS and file upload middleware
    app.use_middleware("/", cors(None));
    app.use_middleware("/upload", file_upload(None));

    app.listen(3000, || {
        println!("Server running on port 3000");
    }).await;
}
```

Learn more about middleware in the [Middleware Guide](./docs/guides/middleware.md).

## Documentation

[Getting Started Guide](./docs/getting-started.md)  
[Middleware Guide](./docs/guides/middleware.md)  
[API Reference](./docs/api-reference/)

## Changelog

[View Changelog](./CHANGELOG.md)
