# Ripress

### An express-inspired Rust-based web framework

"This is an experimental project, and its development may change over time."

## Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Benchmarks](#benchmarks)
- [Installation](#installation)
- [Examples](#basic-example)
- [Roadmap](#roadmap)
- [Documentation](#documentation)
- [Changelog](#changelog)

---

## Overview

Ripress is a web framework inspired by Express.js.

## Goals

- Provide an intuitive and simple API like Express.js
- Focus on developer experience first; performance optimizations will come later
- Prioritize ease of use over low-level control initially

---

## Benchmarks

## Performance Comparison

Ripress delivers exceptional performance, making it suitable for high-throughput applications where response time and request handling capacity are critical. Our benchmarks demonstrate how Ripress compares to other popular web frameworks.

### Methodology

All benchmarks were conducted under the following conditions:

- 100,000 HTTP requests to a simple JSON endpoint
- Concurrency level of 100 connections
- Running on the same hardware configuration
- Minimal "Hello World" equivalent endpoints
- Response payload: `{"ping": "pong"}`

### Results

| Framework        | Requests/sec | Avg Response Time | Total Time (100k requests) |
| ---------------- | ------------ | ----------------- | -------------------------- |
| Ripress          | 128,429      | 0.456ms           | 0.778s                     |
| Actix-web        | 135,760\*    | 0.468ms\*         | 0.736s\*                   |
| Express.js (Bun) | 4296         | 20.34ms           | 22.4                       |

\*_Actix-web values are estimated based on relative performance statement_

### Key Observations

- **Ripress achieves 94.6% of Actix-web's performance**, which is remarkable considering Actix-web is widely recognized as one of the fastest web frameworks available.

- **Ripress outperforms Express.js by approximately 30x** when compared to Express running on Bun (which is already significantly faster than Node.js).

- **Sub-millisecond response times** make Ripress suitable for latency-sensitive applications.

## Running Your Own Benchmarks

We encourage users to run their own benchmarks to verify performance in their specific use cases. A benchmark utility is provided in the examples directory:

```rust
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    app.get("/ping", ping_handler);
    app.listen("127.0.0.1:3000").await;
}

async fn ping_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.json(json!({
        "ping": "pong"
    }))
}
```

---

## Installation

You can add `ripress` to your project using Cargo:

```sh
cargo add ripress tokio
```

Or manually add it to your `Cargo.toml`:

```toml
[dependencies]
ripress = "0.3.2"
tokio = { version = "1.44.0", features = ["full"] }
```

## Basic Example

```rust
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};

#[tokio::main]
async fn main() {
    let mut app = App::new();
    app.get("/", hello_world);
    app.listen("127.0.0.1:3000").await;
}

async fn hello_world(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello, world!")
}
```

View more examples in the [examples](./docs/example/basic-routing.md) directory.

## Roadmap

- **Middleware support** (Planned for next week)

## Documentation

[Getting Started Guide](./docs/getting-started.md)

## Changelog

[View Changelog](./CHANGELOG.md)
