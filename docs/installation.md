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
ripress = "0.1"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

### Optional Dependencies

Depending on your use case, you might want to add these common dependencies:

```bash
# For JSON serialization/deserialization
cargo add serde --features derive
cargo add serde_json

# For environment variables
cargo add dotenv
```

## Verify Installation

Create a simple "Hello World" server to verify everything is working:

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

    app.listen(3000, || {
        println!("🚀 Server running on http://localhost:3000");
    })
    .await;
}

async fn hello_world(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(200)
        .json(json!({"message": "Hello, Ripress!"}))
}
```

Run your server:

```bash
cargo run
```

You should see:

```
🚀 Server running on http://localhost:3000
```

Visit `http://localhost:3000` in your browser or use curl:

```bash
curl http://localhost:3000
```

You should get:

```json
{ "message": "Hello, Ripress!" }
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

### Port Already in Use

If port 3000 is already in use, change it in your code:

```rust
app.listen(8080, || {
    println!("🚀 Server running on http://localhost:8080");
})
.await;
```

## Next Steps

Now that you have Ripress installed, you can:

- Follow the [Getting Started Guide](getting-started.md) for a comprehensive tutorial
- Check out the [Examples](examples/) for common use cases
- Explore the [API Reference](api-reference/) for detailed documentation

## Development Setup

For development, you might want to install additional tools:

```bash
# For automatic reloading during development
cargo install cargo-watch

# Run your server with auto-reload
cargo watch -x run
```

This will automatically restart your server when you make changes to your code.
