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

## Goals

- Provide an intuitive and simple API like Express.js
- Focus on developer experience first; performance optimizations will come later
- Prioritize ease of use over low-level control initially

---

## Installation

You can add `ripress` to your project using Cargo:

```sh
cargo add ripress tokio
```

Or manually add it to your `Cargo.toml`:

```toml
[dependencies]
ripress = "0.6.1"
tokio = { version = "1.46.1", features = ["full"] }
```

## Basic Example

```rust
use ripress::app::App;
use ripress::context::{HttpRequest, HttpResponse};

#[tokio::main]
async fn main() {
    let mut app = App::new();
    app.get("/", hello_world);
    app.listen(3000, || {}).await;
}

async fn hello_world(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello, world!")
}
```

View more basic examples in [Examples](./docs/example/) dir.

View full blown code examples [here](https://github.com/Guru901/ripress-examples).

## Documentation

[Getting Started Guide](./docs/getting-started.md)

## Changelog

[View Changelog](./CHANGELOG.md)
