# Ripress 🚀

## Overview

Ripress is an Express-inspired web framework for Rust, designed to provide a simple and intuitive experience for handling HTTP requests and responses. 🦀

## Features ✨

- **Express-like routing** with `App` and `Router`
- **Async handler support** (built on `tokio`)
- **Request and response objects** (`HttpRequest`, `HttpResponse`)
- **Built-in parsing** for JSON, text, and form data
- **Middleware support** for extensible request/response processing
- **Route parameters and query parsing**
- **Built-in file upload middleware** with automatic type detection
- **Type-safe handler signatures**
- **Easy error handling**
- **WebSocket support** via the `wynd` crate (with "with-wynd" feature)
- **Extensible** via custom middleware

## Installation

Add Ripress and Tokio to your `Cargo.toml`:

```bash
cargo add ripress
cargo add tokio --features macros,rt-multi-thread
```

Or manually:

```toml
[dependencies]
ripress = "1.0.1"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
serde_json = "1.0"  # For JSON handling
```

### WebSocket Support

Ripress includes built-in WebSocket support through the `wynd` crate. The "with-wynd" feature is optional and must be explicitly enabled:

```toml
[dependencies]
ripress = { version = "1.0.1", features = ["with-wynd"] }  # Enable WebSocket support
wynd = "0.4"  # WebSocket library
```

## Quick Start

Here's a complete example that demonstrates core Ripress features including WebSocket support:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};
use serde_json::json;
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let mut wynd = Wynd::new();

    // Define HTTP routes
    app.get("/", index);
    app.get("/user/{id}", find_user);
    app.post("/submit", submit_form);

    // WebSocket connection handler
    wynd.on_connection(|conn| async move {
        conn.on_text(|event, _| async move {
            println!("WebSocket message: {}", event.data);
        });
    });

    // Mount WebSocket at /ws path
    app.use_wynd("/ws", wynd.handler());

    // Start the server
    app.listen(3000, || {
        println!("🚀 Server is running on http://localhost:3000");
        println!("🔌 WebSocket available at ws://localhost:3000/ws");
    })
    .await;
}

// Handler functions
async fn index(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(200)
        .json(json!({"message": "Welcome to Ripress! 🦀"}))
}

async fn find_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.params.get("id");
    match user_id {
        Some(user_id) => {
            res.status(200).json(json!({
                "user_id": user_id,
                "message": format!("Hello, {}!", user_id)
            }))
        },
        None => {
            res.status(400).json(json!({
                "error": "User ID is required"
            }))
        }
    }
}

async fn submit_form(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.json() {
        Ok(body) => {
            res.status(200).json(json!({
                "message": "Data received successfully",
                "data": body
            }))
        },
        Err(_) => {
            res.status(400).json(json!({
                "error": "Invalid JSON payload"
            }))
        }
    }
}
```

## Testing Your App

Once you run `cargo run`, you can test your endpoints:

```bash
# Get welcome message
curl http://localhost:3000/

# Get user by ID
curl http://localhost:3000/user/123

# Submit JSON data
curl -X POST http://localhost:3000/submit \
  -H "Content-Type: application/json" \
  -d '{"name": "John", "age": 30}'

# Test WebSocket connection (using wscat or similar tool)
wscat -c ws://localhost:3000/ws
```

## What's Next?

- 📖 [Routing Guide](./guides/routing.md) - Learn about advanced routing patterns
- 🔧 [Middleware](./guides/middleware.md) - Add custom middleware to your app
- 📝 [Request & Response](./guides/request-response.md) - Deep dive into handling HTTP data
- 🔌 [WebSocket Guide](./guides/websocket.md) - Build real-time applications
- 🎯 [Examples](./example/) - More practical examples and use cases

---

Ready to build something amazing with Ripress? Let's dive deeper! 🚀
