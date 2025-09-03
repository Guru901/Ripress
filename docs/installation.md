# Installation

This guide will walk you through setting up Ripress in your Rust project and getting your first web server running.

## Prerequisites

Before installing Ripress, make sure you have:

- **Rust 1.75 or higher** - [Install Rust](https://rustup.rs/)
- **Cargo** (comes with Rust)

You can check your Rust version with:

```bash
rustc --version
```

## Adding Ripress to Your Project

### New Project

If you're starting a new project, create it with Cargo:

```bash
cargo new my-ripress-app
cd my-ripress-app
```

### Adding Dependencies

Add Ripress and the required async runtime to your `Cargo.toml`:

```bash
cargo add ripress
cargo add tokio --features macros,rt-multi-thread
```

Alternatively, you can manually add them to your `Cargo.toml`:

```toml
[dependencies]
ripress = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

### WebSocket Support (with-wynd feature)

Ripress includes built-in WebSocket support through the `wynd` crate. The "with-wynd" feature is enabled by default, but you can explicitly control it:

```toml
[dependencies]
ripress = { version = "1", features = ["with-wynd"] }  # Enable WebSocket support (default)
wynd = "0.4"  # WebSocket library
```

Or disable WebSocket support:

```toml
[dependencies]
ripress = { version = "1", default-features = false }  # Disable WebSocket support
```

With WebSocket support enabled, you can create real-time applications:

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let mut wynd = Wynd::new();

    // HTTP route
    app.get("/", |_, res| async move {
        res.ok().text("Hello, World!")
    });

    // WebSocket connection handler
    wynd.on_connection(|conn| async move {
        conn.on_text(|event, _| async move {
            println!("Received: {}", event.data);
        });
    });

    // Mount WebSocket at /ws path
    app.use_wynd("/ws", wynd.handler());

    app.listen(3000, || {
        println!("Server running on http://localhost:3000");
        println!("WebSocket available at ws://localhost:3000/ws");
    })
    .await;
}
```

### Optional Dependencies

Depending on your use case, you might want to add these common dependencies:

```bash
# For JSON serialization/deserialization
cargo add serde --features derive
cargo add serde_json

# For environment variables
cargo add dotenv

# For UUID support (often useful for IDs)
cargo add uuid --features v4
```

## Verify Installation

Create a simple "Hello World" server to verify everything is working.

First, add the JSON dependencies:

```bash
cargo add serde --features derive
cargo add serde_json
```

Replace the contents of `src/main.rs` with:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", hello_world);
    app.get("/health", health_check);

    app.listen(3000, || {
        println!("🚀 Server running on http://localhost:3000");
        println!("📖 Try: curl http://localhost:3000");
    })
    .await;
}

async fn hello_world(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(200)
        .json(json!({
            "message": "Hello, Ripress! 🦀",
            "version": "1.x",
            "framework": "ripress"
        }))
}

async fn health_check(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(200)
        .json(json!({"status": "ok", "timestamp": chrono::Utc::now()}))
}
```

Run your server:

```bash
cargo run
```

You should see:

```
🚀 Server running on http://localhost:3000
📖 Try: curl http://localhost:3000
```

Test your endpoints:

```bash
# Main endpoint
curl http://localhost:3000

# Health check
curl http://localhost:3000/health
```

Expected responses:

```json
// GET /
{
  "message": "Hello, Ripress! 🦀",
  "version": "1.0.1",
  "framework": "ripress"
}

// GET /health
{
  "status": "ok",
  "timestamp": "2025-08-14T10:30:00Z"
}
```

## Common Installation Issues

### Compilation Errors

If you encounter compilation errors, make sure:

1. Your Rust version is up to date:

   ```bash
   rustup update
   ```

2. Your dependencies are compatible:

   ```bash
   cargo update
   ```

3. Clear your cache if needed:
   ```bash
   cargo clean
   cargo build
   ```

### Port Already in Use

If port 3000 is already in use, change it in your code:

```rust
app.listen(8080, || {
    println!("🚀 Server running on http://localhost:8080");
})
.await;
```

### Missing Features

If you get errors about missing features, ensure you have the correct Tokio features:

```toml
tokio = { version = "1.0", features = ["macros", "rt-multi-thread", "net"] }
```

## Development Setup

For development, install additional tools to improve your workflow:

```bash
# For automatic reloading during development
cargo install cargo-watch

# For better error messages
cargo install cargo-expand

# Run your server with auto-reload
cargo watch -x run
```

### Development Configuration

Create a `.cargo/config.toml` for faster builds:

```toml
[build]
rustflags = ["-C", "link-arg=-fuse-ld=lld"] # Faster linking (if you have lld installed)
```

## Next Steps

Now that you have Ripress installed, you can:

- 📚 Follow the [Getting Started Guide](./getting-started) for a comprehensive tutorial
- 💡 Check out the [Examples](./examples) for common use cases
- 📖 Explore the [API Reference](./api-references) for detailed documentation
- 🎯 Try the [Tutorials](./tutorials) for step-by-step projects

## Performance Tips

For production deployments:

```bash
# Build with optimizations
cargo build --release

# Run the optimized binary
./target/release/my-ripress-app
```

---

✅ **Installation Complete!** You're now ready to build fast, reliable web applications with Ripress 1.x!
