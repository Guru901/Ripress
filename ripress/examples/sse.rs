//! Server-Sent Events (SSE) Example
//!
//! Demonstrates real-time streaming with Server-Sent Events:
//! - Continuous event streams
//! - Periodic updates
//! - Named events
//! - Real-time data pushing

use bytes::Bytes;
use futures::{StreamExt, stream};
use ripress::req::HttpRequest;
use ripress::{app::App, res::HttpResponse, types::RouterFns};
use serde_json::json;
use std::time::{Duration, SystemTime};
use std::time::UNIX_EPOCH;
use tokio::time::interval;

fn now_hms() -> String {
    // Get current time since UNIX_EPOCH and format as HH:MM:SS using std
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0));
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

// Simple LCG random number generator (for demo only, not secure)
struct LcgRng(u64);
impl LcgRng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    // outputs a number uniformly in 0..max
    fn next_u32(&mut self, max: u32) -> u32 {
        // LCG constants from Numerical Recipes
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        ((self.0 >> 16) as u32) % max
    }
    // outputs f64 in 0..1
    fn next_f64(&mut self) -> f64 {
        let n = self.next_u32(u32::MAX);
        n as f64 / u32::MAX as f64
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // Serve HTML page with SSE client
    app.get("/", |_: HttpRequest, res: HttpResponse| async move {
        let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>SSE Example</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        .event-box { background: #f5f5f5; padding: 15px; border-radius: 8px; margin: 10px 0; min-height: 200px; }
        .event { background: white; padding: 10px; margin: 5px 0; border-left: 3px solid #4CAF50; }
        button { background: #4CAF50; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; margin: 5px; }
        button:hover { background: #45a049; }
        .status { padding: 10px; background: #e3f2fd; border-radius: 4px; margin: 10px 0; }
    </style>
</head>
<body>
    <h1>📡 Server-Sent Events Demo</h1>
    
    <div class="status" id="status">Status: Disconnected</div>
    
    <button onclick="connectTime()">Connect Time Stream</button>
    <button onclick="connectCounter()">Connect Counter</button>
    <button onclick="connectRandom()">Connect Random Numbers</button>
    <button onclick="disconnect()">Disconnect All</button>
    
    <h2>Events:</h2>
    <div class="event-box" id="events"></div>
    
    <script>
        let eventSource = null;
        
        function addEvent(message) {
            const div = document.createElement('div');
            div.className = 'event';
            div.textContent = new Date().toLocaleTimeString() + ' - ' + message;
            const container = document.getElementById('events');
            container.insertBefore(div, container.firstChild);
            
            // Keep only last 20 events
            while (container.children.length > 20) {
                container.removeChild(container.lastChild);
            }
        }
        
        function updateStatus(status) {
            document.getElementById('status').textContent = 'Status: ' + status;
        }
        
        function connectTime() {
            disconnect();
            eventSource = new EventSource('/sse/time');
            setupEventSource();
            updateStatus('Connected to Time Stream');
        }
        
        function connectCounter() {
            disconnect();
            eventSource = new EventSource('/sse/counter');
            setupEventSource();
            updateStatus('Connected to Counter');
        }
        
        function connectRandom() {
            disconnect();
            eventSource = new EventSource('/sse/random');
            setupEventSource();
            updateStatus('Connected to Random Numbers');
        }
        
        function setupEventSource() {
            eventSource.onmessage = function(e) {
                addEvent(e.data);
            };
            
            eventSource.onerror = function(e) {
                addEvent('Error or connection closed');
                updateStatus('Disconnected');
            };
        }
        
        function disconnect() {
            if (eventSource) {
                eventSource.close();
                eventSource = null;
                updateStatus('Disconnected');
            }
        }
    </script>
</body>
</html>
        "#;
        
        res.html(html)
    });

    // SSE endpoint - Current time every second
    app.get("/sse/time", |_: HttpRequest, res: HttpResponse| async move {
        let stream = stream::unfold(interval(Duration::from_secs(1)), |mut timer| async move {
            timer.tick().await;
            let time = now_hms();
            let data = format!("data: Current time: {}\n\n", time);
            Some((Ok::<Bytes, std::io::Error>(Bytes::from(data)), timer))
        });

        res.set_header("content-type", "text/event-stream")
            .set_header("cache-control", "no-cache")
            .set_header("connection", "keep-alive")
            .write(stream)
    });

    // SSE endpoint - Counter
    app.get("/sse/counter", |_: HttpRequest, res: HttpResponse| async move {
        let stream = stream::unfold((interval(Duration::from_millis(500)), 0), |(mut timer, count)| async move {
            timer.tick().await;
            let data = format!("data: Count: {}\n\n", count);
            Some((Ok::<Bytes, std::io::Error>(Bytes::from(data)), (timer, count + 1)))
        }).take(50); // Stop after 50 events

        res.set_header("content-type", "text/event-stream")
            .set_header("cache-control", "no-cache")
            .set_header("connection", "keep-alive")
            .write(stream)
    });

    // SSE endpoint - Random numbers
    app.get("/sse/random", |_: HttpRequest, res: HttpResponse| async move {
        let rng = LcgRng::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(42))
                .as_secs(),
        );
        let stream = stream::unfold((interval(Duration::from_millis(800)), rng), |(mut timer, mut rng)| async move {
            timer.tick().await;
            let random = rng.next_u32(100);
            let data = format!("data: Random number: {}\n\n", random);
            Some((Ok::<Bytes, std::io::Error>(Bytes::from(data)), (timer, rng)))
        });

        res.set_header("content-type", "text/event-stream")
            .set_header("cache-control", "no-cache")
            .set_header("connection", "keep-alive")
            .write(stream)
    });

    // SSE endpoint - Named events
    app.get("/sse/events", |_: HttpRequest, res: HttpResponse| async move {
        let stream = stream::unfold((interval(Duration::from_secs(2)), 0), |(mut timer, count)| async move {
            timer.tick().await;
            
            let event_type = match count % 3 {
                0 => "message",
                1 => "alert",
                _ => "notification",
            };
            
            let data = format!(
                "event: {}\ndata: {}\n\n",
                event_type,
                json!({"type": event_type, "count": count})
            );
            
            Some((Ok::<Bytes, std::io::Error>(Bytes::from(data)), (timer, count + 1)))
        });

        res.set_header("content-type", "text/event-stream")
            .set_header("cache-control", "no-cache")
            .set_header("connection", "keep-alive")
            .write(stream)
    });

    // SSE endpoint - Stock prices simulation
    app.get("/sse/stocks", |_: HttpRequest, res: HttpResponse| async move {
        let rng = LcgRng::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(31415))
                .as_secs(),
        );
        let stream = stream::unfold((interval(Duration::from_secs(1)), 100.0, rng), |(mut timer, mut price, mut rng)| async move {
            timer.tick().await;

            // Simulate price changes
            let change = (rng.next_f64() - 0.5) * 5.0;
            price += change;
            if price < 50.0 { price = 50.0; }
            if price > 150.0 { price = 150.0; }
            
            let data = format!(
                "data: {}\n\n",
                json!({
                    "symbol": "RUST",
                    "price": format!("{:.2}", price),
                    "change": format!("{:+.2}", change)
                })
            );
            
            Some((Ok::<Bytes, std::io::Error>(Bytes::from(data)), (timer, price, rng)))
        });

        res.set_header("content-type", "text/event-stream")
            .set_header("cache-control", "no-cache")
            .set_header("connection", "keep-alive")
            .set_header("access-control-allow-origin", "*")
            .write(stream)
    });

    println!("📡 SSE example server starting on http://127.0.0.1:3000");
    println!("\nOpen http://127.0.0.1:3000 in your browser to see real-time events\n");
    println!("Available SSE endpoints:");
    println!("  - /sse/time    : Current time every second");
    println!("  - /sse/counter : Counter from 0 to 50");
    println!("  - /sse/random  : Random numbers");
    println!("  - /sse/events  : Named events");
    println!("  - /sse/stocks  : Stock price simulation");

    app.listen(3000, || {}).await;

    Ok(())
}