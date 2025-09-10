# WebSocket Guide

## Overview

Ripress provides built-in WebSocket support through the `wynd` crate when the "with-wynd" feature is enabled. This allows you to create real-time applications alongside your HTTP routes, enabling features like live chat, real-time notifications, collaborative editing, and more.

## Prerequisites

### Dependencies

Ensure you have the WebSocket dependencies in your `Cargo.toml`:

```toml
[dependencies]
ripress = { version = "1", features = ["with-wynd"] }  # Enable WebSocket support
wynd = "0.4"  # WebSocket library
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

### Feature Control

You can explicitly control WebSocket support:

```toml
# Enable WebSocket support
ripress = { version = "1", features = ["with-wynd"] }

# Use without WebSocket support (default)
ripress = "1"
```

## Basic WebSocket Setup

### Simple Echo Server

Here's a minimal example that echoes back any message sent to it:

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let mut wynd = Wynd::new();

    // HTTP route
    app.get("/", |_, res| async move {
        res.ok().text("WebSocket Echo Server")
    });

    // WebSocket connection handler
    wynd.on_connection(|conn| async move {
        conn.on_text(|event, _| async move {
            println!("Received: {}", event.data);

            // Echo the message back
            if let Err(e) = event.handle.send_text(&format!("Echo: {}", event.data)).await {
                eprintln!("Failed to send echo: {}", e);
            }
        });
    });

    // Mount WebSocket at /ws path
    app.use_wynd("/ws", wynd.handler());

    app.listen(3000, || {
        println!("🚀 Server running on http://localhost:3000");
        println!("🔌 WebSocket available at ws://localhost:3000/ws");
    })
    .await;
}
```

### Testing the Echo Server

You can test this using various WebSocket clients:

```bash
# Using wscat (install with: npm install -g wscat)
wscat -c ws://localhost:3000/ws

# Using websocat (install with: cargo install websocat)
websocat ws://localhost:3000/ws
```

## WebSocket Event Handlers

The `wynd` crate provides several event handlers for different WebSocket events:

### Text Message Handler

Handle text-based messages from clients:

```rust
wynd.on_connection(|conn| async move {
    conn.on_text(|event, _| async move {
        println!("Text message: {}", event.data);

        // Process the message
        let response = format!("Processed: {}", event.data);

        // Send response back
        if let Err(e) = event.handle.send_text(&response).await {
            eprintln!("Failed to send response: {}", e);
        }
    });
});
```

### Binary Message Handler

Handle binary data (images, files, etc.):

```rust
wynd.on_connection(|conn| async move {
    conn.on_binary(|event, _| async move {
        println!("Binary message received, size: {} bytes", event.data.len());

        // Process binary data
        let response = format!("Received {} bytes of binary data", event.data.len());

        // Send text response back
        if let Err(e) = event.handle.send_text(&response).await {
            eprintln!("Failed to send response: {}", e);
        }

        // Or echo binary data back
        if let Err(e) = event.handle.send_binary(&event.data).await {
            eprintln!("Failed to send binary echo: {}", e);
        }
    });
});
```

### Connection Close Handler

Handle when clients disconnect:

```rust
wynd.on_connection(|conn| async move {
    conn.on_close(|event, _| async move {
        println!("Connection closed with code: {:?}", event.code);
        println!("Close reason: {:?}", event.reason);
    });
});
```

### Error Handler

Handle WebSocket errors:

```rust
wynd.on_connection(|conn| async move {
    conn.on_error(|event, _| async move {
        eprintln!("WebSocket error: {:?}", event.error);
    });
});
```

### Complete Event Handler Example

Here's an example with all event handlers:

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::Wynd;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let mut wynd = Wynd::new();

    app.get("/", |_, res| async move {
        res.ok().text("WebSocket Event Handler Demo")
    });

    wynd.on_connection(|conn| async move {
        println!("New WebSocket connection established");

        // Handle text messages
        conn.on_text(|event, _| async move {
            println!("Text message: {}", event.data);

            let response = match event.data.as_str() {
                "ping" => "pong".to_string(),
                "hello" => "Hello there!".to_string(),
                _ => format!("Echo: {}", event.data),
            };

            if let Err(e) = event.handle.send_text(&response).await {
                eprintln!("Failed to send response: {}", e);
            }
        });

        // Handle binary messages
        conn.on_binary(|event, _| async move {
            println!("Binary message: {} bytes", event.data.len());

            let response = format!("Received {} bytes of binary data", event.data.len());
            if let Err(e) = event.handle.send_text(&response).await {
                eprintln!("Failed to send response: {}", e);
            }
        });

        // Handle connection close
        conn.on_close(|event, _| async move {
            println!("Connection closed: {:?}", event.code);
        });

        // Handle errors
        conn.on_error(|event, _| async move {
            eprintln!("WebSocket error: {:?}", event.error);
        });
    });

    app.use_wynd("/ws", wynd.handler());

    app.listen(3000, || {
        println!("🚀 Server running on http://localhost:3000");
        println!("🔌 WebSocket available at ws://localhost:3000/ws");
    })
    .await;
}
```

## Advanced WebSocket Applications

### Real-time Chat Application

Here's a complete chat application with message broadcasting:

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::Wynd;
use std::sync::Arc;
use tokio::sync::broadcast;
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let mut wynd = Wynd::new();

    // Create a broadcast channel for chat messages
    let (tx, _rx) = broadcast::channel::<String>(100);
    let tx = Arc::new(tx);

    // HTTP route for the chat page
    app.get("/", |_, res| async move {
        res.ok().html(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>WebSocket Chat</title>
                <style>
                    body { font-family: Arial, sans-serif; margin: 20px; }
                    #messages { height: 400px; overflow-y: scroll; border: 1px solid #ccc; padding: 10px; margin-bottom: 10px; }
                    #message { width: 70%; padding: 5px; }
                    button { padding: 5px 15px; }
                </style>
            </head>
            <body>
                <h1>WebSocket Chat</h1>
                <div id="messages"></div>
                <input type="text" id="message" placeholder="Type a message...">
                <button onclick="sendMessage()">Send</button>
                <script>
                    const ws = new WebSocket('ws://localhost:3000/ws');
                    const messages = document.getElementById('messages');
                    const input = document.getElementById('message');

                    ws.onopen = function() {
                        console.log('Connected to WebSocket');
                        addMessage('System: Connected to chat server');
                    };

                    ws.onmessage = function(event) {
                        addMessage(event.data);
                    };

                    ws.onclose = function() {
                        addMessage('System: Disconnected from chat server');
                    };

                    function addMessage(text) {
                        const div = document.createElement('div');
                        div.textContent = text;
                        messages.appendChild(div);
                        messages.scrollTop = messages.scrollHeight;
                    }

                    function sendMessage() {
                        if (input.value.trim()) {
                            ws.send(input.value);
                            input.value = '';
                        }
                    }

                    input.addEventListener('keypress', function(e) {
                        if (e.key === 'Enter') sendMessage();
                    });
                </script>
            </body>
            </html>
        "#)
    });

    // WebSocket connection handler with chat functionality
    wynd.on_connection(|conn| async move {
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        println!("New chat participant joined");

        // Handle incoming text messages
        conn.on_text(|event, _| async move {
            let message = format!("User: {}", event.data);
            println!("{}", message);

            // Broadcast message to all connected clients
            if let Err(e) = tx.send(message.clone()) {
                eprintln!("Failed to broadcast message: {}", e);
            }

            // Send confirmation back to sender
            if let Err(e) = event.handle.send_text("Message sent!").await {
                eprintln!("Failed to send confirmation: {}", e);
            }
        });

        // Handle connection close
        conn.on_close(|event, _| async move {
            println!("Chat participant left: {:?}", event.code);
        });

        // Handle errors
        conn.on_error(|event, _| async move {
            eprintln!("WebSocket error: {:?}", event.error);
        });
    });

    // Mount WebSocket at /ws path
    app.use_wynd("/ws", wynd.handler());

    app.listen(3000, || {
        println!("🚀 Chat server running on http://localhost:3000");
        println!("🔌 WebSocket available at ws://localhost:3000/ws");
    })
    .await;
}
```

### Real-time Notifications

Here's an example of a notification system:

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::Wynd;
use std::sync::Arc;
use tokio::sync::broadcast;
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let mut wynd = Wynd::new();

    // Create a broadcast channel for notifications
    let (tx, _rx) = broadcast::channel::<String>(100);
    let tx = Arc::new(tx);

    // HTTP route for the notification page
    app.get("/", |_, res| async move {
        res.ok().html(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Real-time Notifications</title>
                <style>
                    body { font-family: Arial, sans-serif; margin: 20px; }
                    #notifications { height: 300px; overflow-y: scroll; border: 1px solid #ccc; padding: 10px; }
                    .notification { margin: 5px 0; padding: 10px; background: #f0f0f0; border-radius: 5px; }
                    .urgent { background: #ffebee; border-left: 4px solid #f44336; }
                </style>
            </head>
            <body>
                <h1>Real-time Notifications</h1>
                <div id="notifications"></div>
                <button onclick="requestNotification()">Request Notification</button>
                <script>
                    const ws = new WebSocket('ws://localhost:3000/ws');
                    const notifications = document.getElementById('notifications');

                    ws.onmessage = function(event) {
                        const data = JSON.parse(event.data);
                        addNotification(data.message, data.urgent);
                    };

                    function addNotification(message, urgent) {
                        const div = document.createElement('div');
                        div.className = 'notification' + (urgent ? ' urgent' : '');
                        div.textContent = message;
                        notifications.appendChild(div);
                        notifications.scrollTop = notifications.scrollHeight;
                    }

                    function requestNotification() {
                        ws.send(JSON.stringify({type: 'request_notification'}));
                    }
                </script>
            </body>
            </html>
        "#)
    });

    // WebSocket connection handler
    wynd.on_connection(|conn| async move {
        let tx = tx.clone();

        println!("New notification subscriber connected");

        // Handle incoming messages
        conn.on_text(|event, _| async move {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&event.data) {
                if let Some(msg_type) = data.get("type").and_then(|v| v.as_str()) {
                    match msg_type {
                        "request_notification" => {
                            // Send a test notification
                            let notification = json!({
                                "message": "This is a test notification!",
                                "urgent": false
                            });

                            if let Err(e) = event.handle.send_text(&notification.to_string()).await {
                                eprintln!("Failed to send notification: {}", e);
                            }
                        }
                        _ => {
                            eprintln!("Unknown message type: {}", msg_type);
                        }
                    }
                }
            }
        });

        // Handle connection close
        conn.on_close(|event, _| async move {
            println!("Notification subscriber disconnected: {:?}", event.code);
        });
    });

    // Mount WebSocket at /ws path
    app.use_wynd("/ws", wynd.handler());

    // Start the server
    app.listen(3000, || {
        println!("🚀 Notification server running on http://localhost:3000");
        println!("🔌 WebSocket available at ws://localhost:3000/ws");
    })
    .await;
}
```

## WebSocket Best Practices

### 1. Error Handling

Always handle WebSocket errors gracefully:

```rust
wynd.on_connection(|conn| async move {
    conn.on_error(|event, _| async move {
        eprintln!("WebSocket error: {:?}", event.error);
        // Log the error, notify administrators, etc.
    });
});
```

### 2. Connection Management

Track active connections for cleanup:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

let connections = Arc::new(Mutex::new(HashMap::new()));

wynd.on_connection(|conn| async move {
    let connections = connections.clone();

    // Store connection
    let conn_id = uuid::Uuid::new_v4();
    connections.lock().await.insert(conn_id, conn.clone());

    conn.on_close(|event, _| async move {
        // Remove connection from tracking
        connections.lock().await.remove(&conn_id);
        println!("Connection {} closed", conn_id);
    });
});
```

### 3. Message Validation

Validate incoming messages before processing:

```rust
conn.on_text(|event, _| async move {
    // Validate message length
    if event.data.len() > 1000 {
        let error_msg = "Message too long (max 1000 characters)";
        if let Err(e) = event.handle.send_text(error_msg).await {
            eprintln!("Failed to send error: {}", e);
        }
        return;
    }

    // Validate message content
    if event.data.trim().is_empty() {
        return; // Ignore empty messages
    }

    // Process valid message
    println!("Valid message: {}", event.data);
});
```

### 4. Rate Limiting

Implement rate limiting for WebSocket messages:

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

let rate_limits = Arc::new(Mutex::new(HashMap::new()));

wynd.on_connection(|conn| async move {
    let rate_limits = rate_limits.clone();

    conn.on_text(|event, _| async move {
        let now = Instant::now();
        let mut limits = rate_limits.lock().await;

        // Simple rate limiting: max 10 messages per minute
        let client_id = "client"; // In real app, use actual client identifier

        if let Some(last_message) = limits.get(&client_id) {
            if now.duration_since(*last_message) < Duration::from_secs(6) {
                let error_msg = "Rate limit exceeded. Please wait before sending another message.";
                if let Err(e) = event.handle.send_text(error_msg).await {
                    eprintln!("Failed to send rate limit error: {}", e);
                }
                return;
            }
        }

        limits.insert(client_id.to_string(), now);

        // Process message
        println!("Message: {}", event.data);
    });
});
```

### 5. Security Considerations

- **Authentication**: Implement WebSocket authentication if needed
- **Input Validation**: Always validate and sanitize incoming messages
- **CORS**: Configure CORS properly for WebSocket connections
- **Rate Limiting**: Prevent abuse with rate limiting
- **Connection Limits**: Limit the number of concurrent connections

### 6. Graceful Shutdown

Handle connection closures properly:

```rust
use tokio::signal;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let mut wynd = Wynd::new();

    // Setup WebSocket handlers...

    // Handle graceful shutdown
    tokio::select! {
        _ = app.listen(3000, || {
            println!("Server running on http://localhost:3000");
        }) => {},
        _ = signal::ctrl_c() => {
            println!("Shutting down gracefully...");
            // Clean up connections, save state, etc.
        }
    }
}
```

## Testing WebSocket Connections

### Using wscat

```bash
# Install wscat
npm install -g wscat

# Connect to WebSocket
wscat -c ws://localhost:3000/ws

# Send messages
> Hello, WebSocket!
< Echo: Hello, WebSocket!
```

### Using websocat

```bash
# Install websocat
cargo install websocat

# Connect to WebSocket
websocat ws://localhost:3000/ws

# Send messages
Hello, WebSocket!
```

### Using curl (for WebSocket upgrade testing)

```bash
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" \
     -H "Sec-WebSocket-Version: 13" -H "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
     http://localhost:3000/ws
```

### Browser Testing

You can test WebSocket connections directly in the browser console:

```javascript
const ws = new WebSocket("ws://localhost:3000/ws");

ws.onopen = function () {
  console.log("Connected to WebSocket");
  ws.send("Hello from browser!");
};

ws.onmessage = function (event) {
  console.log("Received:", event.data);
};

ws.onclose = function () {
  console.log("Disconnected from WebSocket");
};

ws.onerror = function (error) {
  console.error("WebSocket error:", error);
};
```

## WebSocket with Middleware

WebSocket connections can coexist with HTTP middleware. The WebSocket middleware is applied before other middleware, ensuring proper WebSocket upgrade handling:

```rust
let mut app = App::new();
let mut wynd = Wynd::new();

// Add HTTP middleware
app.use_pre_middleware("/", |req, res| async {
    println!("HTTP request: {} {}", req.method, req.path);
    (req, None)
});

// Add authentication middleware
app.use_pre_middleware("/api/", |req, res| async {
    // Check authentication for API routes
    if req.headers.get("Authorization").is_none() {
        return (req, Some(res.unauthorized().text("Authentication required")));
    }
    (req, None)
});

// WebSocket setup (runs before other middleware)
wynd.on_connection(|conn| async move {
    conn.on_text(|event, _| async move {
        println!("WebSocket message: {}", event.data);
    });
});

app.use_wynd("/ws", wynd.handler());
```

## Troubleshooting

### Common Issues

1. **WebSocket connection fails**: Check if the "with-wynd" feature is enabled
2. **Messages not received**: Ensure event handlers are properly configured
3. **Connection drops**: Implement reconnection logic in client code
4. **Performance issues**: Consider connection pooling and message batching

### Debug Tips

1. **Enable logging**: Use `RUST_LOG=debug` to see detailed WebSocket logs
2. **Test with simple clients**: Use `wscat` or `websocat` for testing
3. **Check browser console**: Look for WebSocket errors in browser dev tools
4. **Monitor connections**: Track active connections and their states

---

This guide covers the essential aspects of WebSocket development with Ripress. For more advanced topics, refer to the [wynd crate documentation](https://docs.rs/wynd) and explore the examples in the Ripress repository.
