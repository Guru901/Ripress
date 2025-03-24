# WebSocket Handler (WebSocket)

## Overview

The `WebSocket` struct provides real-time bidirectional communication capabilities in Ripress. It handles text messages, binary data, connection events, and maintains connection health through heartbeat mechanisms.

NOTE - WebSocket support is currently only limited to one endpoint per application. This will be fixed in the next release.

## Creating a WebSocket Handler

Creates a new WebSocket instance:

```rust
use ripress::websocket::WebSocket;

let ws = WebSocket::new("/ws");
```

## Sending Messages

Sends a text message to the connected WebSocket client:

```rust
ws.send("Hello client!", None);
```

With a context (inside message handler):

```rust
ws.on_text(|msg| {
    // Respond immediately to a received message
    ws.send("Got your message!");
});
```

## Registering with App

The WebSocket handler must be registered with your App instance:

```rust
use ripress::{app::App, websocket::WebSocket};

let mut app = App::new();
let ws = WebSocket::new("/ws");
ws.register(&mut app);
```

## Event Handlers

### Text Message Handler

Handle incoming text messages:

```rust
let mut ws = WebSocket::new("/ws");
ws.on_text(|msg| {
    println!("Received message: {}", msg);
});
```

### Binary Message Handler

Handle incoming binary data:

```rust
ws.on_binary(|data| {
    println!("Received binary data: {:?}", data);
});
```

### Connection Events

Handle client connect/disconnect events:

```rust
// Connection handler
ws.on_connect(|| {
    println!("New client connected!");
});

// Disconnection handler
ws.on_disconnect(|| {
    println!("Client disconnected");
});
```

## Complete Example

Here's a complete example showing how to set up a WebSocket server:

```rust
use ripress::{app::App, websocket::WebSocket};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    let ws = WebSocket::new("/ws");

    // Register WebSocket handlers
    ws.register(&mut app);

    // Set up event handlers
    app.ws.on_text(|msg| {
        println!("Received: {}", msg);
    });

    app.ws.on_connect(|| {
        println!("Client connected");
    });

    app.ws.on_disconnect(|| {
        println!("Client disconnected");
    });

    // Start the server
    app.listen(3000, || {
        println!("WebSocket server running on ws://localhost:3000/ws");
    })
    .await;
}
```

## Technical Details

### Heartbeat Mechanism

The WebSocket implementation includes an automatic heartbeat mechanism to maintain connection health:

- Heartbeat Interval: 5 seconds
- Client Timeout: 10 seconds

If a client doesn't respond to ping messages within the timeout period, it will be automatically disconnected.

### Message Types

The WebSocket handler supports these message types:

- Text messages (UTF-8 encoded strings)
- Binary messages (raw bytes)
- Ping/Pong messages (automatic handling)
- Close messages (triggers disconnect handler)

### Thread Safety

All event handlers are wrapped in `Arc` (Atomic Reference Counting) and must implement `Send + Sync` traits, making them safe to use across threads.

## Best Practices

1. **Register Early**: Register the WebSocket handler before setting up event handlers
2. **Error Handling**: Implement proper error handling in your callbacks
3. **Resource Cleanup**: Use the disconnect handler to clean up any resources
4. **Message Validation**: Validate incoming messages in your text/binary handlers

## Limitations

- Only one WebSocket endpoint per application
- Callbacks cannot be changed after the server starts
- No built-in message broadcasting (implement manually if needed)
