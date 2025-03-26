use actix::AsyncContext;
use actix::{Actor, ActorContext, StreamHandler};
use actix_web::web::Bytes;
use actix_web_actors::ws;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
    on_message_cl: Arc<Mutex<dyn FnMut(&str) + Send + Sync>>,
    on_disconnect_cl: Arc<dyn Fn(WebSocketConn, Vec<&WebSocketConn>) + Send + Sync>,
    pub(crate) on_connect_cl: Arc<dyn Fn(WebSocketConn, Vec<WebSocketConn>) + Send + Sync>,
    on_binary_cl: Arc<dyn Fn(Bytes) + Send + Sync>,
    pub(crate) path: String,
    send_text: String,
    clients: Vec<WebSocketConn>,
}

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut ws = WebSocketConn::new();
        ws.is_open = ctx.address().connected();
        ws.type_id = std::any::TypeId::of::<Self>();
        let mut clients = self.clients.clone();
        clients.push(ws.clone());
        (self.on_connect_cl)(ws, clients.clone());
        self.clients = clients;
        self.hb(ctx);
    }
}

impl Default for WebSocket {
    fn default() -> Self {
        Self {
            hb: Instant::now(),
            on_message_cl: Arc::new(Mutex::new(|_: &str| {})),
            on_binary_cl: Arc::new(|_| {}),
            on_disconnect_cl: Arc::new(|_, _| {}),
            on_connect_cl: Arc::new(|_, _| {}),
            path: String::new(),
            send_text: String::new(),
            clients: Vec::new(),
        }
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if !self.send_text.is_empty() {
            ctx.text(&*self.send_text);
            self.send_text.clear();
        }
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                if let Ok(mut guard) = self.on_message_cl.lock() {
                    guard(text.to_string().as_str());
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                (self.on_binary_cl)(bin.clone());
                ctx.binary(bin);
            }
            Ok(ws::Message::Close(reason)) => {
                let type_id = std::any::TypeId::of::<Self>();
                ctx.close(reason);
                let ws = self.clients.iter().find(|f| f.type_id == type_id);
                let clients = self.clients.iter().filter(|f| f.type_id != type_id);
                (self.on_disconnect_cl)(ws.unwrap().clone(), clients.collect());
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
            on_message_cl: Arc::new(Mutex::new(|_: &str| {})),
            on_connect_cl: Arc::new(|_, _| {}),
            on_disconnect_cl: Arc::new(|_, _| {}),
            on_binary_cl: Arc::new(|_| {}),
            path: path.to_string(),
            send_text: String::new(),
            clients: Vec::new(),
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
        F: FnMut(&str) + Send + Sync + 'static,
    {
        self.on_message_cl = Arc::new(Mutex::new(cl));
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
    /// ws.on_disconnect(|ws, clients| {
    ///     println!("Client disconnected");
    /// });
    /// ```

    pub fn on_disconnect<F>(&mut self, cl: F)
    where
        F: Fn(WebSocketConn, Vec<&WebSocketConn>) + Send + Sync + 'static,
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

    /// Sends a text message to the connected client
    ///
    /// ## Arguments
    ///
    /// * `text` - The text message to send
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::websocket::WebSocket;
    ///
    /// let mut ws = WebSocket::new("/ws");
    /// ws.send("Hello client!");
    /// ```
    ///
    pub fn send(&mut self, text: &str) {
        self.send_text = text.into();
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
    /// ws.on_connect(|ws, clients| {
    ///     println!("Client connected");
    /// });
    /// ```

    pub fn on_connect<F>(&mut self, cl: F)
    where
        F: Fn(WebSocketConn, Vec<WebSocketConn>) + Send + Sync + 'static,
    {
        self.on_connect_cl = Arc::new(cl);
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        let disconnect_cl = self.on_disconnect_cl.clone();
        ctx.run_interval(HEARTBEAT_INTERVAL, move |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                let type_id = std::any::TypeId::of::<Self>();
                let ws = act.clients.iter().find(|f| f.type_id == type_id);
                let clients = act.clients.iter().filter(|f| f.type_id != type_id);
                (disconnect_cl)(ws.unwrap().clone(), clients.collect());
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

#[derive(Clone, Debug)]
pub struct WebSocketConn {
    pub id: String,
    pub is_open: bool,
    pub metadata: HashMap<String, serde_json::Value>,
    type_id: TypeId,
}

impl WebSocketConn {
    fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            is_open: false,
            metadata: HashMap::new(),
            type_id: TypeId::of::<Self>(),
        }
    }
}
