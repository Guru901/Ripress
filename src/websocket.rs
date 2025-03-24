use actix::AsyncContext;
use actix::{Actor, ActorContext, StreamHandler};
use actix_web::web::Bytes;
use actix_web_actors::ws;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::app::App;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// WebSocket handler that manages real-time bidirectional communication
///
/// Handles text messages, binary data, connection events, and maintains
/// heartbeat to detect disconnected clients.

#[derive(Clone)]
pub struct WebSocket {
    pub(crate) hb: Instant,
    on_message_cl: Arc<dyn Fn(&str) + Send + Sync>,
    on_disconnect_cl: Arc<dyn Fn() + Send + Sync>,
    pub(crate) on_connect_cl: Arc<dyn Fn() + Send + Sync>,
    on_binary_cl: Arc<dyn Fn(Bytes) + Send + Sync>,
    pub(crate) path: String,
}

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl Default for WebSocket {
    fn default() -> Self {
        Self {
            hb: Instant::now(),
            on_message_cl: Arc::new(|_| {}),
            on_binary_cl: Arc::new(|_| {}),
            on_disconnect_cl: Arc::new(|| {}),
            on_connect_cl: Arc::new(|| {}),
            path: String::new(),
        }
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                (self.on_message_cl)(text.to_string().as_str());
                ctx.text(format!("Echo: {}", text));
            }
            Ok(ws::Message::Binary(bin)) => {
                (self.on_binary_cl)(bin.clone());
                ctx.binary(bin);
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                (self.on_disconnect_cl)();
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl WebSocket {
    /// Creates a new WebSocket instance.
    ///
    /// ## Arguments
    ///
    /// * `path` - The endpoint path where the WebSocket will listen
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::websocket::WebSocket;
    ///
    /// let ws = WebSocket::new("/ws");
    /// ```
    pub fn new(path: &str) -> Self {
        let ws = Self {
            hb: Instant::now(),
            on_message_cl: Arc::new(|_| {}),
            on_connect_cl: Arc::new(|| {}),
            on_disconnect_cl: Arc::new(|| {}),
            on_binary_cl: Arc::new(|_| {}),
            path: path.to_string(),
        };
        ws
    }

    /// Registers the WebSocket handler with the application
    ///
    /// ## Arguments
    ///
    /// * `app` - Mutable reference to the App instance
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::{app::App, websocket::WebSocket};
    ///
    /// let mut app = App::new();
    /// let ws = WebSocket::new("/ws");
    /// ws.register(&mut app);
    /// ```

    pub fn register(&self, app: &mut App) {
        app.ws_path = self.path.clone();
        app.ws = self.clone();
        app.has_ws = true;
    }

    /// Sets the callback for handling text messages
    ///
    /// ## Arguments
    ///
    /// * `cl` - Closure that takes a string slice parameter
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::websocket::WebSocket;
    ///
    /// let mut ws = WebSocket::new("/ws");
    /// ws.on_text(|msg| {
    ///     println!("Received message: {}", msg);
    /// });
    /// ```

    pub fn on_text<F>(&mut self, cl: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_message_cl = Arc::new(cl);
    }

    /// Sets the callback for handling disconnection events
    ///
    /// ## Arguments
    ///
    /// * `cl` - Closure that takes no parameters
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::websocket::WebSocket;
    ///
    /// let mut ws = WebSocket::new("/ws");
    /// ws.on_disconnect(|| {
    ///     println!("Client disconnected");
    /// });
    /// ```

    pub fn on_disconnect<F>(&mut self, cl: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_disconnect_cl = Arc::new(cl);
    }

    /// Sets the callback for handling binary messages
    ///
    /// ## Arguments
    ///
    /// * `cl` - Closure that takes a Bytes parameter
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::websocket::WebSocket;
    ///
    /// let mut ws = WebSocket::new("/ws");
    /// ws.on_binary(|data| {
    ///     println!("Received binary data: {:?}", data);
    /// });
    /// ```

    pub fn on_binary<F>(&mut self, cl: F)
    where
        F: Fn(Bytes) + Send + Sync + 'static,
    {
        self.on_binary_cl = Arc::new(cl);
    }

    /// Sets the callback for handling connection events
    ///
    /// ## Arguments
    ///
    /// * `cl` - Closure that takes no parameters
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::websocket::WebSocket;
    ///
    /// let mut ws = WebSocket::new("/ws");
    /// ws.on_connect(|| {
    ///     println!("Client connected");
    /// });
    /// ```

    pub fn on_connect<F>(&mut self, cl: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_connect_cl = Arc::new(cl);
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        let disconnect_cl = self.on_disconnect_cl.clone();
        ctx.run_interval(HEARTBEAT_INTERVAL, move |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                (disconnect_cl)();
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}
