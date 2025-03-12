# Ripress

## Overview

Ripress is an Express-inspired web framework for Rust, designed to provide a simple and intuitive experience for handling HTTP requests and responses.

## Features

- Express-like routing.
- JSON and text response helpers.
- Query parameter and URL parameter extraction.
- Support for application/json and text/plain request bodies.
- Future plans:
  - Middleware support.
  - Better error handling with response helpers like `res.ok()` and `res.bad_request()`.

## Installation

```bash
cargo add ripress
```

## Quick Start

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", index);
    app.get("/user/{id}", find_user);
    app.post("/submit", submit_form);

    app.listen("127.0.0.1:3000").await;
}

async fn index(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(200)
        .json(json!({"message": "Welcome to Ripress!"}))
}

async fn find_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.get_params("id").unwrap_or("unknown".to_string());
    res.status(200).text(format!("Hello, {user_id}"))
}

async fn submit_form(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.json().unwrap_or(json!({"error": "Invalid JSON"}));
    res.status(200).json(body)
}
```
