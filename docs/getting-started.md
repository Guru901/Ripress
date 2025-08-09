# Ripress

## Overview

Ripress is an Express-inspired web framework for Rust, designed to provide a simple and intuitive experience for handling HTTP requests and responses.

## Features

- Express-like routing with `App` and `Router`
- Async handler support (built on `tokio`)
- Request and response objects (`HttpRequest`, `HttpResponse`)
- Built-in JSON, text, and form parsing
- Middleware support
- Route parameters and query parsing
- Type-safe handler signatures
- Easy error handling
- Extensible via custom middleware

## Installation

```bash
cargo add ripress tokio
```

## Quick Start

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", index);
    app.get("/user/{id}", find_user);
    app.post("/submit", submit_form);

    app.listen(3000, || {
        println!("Server is running on port 3000");
    })
    .await;
}

async fn index(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(200)
        .json(json!({"message": "Welcome to Ripress!"}))
}

async fn find_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.params.get("id");

    match user_id {
        Some(user_id) => res.status(200).text(format!("Hello, {user_id}")),
        None => res.text("User id is required"),
    }
}

async fn submit_form(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.json().unwrap_or(json!({"error": "Invalid JSON"}));
    res.status(200).json(body)
}
```
