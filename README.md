# Ripress

### An Express.js-inspired web framework for Rust

Please star the repo if you like it, so that I know someone is using it.

## What is Ripress?

Ripress is an Express.js-inspired web framework for Rust that combines the familiar developer experience of Express with the performance and safety of Rust. Built on top of [Hyper](https://hyper.rs) and [Tokio](https://tokio.rs), Ripress provides a simple and intuitive API for building fast, reliable web applications.

## Why Choose Ripress?

**🚀 Performance**

- Ripress is just ~6–7% slower than Actix-Web (one of the fastest Rust web frameworks), even though performance hasn’t been the main focus yet.
- Nearly 10x faster than Express.js

**💡 Developer Experience**

- Express.js-familiar API that Rust developers will love
- In many cases, even simpler than Express.js
- Comprehensive error handling and type safety

**⚡ Modern Foundation**

- Async/await support with Tokio
- HTTP/2 support via Hyper

![Performance Benchmark](./benchmark.png)

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Goals](#goals)
- [Installation](#installation)
- [Examples](#basic-example)
- [Documentation](#documentation)
- [Changelog](#changelog)

---

## Overview

Ripress is a web framework inspired by Express.js, designed to bring the familiar and intuitive Express.js developer experience to the Rust ecosystem while maintaining Rust's performance and safety guarantees.

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
    middlewares::{file_upload::file_upload},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Add CORS and file upload middleware
    app.use_cors(None)
        .use_middleware("/upload", file_upload(None))
        .use_rate_limiter(None);

    app.listen(3000, || {
        println!("Server running on port 3000");
    }).await;
}
```

Learn more about middleware in the [Middleware Guide](./docs/guides/middleware.md).

## Get Started

Ready to build something amazing? Jump into our [Installation Guide](./installation) or explore the framework on [GitHub](https://github.com/guru901/ripress).

You can also check out the complete API documentation on [Docs.rs](https://docs.rs/ripress/latest/ripress/).

## Documentation

- [Getting Started Guide](./docs/getting-started.md)
- [Middleware Guide](./docs/guides/middleware.md)
- [API Reference](./docs/api-reference/)

## Changelog

[View Changelog](./CHANGELOG.md)

---

_Ripress v1.0.1 - Production Ready_ ✨
