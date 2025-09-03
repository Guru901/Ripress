# WebSocket Examples

This directory contains examples demonstrating WebSocket functionality in Ripress.

## Basic WebSocket Echo Server

A simple WebSocket server that echoes back any message sent to it.

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

## Real-time Chat Application

A complete chat application with message broadcasting to all connected clients.

```rust
use ripress::{app::App, types::RouterFns};
use wynd::wynd::Wynd;
use std::sync::Arc;
use tokio::sync::broadcast;

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
            <head><title>WebSocket Chat</title></head>
            <body>
                <h1>WebSocket Chat</h1>
                <div id="messages"></div>
                <input type="text" id="message" placeholder="Type a message...">
                <button onclick="sendMessage()">Send</button>
                <script>
                    const ws = new WebSocket('ws://localhost:3000/ws');
                    const messages = document.getElementById('messages');
                    const input = document.getElementById('message');

                    ws.onmessage = function(event) {
                        const div = document.createElement('div');
                        div.textContent = event.data;
                        messages.appendChild(div);
                        messages.scrollTop = messages.scrollHeight;
                    };

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

## Testing WebSocket Connections

You can test these examples using various WebSocket clients:

```bash
# Using wscat (install with: npm install -g wscat)
wscat -c ws://localhost:3000/ws

# Using websocat (install with: cargo install websocat)
websocat ws://localhost:3000/ws
```

## Dependencies

Make sure you have the required dependencies in your `Cargo.toml`:

```toml
[dependencies]
ripress = { version = "1", features = ["with-wynd"] }  # Enable WebSocket support
wynd = "0.4"  # WebSocket library
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## More Examples

For more comprehensive examples, see:

- [WebSocket Guide](./websocket.md) - Complete guide with best practices
- [API Reference](./api-reference/app.md) - Detailed API documentation
- [Tutorial](./tutorial/your-first-app.md) - Step-by-step tutorial with WebSocket integration
