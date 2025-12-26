//! # App Module
//!
//! The core application module for Ripress, providing Express.js-like functionality
//! for building HTTP servers in Rust. This module contains the main [`App`] struct
//! and [`Middleware`] definitions that form the foundation of a Ripress web application.
//!
//! ## Key Features
//!
//! - Express.js-like routing and middleware system
//! - Built-in middleware for common tasks (CORS, logging, rate limiting, etc.)
//! - Static file serving capabilities
//! - WebSocket support (with `wynd` feature)
//! - Async/await support throughout
//!
//! ## Basic Usage
//!
//! ```no_run
//! use ripress::app::App;
//! use ripress::types::RouterFns;
//! use ripress::req::HttpRequest;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut app = App::new();
//!
//!     app.get("/", |_req: HttpRequest, res| async move {
//!         res.ok().text("Hello, World!")
//!     });
//!
//!     app.listen(3000, || {
//!         println!("Server running on http://localhost:3000");
//!     }).await;
//! }
//! ```

#![warn(missing_docs)]

use crate::app::api_error::ApiError;

use crate::helpers::{box_future_middleware, exec_post_middleware, exec_pre_middleware};
use crate::middlewares::body_limit::body_limit;
#[cfg(feature = "compression")]
use crate::middlewares::compression::{compression, CompressionConfig};
use crate::middlewares::cors::{cors, CorsConfig};
#[cfg(feature = "logger")]
use crate::middlewares::logger::{logger, LoggerConfig};
use crate::middlewares::rate_limiter::{rate_limiter, RateLimiterConfig};
use crate::middlewares::shield::{shield, ShieldConfig};
#[cfg(feature = "with-wynd")]
use crate::middlewares::WyndMiddleware;
use crate::middlewares::{Middleware, MiddlewareType};
use crate::req::HttpRequest;
use crate::res::HttpResponse;
use crate::router::Router;
#[cfg(feature = "with-wynd")]
use crate::types::WyndMiddlewareHandler;
use crate::types::{HandlerMiddleware, HttpMethods, RouteBuilder, RouterFns};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::http::StatusCode;
use hyper::server::conn::http1;
use hyper::server::conn::http2;
use hyper::service::Service;
use hyper::{header, Method};
use hyper::{Request, Response};
use hyper_staticfile::Static;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use routerify_ng::ext::RequestExt;
use routerify_ng::RouterService;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

pub(crate) mod api_error;

/// The App struct is the core of Ripress, providing a simple interface for creating HTTP servers and handling requests.
///
/// It follows an Express-like pattern for route handling and middleware management. The App struct
/// manages routes, middlewares, static file serving, and server lifecycle.
///
/// ## Features
///
/// - **Routing**: HTTP method-based routing (GET, POST, PUT, DELETE, etc.)
/// - **Middleware**: Pre and post-processing middleware with path-based matching
/// - **Static Files**: Serve static assets with proper headers and caching
/// - **WebSocket Support**: Optional WebSocket support via the `wynd` crate
/// - **Built-in Middleware**: CORS, logging, rate limiting, compression, and security headers
///
/// ## Example
///
/// ```ignore
/// use ripress::app::App;
/// use ripress::types::RouterFns;
/// use ripress::req::HttpRequest;
///
/// #[tokio::main]
/// async fn main() {
///     let mut app = App::new();
///
///     // Add middleware
///     app.use_cors(None);
///     app.use_logger(None);
///
///     // Add routes
///     app.get("/", |_req: HttpRequest, res| async move {
///         res.ok().text("Hello, World!")
///     });
///
///     app.post("/api/users", |req: HttpRequest, res| async move {
///         // Handle user creation
///         res.ok().json("User created")
///     });
///
///     // Serve static files
///     app.static_files("/public", "./public").unwrap();
///
///     // Start server
///     app.listen(3000, || {
///         println!("Server running on http://localhost:3000");
///     }).await;
/// }
/// ```
pub struct App {
    /// The collection of registered routes organized by path and HTTP method.
    routes: Vec<Arc<RouteBuilder>>,

    pub(crate) host: String,
    /// Enables or disables HTTP/2 support for the server.
    ///
    /// When `true`, the underlying Hyper server is configured to accept HTTP/2
    /// connections (in addition to HTTP/1.1). HTTP/2 support is negotiated
    /// automatically by Hyper based on the incoming connection and does not
    /// require any changes to your route handlers or middleware.
    ///
    /// By default, HTTP/2 is **enabled**.
    pub(crate) http2: bool,
    /// Optional advanced configuration for HTTP/2 behavior.
    ///
    /// These settings allow fine-tuning HTTP/2 flow control, concurrency, and
    /// keep-alive behavior. When `None`, Hyper's defaults are used.
    pub(crate) http2_config: Option<Http2Config>,
    /// The list of middleware functions to be applied to requests.
    pub(crate) middlewares: Vec<Arc<Middleware>>,
    /// Static file mappings from mount path to filesystem path.
    pub(crate) static_files: HashMap<&'static str, &'static str>,

    pub(crate) graceful_shutdown: bool,
    #[cfg(feature = "with-wynd")]
    /// Optional WebSocket middleware (only available with `wynd` feature).
    pub(crate) wynd_middleware: Option<WyndMiddleware>,
}

/// Advanced configuration options for HTTP/2 behavior.
///
/// All fields are optional; if a field is `None`, Hyper's internal default for
/// that setting is used. Most applications can rely on the defaults and only
/// override `max_concurrent_streams` or timeouts for specific workloads.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Http2Config {
    /// If `true`, only HTTP/2 connections are accepted on this listener.
    /// If `false`, HTTP/1.1 and HTTP/2 are both supported (negotiated by Hyper).
    pub http2_only: bool,
    /// Maximum number of concurrent streams allowed per HTTP/2 connection.
    pub max_concurrent_streams: Option<u32>,
    /// Initial stream-level flow control window size.
    pub initial_stream_window_size: Option<u32>,
    /// Initial connection-level flow control window size.
    pub initial_connection_window_size: Option<u32>,
    /// Enable or disable Hyper's adaptive flow control window behavior.
    pub adaptive_window: Option<bool>,
    /// Maximum allowed HTTP/2 frame size in bytes.
    pub max_frame_size: Option<u32>,
    /// Maximum size of the header list (in octets) that is allowed.
    pub max_header_list_size: Option<u32>,
    /// Interval at which HTTP/2 PING frames are sent to keep the connection alive.
    pub keep_alive_interval: Option<Duration>,
    /// Timeout waiting for a PING ACK before considering the connection dead.
    pub keep_alive_timeout: Option<Duration>,
    /// Whether to send keep-alive PINGs even when the connection is idle.
    pub keep_alive_while_idle: Option<bool>,
}

impl RouterFns for App {
    fn routes(&mut self) -> &mut Vec<Arc<RouteBuilder>> {
        &mut self.routes
    }
}

impl App {
    /// Creates a new App instance with empty routes and middleware.
    ///
    /// This is the starting point for building a Ripress application. The returned
    /// App instance has no routes or middleware configured and is ready to be customized.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    ///
    /// let mut app = App::new();
    /// ```
    pub fn new() -> Self {
        App {
            routes: Vec::new(),
            http2: true,
            http2_config: None,
            middlewares: Vec::new(),
            static_files: HashMap::new(),
            graceful_shutdown: false,
            host: String::from("0.0.0.0"),
            #[cfg(feature = "with-wynd")]
            wynd_middleware: None,
        }
    }

    /// Sets the host address for the server to bind to.
    ///
    /// This method allows you to specify the network interface (host) that the Ripress server will listen on.
    /// By default, the server binds to `"0.0.0.0"` (all interfaces). You may want to bind to
    /// `"127.0.0.1"` (localhost only) or an external IP for remote access, depending on your deployment requirements.
    ///
    /// **Note:** If you use an empty string (`""`), the server may not bind properly. Use valid IPv4 or IPv6 addresses.
    ///
    /// # Arguments
    ///
    /// * `host` - The host address (e.g., `"127.0.0.1"`, `"0.0.0.0"`, or an IPv6 address like `"::1"`).
    ///
    /// # Examples
    ///
    /// ```
    /// use ripress::app::App;
    /// let app = App::new().host("127.0.0.1");
    /// ```
    pub fn host(&mut self, host: &str) -> &mut Self {
        self.host = host.to_string();
        self
    }

    /// Enables or disables HTTP/2 support for the application.
    ///
    /// By default, HTTP/2 is enabled so that compatible clients can negotiate
    /// HTTP/2 with the server transparently via Hyper. Disabling HTTP/2 forces
    /// all connections to use HTTP/1.1 only.
    ///
    /// This setting only affects the underlying protocol negotiation; your
    /// route handlers, middleware, and response APIs remain unchanged.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Set to `true` to enable HTTP/2, or `false` to disable it.
    ///
    /// # Examples
    ///
    /// ```
    /// use ripress::app::App;
    ///
    /// let mut app = App::new();
    ///
    /// // Disable HTTP/2 and serve only HTTP/1.1
    /// app.enable_http2(false);
    /// ```
    pub fn enable_http2(&mut self, enabled: bool) -> &mut Self {
        self.http2 = enabled;
        self
    }

    /// Applies advanced HTTP/2 configuration for the application.
    ///
    /// This method allows fine-tuning of HTTP/2 behavior such as maximum
    /// concurrent streams, flow-control windows, and keep-alive settings.
    /// All fields in [`Http2Config`] are optional; any `None` values will
    /// cause Hyper's defaults to be used for that setting.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use ripress::app::{App, Http2Config};
    ///
    /// let mut app = App::new();
    ///
    /// app.enable_http2(true)
    ///     .http2_config(Http2Config {
    ///         http2_only: false,
    ///         max_concurrent_streams: Some(100),
    ///         keep_alive_interval: Some(Duration::from_secs(30)),
    ///         keep_alive_timeout: Some(Duration::from_secs(10)),
    ///         ..Default::default()
    ///     });
    /// ```
    pub fn http2_config(&mut self, config: Http2Config) -> &mut Self {
        self.http2_config = Some(config);
        self
    }

    /// Adds a middleware to the application (deprecated).
    ///
    /// ## Deprecation Notice
    ///
    /// This method is deprecated since version 1.9.0. Use [`use_pre_middleware`] instead
    /// for better clarity about middleware execution order.
    ///
    /// ## Arguments
    ///
    /// * `path` - Optional path prefix where the middleware should apply. Defaults to "/" (all paths)
    /// * `middleware` - The middleware function to add
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    /// use ripress::req::HttpRequest;
    ///
    /// let mut app = App::new();
    ///
    /// // This is deprecated - use use_pre_middleware instead
    /// app.use_middleware(Some("/api"), |req: HttpRequest, res| async move {
    ///     println!("Processing API request");
    ///     (req, None)
    /// });
    /// ```
    #[deprecated(since = "1.9.0", note = "Use `use_pre_middleware` instead")]
    pub fn use_middleware<F, Fut, P>(&mut self, path: P, middleware: F) -> &mut Self
    where
        P: Into<Option<&'static str>>,
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static,
    {
        let path = path.into().unwrap_or("/").to_string();
        self.middlewares.push(Arc::new(Middleware {
            func: Self::middleware_from_closure(middleware),
            path,
            middleware_type: MiddlewareType::Pre,
        }));
        self
    }

    /// Enables graceful shutdown for the application.
    ///
    /// When graceful shutdown is enabled, the server will listen for a shutdown signal
    /// (such as Ctrl+C) and attempt to shut down cleanly, finishing any in-flight requests
    /// before exiting. This is useful for production environments where you want to avoid
    /// abruptly terminating active connections.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    ///
    /// let mut app = App::new();
    /// app.with_graceful_shutdown();
    /// ```
    pub fn with_graceful_shutdown(&mut self) {
        self.graceful_shutdown = true
    }

    /// Adds a pre-execution middleware to the application.
    ///
    /// Pre-middlewares are executed before the route handler. They can modify the request,
    /// short-circuit the processing by returning a response, or pass control to the next
    /// middleware in the chain.
    ///
    /// ## Arguments
    ///
    /// * `path` - Optional path prefix where the middleware should apply. If `None`, defaults to "/" (all paths)
    /// * `middleware` - The middleware function that receives `(HttpRequest, HttpResponse)` and returns a future
    ///   resolving to `(HttpRequest, Option<HttpResponse>)`. If `Some(response)` is returned, processing stops
    ///   and the response is sent. If `None` is returned, processing continues.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    /// use ripress::req::HttpRequest;
    ///
    /// let mut app = App::new();
    ///
    /// // Authentication middleware for API routes
    /// app.use_pre_middleware(Some("/api"), |req: HttpRequest, res| async move {
    ///     if req.headers.get("authorization").is_none() {
    ///         return (req, Some(res.unauthorized().text("Missing authorization header")));
    ///     }
    ///     (req, None) // Continue processing
    /// });
    ///
    /// // Logging middleware for all routes
    /// app.use_pre_middleware(None, |req: HttpRequest, res| async move {
    ///     println!("Request: {} {}", req.method, req.path);
    ///     (req, None)
    /// });
    /// ```
    pub fn use_pre_middleware<F, Fut, P>(&mut self, path: P, middleware: F) -> &mut Self
    where
        P: Into<Option<&'static str>>,
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static,
    {
        let path = path.into().unwrap_or("/").to_string();
        self.middlewares.push(Arc::new(Middleware {
            func: Self::middleware_from_closure(middleware),
            path: path,
            middleware_type: MiddlewareType::Pre,
        }));
        self
    }

    /// Adds multiple pre-execution middlewares to the application at once.
    ///
    /// Each middleware can be registered with an optional path prefix; if `None`, defaults to "/".
    /// Pre-middlewares run before the route handler and can modify the request, short-circuit with a response,
    /// or continue the processing chain.
    ///
    /// # Arguments
    ///
    /// * `middlewares` - A vector of tuples, where each tuple contains an optional path prefix and a middleware closure.
    ///
    /// # Example
    ///
    /// ```
    /// use ripress::{app::App, types::Middlewares, req::HttpRequest};
    ///
    /// let mut app = App::new();
    ///
    /// let pre: Middlewares = vec![
    ///     ("/", Box::new(|req: HttpRequest, res| Box::pin(async move { (req, None) }))),
    ///     ("/admin", Box::new(|req: HttpRequest, res| Box::pin(async move {
    ///         // admin check logic
    ///         (req, None)
    ///     }))),
    /// ];
    ///
    /// app.use_pre_middlewares(pre);
    /// ```
    pub fn use_pre_middlewares<F, Fut, P>(&mut self, middlewares: Vec<(P, F)>) -> &mut Self
    where
        P: Into<Option<&'static str>> + Copy,
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static,
    {
        for (path, middleware) in middlewares {
            self.use_pre_middleware(path, middleware);
        }
        self
    }

    /// Adds multiple post-execution middlewares to the application at once.
    ///
    /// Each middleware can be registered with an optional path prefix; if `None`, defaults to "/".
    /// Post-middlewares run after the route handler, can modify the response or perform cleanup.
    ///
    /// # Arguments
    ///
    /// * `middlewares` - A vector of tuples, where each tuple contains an optional path prefix and a middleware closure.
    ///
    /// # Example
    ///
    /// ```
    /// use ripress::{app::App, types::Middlewares, req::HttpRequest};
    ///
    /// let mut app = App::new();
    ///
    /// let post: Middlewares = vec![
    ///     ("/", Box::new(|req: HttpRequest, res| Box::pin(async move { (req, Some(res)) }))),
    ///     ("/api", Box::new(|req: HttpRequest, res| Box::pin(async move {
    ///         // response logging
    ///         (req, Some(res))
    ///     }))),
    /// ];
    ///
    /// app.use_post_middlewares(post);
    /// ```
    pub fn use_post_middlewares<F, Fut, P>(&mut self, middlewares: Vec<(P, F)>) -> &mut Self
    where
        P: Into<Option<&'static str>> + Copy,
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static,
    {
        for (path, middleware) in middlewares {
            self.use_post_middleware(path, middleware);
        }
        self
    }

    /// Adds a post-execution middleware to the application.
    ///
    /// Post-middlewares are executed after the route handler has processed the request.
    /// They can modify the response or perform cleanup operations. They cannot short-circuit
    /// processing since the route handler has already run.
    ///
    /// ## Arguments
    ///
    /// * `path` - Optional path prefix where the middleware should apply. If `None`, defaults to "/" (all paths)
    /// * `middleware` - The middleware function that receives `(HttpRequest, HttpResponse)` where the response
    ///   has been populated by the route handler. Returns a future resolving to `(HttpRequest, Option<HttpResponse>)`.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    /// use ripress::req::HttpRequest;
    ///
    /// let mut app = App::new();
    ///
    /// // Add security headers to all responses
    /// app.use_post_middleware(None, |req: HttpRequest, mut res| async move {
    ///     res = res.set_header("X-Frame-Options", "DENY")
    ///         .set_header("X-Content-Type-Options", "nosniff");
    ///     (req, Some(res))
    /// });
    ///
    /// // Log response status for API routes
    /// app.use_post_middleware(Some("/api"), |req: HttpRequest, res| async move {
    ///     println!("API Response: {}", req.path);
    ///     (req, Some(res))
    /// });
    /// ```
    pub fn use_post_middleware<F, Fut, P>(&mut self, path: P, middleware: F) -> &mut Self
    where
        P: Into<Option<&'static str>>,
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static,
    {
        let path = path.into().unwrap_or("/").to_string();
        self.middlewares.push(Arc::new(Middleware {
            func: Self::middleware_from_closure(middleware),
            path: path,
            middleware_type: MiddlewareType::Post,
        }));
        self
    }

    /// Adds a logger middleware to the application.
    ///
    /// The logger middleware logs incoming HTTP requests with configurable options.
    /// It uses the `tracing` crate for logging, so make sure to initialize a tracing
    /// subscriber in your application.
    ///
    /// ## Arguments
    ///
    /// * `config` - Optional [`LoggerConfig`] to customize logging behavior. If `None`,
    ///   default settings are used which log basic request information.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    /// use ripress::middlewares::logger::LoggerConfig;
    ///
    /// // Initialize tracing (required for logging to work)
    /// tracing_subscriber::fmt::init();
    ///
    /// let mut app = App::new();
    ///
    /// // Use default logger settings
    /// app.use_logger(None);
    ///
    /// // Use custom logger configuration
    /// app.use_logger(Some(LoggerConfig {
    ///     method: true,      // Log HTTP method
    ///     path: true,        // Log request path
    ///     status: true,      // Log response status
    ///     ..Default::default()
    /// }));
    /// ```
    ///
    /// ## Default Behavior
    ///
    /// - Logs to the `info` level
    /// - Includes HTTP method, path, and response status
    /// - Applied to all routes ("/")
    /// - Executed as post-middleware (after route handling)
    #[cfg(feature = "logger")]
    pub fn use_logger(&mut self, config: Option<LoggerConfig>) -> &mut Self {
        self.middlewares.push(Arc::new(Middleware {
            func: Self::middleware_from_closure(logger(config)),
            path: "/".to_string(),
            middleware_type: MiddlewareType::Post,
        }));
        self
    }

    /// Adds a CORS (Cross-Origin Resource Sharing) middleware to the application.
    ///
    /// CORS middleware handles cross-origin requests by setting appropriate headers
    /// and responding to preflight OPTIONS requests. This is essential for web applications
    /// that need to accept requests from different domains.
    ///
    /// ## Arguments
    ///
    /// * `config` - Optional [`CorsConfig`] to customize CORS behavior. If `None`,
    ///   permissive default settings are used.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    /// use ripress::middlewares::cors::CorsConfig;
    ///
    /// let mut app = App::new();
    ///
    /// // Use permissive default CORS settings (allows all origins)
    /// app.use_cors(None);
    ///
    /// // Use custom CORS configuration
    /// app.use_cors(Some(CorsConfig {
    ///     allowed_origin: "https://example.com",
    ///     allowed_methods: "GET, POST, PUT, DELETE, OPTIONS, HEAD",
    ///     allowed_headers: "Content-Type, Authorization",
    ///     ..Default::default()
    /// }));
    /// ```
    ///
    /// ## Default Behavior
    ///
    /// - Allows all origins (`*`)
    /// - Allows common HTTP methods
    /// - Applied to all routes ("/")
    /// - Executed as pre-middleware
    /// - Automatically handles OPTIONS preflight requests
    pub fn use_cors(&mut self, config: Option<CorsConfig>) -> &mut Self {
        self.middlewares.push(Arc::new(Middleware {
            func: Self::middleware_from_closure(cors(config)),
            path: "/".to_string(),
            middleware_type: MiddlewareType::Pre,
        }));
        self
    }

    /// Adds a request body size limit middleware to the application.
    ///
    /// This middleware enforces a maximum size limit on incoming request bodies to prevent
    /// memory exhaustion attacks and manage resource usage. Requests exceeding the limit
    /// are rejected with a 413 Payload Too Large status.
    ///
    /// ## Arguments
    ///
    /// * `config` - Optional maximum size in bytes for request bodies. If `None`,
    ///   the default limit is 1 MB (1,048,576 bytes).
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    ///
    /// let mut app = App::new();
    ///
    /// // Use the default 1 MB limit
    /// app.use_body_limit(None);
    ///
    /// // Set a custom limit (e.g., 2 MB for file uploads)
    /// app.use_body_limit(Some(2 * 1024 * 1024));
    ///
    /// // Very restrictive limit for API endpoints (100 KB)
    /// app.use_body_limit(Some(100 * 1024));
    /// ```
    ///
    /// ## Behavior
    ///
    /// - Applied to all routes ("/")
    /// - Executed as pre-middleware (before route processing)
    /// - Returns 413 Payload Too Large for requests exceeding the limit
    /// - Does not affect GET requests or requests without bodies
    pub fn use_body_limit(&mut self, config: Option<usize>) -> &mut Self {
        self.middlewares.push(Arc::new(Middleware {
            func: Self::middleware_from_closure(body_limit(config)),
            path: "/".to_string(),
            middleware_type: MiddlewareType::Pre,
        }));
        self
    }

    #[cfg(feature = "with-wynd")]
    /// Adds WebSocket middleware to the application using the Wynd WebSocket library.
    ///
    /// This method enables WebSocket support for your application by integrating with
    /// the Wynd WebSocket library. WebSocket connections will be handled at the specified path.
    ///
    /// ## Feature Requirement
    ///
    /// This method is only available when the `with-wynd` feature is enabled in your `Cargo.toml`:
    ///
    /// ```toml
    /// [dependencies]
    /// ripress = { version = "*", features = ["with-wynd"] }
    /// wynd = { version = "*", features = ["with-ripress"] }
    /// ```
    ///
    /// ## Arguments
    ///
    /// * `path` - The path where WebSocket connections should be accepted (e.g., "/ws", "/websocket")
    /// * `handler` - A Wynd WebSocket handler function that processes WebSocket connections
    ///
    /// ## Example
    ///
    /// ```ignore
    /// use ripress::{app::App, types::RouterFns};
    /// use wynd::wynd::Wynd;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut app = App::new();
    ///     let mut wynd = Wynd::new();
    ///
    ///     // Configure WebSocket event handlers
    ///     wynd.on_connection(|conn| async move {
    ///         println!("New WebSocket connection");
    ///
    ///         conn.on_text(|event, handle| async move {
    ///             println!("Received message: {}", event.data);
    ///             // Echo the message back
    ///             handle.send_text(event.data).await.ok();
    ///         });
    ///
    ///         conn.on_close(|_event| async move {
    ///             println!("WebSocket connection closed");
    ///         });
    ///     });
    ///
    ///     // Add regular HTTP routes
    ///     app.get("/", |_, res| async move {
    ///         res.ok().text("WebSocket server running")
    ///     });
    ///
    ///     // Add WebSocket support at /ws
    ///     app.use_wynd("/ws", wynd.handler());
    ///
    ///     app.listen(3000, || {
    ///         println!("Server with WebSocket support running on http://localhost:3000");
    ///         println!("WebSocket endpoint: ws://localhost:3000/ws");
    ///     }).await;
    /// }
    /// ```
    ///
    /// ## Client Connection
    ///
    /// Clients can connect to the WebSocket endpoint using:
    ///
    /// ```javascript
    /// const ws = new WebSocket('ws://localhost:3000/ws');
    /// ws.onmessage = (event) => console.log('Received:', event.data);
    /// ws.send('Hello WebSocket!');
    /// ```

    pub fn use_wynd<F, Fut>(&mut self, path: &'static str, handler: F) -> &mut Self
    where
        F: Fn(hyper::Request<Full<Bytes>>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = hyper::Result<hyper::Response<Full<hyper::body::Bytes>>>>
            + Send
            + 'static,
    {
        self.wynd_middleware = Some(WyndMiddleware {
            func: Self::wynd_middleware_from_closure(handler),
            path: path.to_string(),
        });
        self
    }

    /// Adds a rate limiting middleware to the application.
    ///
    /// Rate limiting helps protect your application from abuse by limiting the number
    /// of requests a client can make within a specified time window. Requests exceeding
    /// the limit are rejected with a 429 Too Many Requests status.
    ///
    /// ## Arguments
    ///
    /// * `config` - Optional [`RateLimiterConfig`] to customize rate limiting behavior.
    ///   If `None`, default settings are used.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use ripress::app::App;
    /// use ripress::middlewares::rate_limiter::RateLimiterConfig;
    /// use std::time::Duration;
    ///
    ///
    /// let mut app = App::new();
    ///
    /// // Use default rate limiting (typically 100 requests per minute)
    /// app.use_rate_limiter(None);
    ///
    /// // Custom rate limiting configuration
    /// app.use_rate_limiter(Some(RateLimiterConfig {
    ///     max_requests: 10,                    // Allow 10 requests
    ///     window_ms: Duration::from_secs(60),     // Per 60 seconds
    ///     message: "Rate limit exceeded".to_string(),
    ///     ..Default::default()
    /// }));
    /// ```
    ///
    /// ## Default Behavior
    ///
    /// - Applied to all routes ("/")
    /// - Executed as pre-middleware
    /// - Uses client IP address for rate limiting
    /// - Returns 429 Too Many Requests when limit is exceeded
    /// - Includes rate limit headers in responses
    ///
    /// ## Rate Limit Headers
    ///
    /// The middleware adds these headers to responses:
    /// - `X-RateLimit-Limit`: Maximum requests allowed
    /// - `X-RateLimit-Remaining`: Requests remaining in current window
    /// - `X-RateLimit-Reset`: Time when the rate limit window resets
    pub fn use_rate_limiter(&mut self, config: Option<RateLimiterConfig>) -> &mut Self {
        self.middlewares.push(Arc::new(Middleware {
            func: Self::middleware_from_closure(rate_limiter(config)),
            path: "/".to_string(),
            middleware_type: MiddlewareType::Pre,
        }));
        self
    }

    /// Adds a security middleware (shield) to the application.
    ///
    /// The shield middleware helps protect your application from common web vulnerabilities
    /// by setting various HTTP security headers and applying security best practices. This
    /// is essential for production applications.
    ///
    /// ## Arguments
    ///
    /// * `config` - Optional [`ShieldConfig`] to customize the shield middleware's behavior.
    ///   If `None`, secure default settings are applied.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    /// use ripress::middlewares::shield::{ShieldConfig, Hsts};
    ///
    /// let mut app = App::new();
    ///
    /// // Use default shield settings (recommended for most applications)
    /// app.use_shield(None);
    ///
    /// // Custom shield configuration
    /// app.use_shield(Some(ShieldConfig {
    ///     hsts: Hsts {
    ///         enabled: true,
    ///         max_age: 31536000,           // 1 year
    ///         include_subdomains: true,
    ///         preload: true,
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// }));
    /// ```
    ///
    /// ## Security Headers Applied
    ///
    /// The shield middleware can set the following security headers:
    ///
    /// - `Strict-Transport-Security`: Forces HTTPS connections
    /// - `X-Content-Type-Options`: Prevents MIME type sniffing
    /// - `X-Frame-Options`: Prevents clickjacking attacks
    /// - `X-XSS-Protection`: Enables cross-site scripting filtering
    /// - `Referrer-Policy`: Controls referrer information
    /// - `Content-Security-Policy`: Prevents various injection attacks
    ///
    /// ## Default Behavior
    ///
    /// - Applied to all routes ("/")
    /// - Executed as pre-middleware
    /// - Uses secure defaults suitable for most web applications
    /// - Can be customized per security requirements
    pub fn use_shield(&mut self, config: Option<ShieldConfig>) -> &mut Self {
        self.middlewares.push(Arc::new(Middleware {
            func: Self::middleware_from_closure(shield(config)),
            path: "/".to_string(),
            middleware_type: MiddlewareType::Pre,
        }));
        self
    }

    /// Adds a compression middleware to the application.
    ///
    /// Compression middleware automatically compresses response bodies using algorithms
    /// like gzip or deflate, reducing bandwidth usage and improving response times for
    /// clients that support compression.
    ///
    /// ## Arguments
    ///
    /// * `config` - Optional [`CompressionConfig`] to customize compression behavior.
    ///   If `None`, default settings are used with common compression algorithms enabled.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    /// use ripress::middlewares::compression::CompressionConfig;
    ///
    /// let mut app = App::new();
    ///
    /// // Use default compression settings (gzip, deflate)
    /// app.use_compression(None);
    ///
    /// // Custom compression configuration
    /// app.use_compression(Some(CompressionConfig {
    ///     level: 6,                        // Compression level (0-9)
    ///     threshold: 1024,                 // Minimum bytes to compress
    ///     ..Default::default()
    /// }));
    /// ```
    ///
    /// ## Default Behavior
    ///
    /// - Applied to all routes ("/")
    /// - Executed as post-middleware (after response generation)
    /// - Supports gzip and deflate compression
    /// - Automatically negotiates compression based on `Accept-Encoding` header
    /// - Only compresses responses above a minimum size threshold
    /// - Skips compression for already-compressed content types
    ///
    /// ## Content Type Handling
    ///
    /// By default, the middleware:
    /// - Compresses text-based content (HTML, CSS, JavaScript, JSON, XML)
    /// - Skips binary content that's already compressed (images, videos, archives)
    /// - Respects the client's `Accept-Encoding` header preferences
    /// - Adds appropriate `Content-Encoding` headers to compressed responses
    #[cfg(feature = "compression")]
    pub fn use_compression(&mut self, config: Option<CompressionConfig>) -> &mut Self {
        self.middlewares.push(Arc::new(Middleware {
            func: Self::middleware_from_closure(compression(config)),
            path: "/".to_string(),
            middleware_type: MiddlewareType::Post,
        }));
        self
    }

    /// Converts a closure into a middleware handler function.
    ///
    /// This is an internal helper method that wraps user-provided middleware functions
    /// into the expected format for the middleware system.
    fn middleware_from_closure<F, Fut>(f: F) -> HandlerMiddleware
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static,
    {
        Arc::new(move |req: HttpRequest, res| box_future_middleware(f(req, res)))
    }

    #[cfg(feature = "with-wynd")]
    /// Converts a WebSocket handler closure into a Wynd middleware handler.
    ///
    /// This is an internal helper method for the WebSocket functionality.
    fn wynd_middleware_from_closure<F, Fut>(f: F) -> WyndMiddlewareHandler
    where
        F: Fn(hyper::Request<Full<Bytes>>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = hyper::Result<hyper::Response<Full<hyper::body::Bytes>>>>
            + Send
            + 'static,
    {
        Arc::new(move |req| Box::pin(f(req)))
    }

    /// Mounts a [`Router`] at a specific base path, registering all of its routes onto the application.
    ///
    /// This method allows you to modularly organize and group routes using separate routers,
    /// then attach them to your application. Each route registered with the router will be
    /// prefixed by the router's base path. This is useful for API versioning, feature groupings,
    /// or splitting logic into modules. The router's routes are incorporated into the main
    /// application's route table, and will take precedence over static file handlers.
    ///
    /// # Example
    /// ```
    /// use ripress::{app::App, router::Router};
    /// use ripress::{req::HttpRequest, res::HttpResponse};
    /// use ripress::types::RouterFns;
    ///
    /// async fn v1_status(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().json(serde_json::json!({"status": "ok"}))
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut api_router = Router::new("/api/v1");
    ///     api_router.get("/status", v1_status);
    ///     
    ///     let mut app = App::new();
    ///     app.router(api_router);
    /// }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `router` - The [`Router`] instance whose routes will be registered onto this application.
    ///
    /// # Panics
    ///
    /// This method does not panic.
    pub fn router(&mut self, router: Router) {
        let routes = router.routes;
        let base_path = router.base_path;

        for route in routes {
            let path = &route.path;
            let method = &route.method;
            let handler = Arc::clone(&route.handler);

            if path == "/" {
                self.add_route(method, &base_path, move |req: HttpRequest, res| {
                    (handler)(req, res)
                });
            } else {
                let full_path = format!("{}{}", base_path, path);
                self.add_route(method, &full_path, move |req: HttpRequest, res| {
                    (handler)(req, res)
                });
            }
        }
    }

    /// Configures static file serving for the application.
    ///
    /// This method allows you to serve static assets (HTML, CSS, JavaScript, images, etc.)
    /// from the filesystem. Files are served with appropriate MIME types, caching headers,
    /// and ETag support for efficient client-side caching.
    ///
    /// ## Arguments
    ///
    /// * `path` - The URL path where static files should be mounted (e.g., "/public", "/static", "/")
    /// * `file` - The filesystem directory path containing the static files (e.g., "./public", "./dist")
    ///
    /// ## Returns
    ///
    /// * `Ok(())` - If the static file configuration was successful
    /// * `Err(&'static str)` - If there was a validation error with the provided paths
    ///
    /// ## Errors
    ///
    /// This method returns an error in the following cases:
    /// - `file` parameter is "/" (serving from filesystem root is blocked for security)
    /// - `path` parameter is empty
    /// - `file` parameter is empty
    /// - `path` parameter doesn't start with "/"
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    ///
    /// let mut app = App::new();
    ///
    /// // Serve files from ./public directory at /public URL path
    /// app.static_files("/public", "./public").unwrap();
    ///
    /// // Serve CSS and JS assets
    /// app.static_files("/assets", "./dist/assets").unwrap();
    ///
    /// // Serve a Single Page Application (SPA) from root
    /// // API routes take precedence, static files serve as fallback
    /// app.static_files("/", "./dist").unwrap();
    ///
    /// // Multiple static directories
    /// app.static_files("/images", "./uploads/images").unwrap();
    /// app.static_files("/docs", "./documentation").unwrap();
    /// ```
    ///
    /// ## Behavior
    ///
    /// - **Route Precedence**: API routes defined with `get()`, `post()`, etc. take precedence over static files
    /// - **Fallback Serving**: When mounted at "/", static files serve as fallback for unmatched routes
    /// - **MIME Types**: Automatically sets appropriate `Content-Type` headers based on file extensions
    /// - **Caching**: Includes `Cache-Control` and `ETag` headers for efficient browser caching
    /// - **Security**: Prevents directory traversal attacks and blocks serving from filesystem root
    ///
    /// ## File System Layout Example
    ///
    /// ```text
    /// project/
    /// ├── src/main.rs
    /// ├── public/           <- app.static_files("/public", "./public")
    /// │   ├── index.html    <- Accessible at /public/index.html
    /// │   ├── style.css     <- Accessible at /public/style.css
    /// │   └── script.js     <- Accessible at /public/script.js
    /// └── dist/             <- app.static_files("/", "./dist")
    ///     ├── index.html    <- Accessible at / (fallback)
    ///     └── favicon.ico   <- Accessible at /favicon.ico
    /// ```
    ///
    /// ## Security Considerations
    ///
    /// - Never use "/" as the `file` parameter - this is blocked for security reasons
    /// - Use specific directories like "./public" or "./assets"
    /// - The static file server prevents directory traversal (../) attacks automatically
    /// - Consider using a reverse proxy like nginx for serving static files in production
    pub fn static_files(
        &mut self,
        path: &'static str,
        file: &'static str,
    ) -> Result<(), &'static str> {
        // Validate inputs
        if file == "/" {
            return Err("Serving from filesystem root '/' is not allowed for security reasons");
        }
        if path.is_empty() {
            return Err("Mount path cannot be empty");
        }
        if file.is_empty() {
            return Err("File path cannot be empty");
        }
        // Require paths to start with '/'
        if !path.starts_with('/') {
            return Err("Mount path must start with '/'");
        }
        self.static_files.insert(path, file);
        Ok(())
    }

    /// Starts the HTTP server and begins listening for incoming requests.
    ///
    /// This method builds the complete router with all configured routes, middleware,
    /// and static file handlers, then starts the HTTP server on the specified port.
    /// The server runs indefinitely until the process is terminated.
    ///
    /// ## Arguments
    ///
    /// * `port` - The port number to listen on (e.g., 3000, 8080)
    /// * `cb` - A callback function that's executed once the server is ready to accept connections
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use ripress::app::App;
    /// use ripress::types::RouterFns;
    /// use ripress::req::HttpRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut app = App::new();
    ///
    ///     app.get("/", |_req: HttpRequest, res| async move {
    ///         res.ok().text("Hello, World!")
    ///     });
    ///
    ///     app.get("/health", |_req: HttpRequest, res| async move {
    ///         res.ok().json(serde_json::json!({"status": "healthy"}))
    ///     });
    ///
    ///     // Start server with startup message
    ///     app.listen(3000, || {
    ///         println!("🚀 Server running on http://localhost:3000");
    ///         println!("📊 Health check: http://localhost:3000/health");
    ///     }).await;
    /// }
    /// ```
    ///
    /// ## Server Initialization Order
    ///
    /// 1. **WebSocket Middleware**: Applied first (if `wynd` feature is enabled)
    /// 2. **Application Middleware**: Applied in registration order
    ///    - Pre-middleware (before route handlers)
    ///    - Post-middleware (after route handlers)
    /// 3. **API Routes**: Registered with exact path matching
    /// 4. **Static File Routes**: Registered as fallback handlers
    /// 5. **Error Handler**: Global error handling for the application
    ///
    /// ## Network Configuration
    ///
    /// - **Bind Address**: By default, binds to `0.0.0.0:port` (all interfaces); configurable via [`App::host`]
    /// - **Protocols**: HTTP/1.1 and HTTP/2 by default; HTTP/2 can be disabled via [`App::enable_http2`]
    /// - **Concurrent Connections**: Handled asynchronously with Tokio
    ///
    /// ## Error Handling
    ///
    /// If the server fails to start (e.g., port already in use), the error is printed
    /// to stderr and the process continues. You may want to handle this more gracefully:
    ///
    /// ```no_run
    /// # use ripress::app::App;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let app = App::new();
    /// // The server will print errors but won't panic
    /// app.listen(3000, || println!("Server starting...")).await;
    /// // This line is reached if server fails to start
    /// eprintln!("Server failed to start or has shut down");
    /// # }
    /// ```
    ///
    /// ## Production Considerations
    ///
    /// - Consider using environment variables for port configuration
    /// - Implement graceful shutdown handling
    /// - Use a process manager like systemd or PM2
    /// - Configure reverse proxy (nginx, Apache) for production
    /// - Enable logging middleware to monitor requests
    pub async fn listen<F: FnOnce()>(&self, port: u16, cb: F) {
        let mut router = routerify_ng::Router::<ApiError>::builder();

        #[cfg(feature = "with-wynd")]
        if let Some(middleware) = &self.wynd_middleware {
            router = router.middleware(routerify_ng::Middleware::pre({
                use crate::helpers::exec_wynd_middleware;

                let middleware = middleware.clone();
                move |req| exec_wynd_middleware(req, middleware.clone())
            }));
        }

        // Apply middlewares first
        for middleware in &self.middlewares {
            match middleware.middleware_type {
                MiddlewareType::Post => {
                    let middleware = Arc::clone(middleware);
                    router = router.middleware(routerify_ng::Middleware::post_with_info(
                        move |res, info| exec_post_middleware(res, Arc::clone(&middleware), info),
                    ));
                }
                _ => {
                    let middleware = Arc::clone(middleware);
                    router = router.middleware(routerify_ng::Middleware::pre(move |req| {
                        exec_pre_middleware(req, Arc::clone(&middleware))
                    }));
                }
            }
        }

        // Register API routes FIRST (before static files)
        // This ensures API routes take precedence over static file serving
        for route in &self.routes {
            let handler = Arc::clone(&route.handler);

            let method = match route.method {
                HttpMethods::GET => Method::GET,
                HttpMethods::POST => Method::POST,
                HttpMethods::PUT => Method::PUT,
                HttpMethods::DELETE => Method::DELETE,
                HttpMethods::PATCH => Method::PATCH,
                HttpMethods::HEAD => Method::HEAD,
                HttpMethods::OPTIONS => Method::OPTIONS,
            };

            router = router.add(&route.path, vec![method], move |mut req| {
                let handler = Arc::clone(&handler);

                async move {
                    let mut our_req = match HttpRequest::from_hyper_request(&mut req).await {
                        Ok(r) => r,
                        Err(e) => {
                            return Err(ApiError::Generic(
                                HttpResponse::new().bad_request().text(e.to_string()),
                            ));
                        }
                    };

                    req.params().iter().for_each(|(key, value)| {
                        our_req.set_param(key, value);
                    });

                    let response = handler(our_req, HttpResponse::new()).await;

                    let hyper_response = response.to_hyper_response().await;
                    // Infallible means this can never fail, so unwrap is safe
                    Ok(hyper_response.unwrap())
                }
            });
        }

        for (mount_path, serve_from) in self.static_files.iter() {
            let serve_from = (*serve_from).to_string();
            let mount_root = (*mount_path).to_string();

            let route_pattern_owned = if mount_root == "/" {
                "/*".to_string()
            } else {
                format!("{}/{}", mount_root, "*")
            };

            let serve_from_clone = serve_from.clone();
            let mount_root_clone = mount_root.clone();

            router = router.get(route_pattern_owned, move |req| {
                let serve_from = serve_from_clone.clone();
                let mount_root = mount_root_clone.clone();
                async move {
                    match Self::serve_static_with_headers(req, mount_root, serve_from).await {
                        Ok(res) => Ok(res),
                        Err(e) => Err(ApiError::Generic(
                            HttpResponse::new()
                                .internal_server_error()
                                .text(e.to_string()),
                        )),
                    }
                }
            });
        }

        router = router.err_handler(Self::error_handler);
        let router = router.build().unwrap();
        cb();

        let addr = format!("{}:{}", self.host, port)
            .parse::<SocketAddr>()
            .unwrap();

        let listener = TcpListener::bind(addr).await;

        if let Err(e) = listener {
            eprintln!("Error binding to address {}: {}", addr, e);
            return;
        }

        let listener = listener.unwrap();

        let router_service = Arc::new(RouterService::new(router).unwrap());

        let http2_enabled = self.http2;
        let http2_config = self.http2_config.clone();

        if self.graceful_shutdown {
            let mut shutdown = Box::pin(tokio::signal::ctrl_c());

            let http2_config = self.http2_config.clone();

            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _)) => {
                                let service = Arc::clone(&router_service);
                                let http2_enabled = http2_enabled;
                                let http2_config = http2_config.clone();

                                tokio::task::spawn(async move {
                                    // Now service is Arc<RouterService> and not moved
                                    let request_service = match service.call(&stream).await {
                                        Ok(svc) => svc,
                                        Err(err) => {
                                            eprintln!("Error creating per-connection service: {:?}", err);
                                            return;
                                        }
                                    };

                                    // Wrap the stream in TokioIo for hyper
                                    let io = TokioIo::new(stream);

                                    if http2_enabled {
                                        // HTTP/1.1 + HTTP/2 (negotiated by Hyper).
                                        // Enable HTTP/2 support with optional advanced tuning.
                                        if let Some(cfg) = http2_config {
                                            if cfg.http2_only {
                                                let mut h2 = http2::Builder::new(TokioExecutor::new());

                                                if let Some(v) = cfg.max_concurrent_streams {
                                                    h2.max_concurrent_streams(v);
                                                }
                                                if let Some(v) = cfg.initial_stream_window_size {
                                                    h2.initial_stream_window_size(v);
                                                }
                                                if let Some(v) = cfg.initial_connection_window_size {
                                                    h2.initial_connection_window_size(v);
                                                }
                                                if let Some(v) = cfg.adaptive_window {
                                                    h2.adaptive_window(v);
                                                }
                                                if let Some(v) = cfg.max_frame_size {
                                                    h2.max_frame_size(v);
                                                }
                                                if let Some(v) = cfg.max_header_list_size {
                                                    h2.max_header_list_size(v);
                                                }
                                                if let Some(v) = cfg.keep_alive_interval {
                                                    h2.keep_alive_interval(v);
                                                }
                                                if let Some(v) = cfg.keep_alive_timeout {
                                                    h2.keep_alive_timeout(v);
                                                }

                                                h2.enable_connect_protocol();

                                                let connection = h2.serve_connection(io, request_service);
                                                if let Err(err) = connection.await {
                                                    eprintln!("Error serving connection: {:?}", err);
                                                }
                                            } else {
                                                let mut builder = Builder::new(TokioExecutor::new());
                                                builder.http1().keep_alive(true);
                                                let mut h2 = builder.http2();
                                                if let Some(v) = cfg.max_concurrent_streams {
                                                    h2.max_concurrent_streams(v);
                                                }
                                                if let Some(v) = cfg.initial_stream_window_size {
                                                    h2.initial_stream_window_size(v);
                                                }
                                                if let Some(v) = cfg.initial_connection_window_size {
                                                    h2.initial_connection_window_size(v);
                                                }
                                                if let Some(v) = cfg.adaptive_window {
                                                    h2.adaptive_window(v);
                                                }
                                                if let Some(v) = cfg.max_frame_size {
                                                    h2.max_frame_size(v);
                                                }
                                                if let Some(v) = cfg.max_header_list_size {
                                                    h2.max_header_list_size(v);
                                                }
                                                if let Some(v) = cfg.keep_alive_interval {
                                                    h2.keep_alive_interval(v);
                                                }
                                                if let Some(v) = cfg.keep_alive_timeout {
                                                    h2.keep_alive_timeout(v);
                                                }
                                                h2.enable_connect_protocol();

                                                let connection =
                                                    builder.serve_connection_with_upgrades(io, request_service);
                                                if let Err(err) = connection.await {
                                                    eprintln!("Error serving connection: {:?}", err);
                                                }
                                            };

                                            // Note: hyper 1.x `Http2Builder` does not currently expose
                                            // an explicit `keep_alive_while_idle` toggle; this flag is
                                            // reserved for future use or more advanced wiring.
                                        } else {
                                            let mut builder = Builder::new(TokioExecutor::new());

                                            builder.http1().keep_alive(true);

                                            let connection =
                                                builder.serve_connection_with_upgrades(io, request_service);
                                            if let Err(err) = connection.await {
                                                eprintln!("Error serving connection: {:?}", err);
                                            }
                                        }
                                    } else {
                                        let mut builder = http1::Builder::new();
                                        builder.keep_alive(true);
                                        let connection = builder
                                            .serve_connection(io, request_service)
                                            .with_upgrades();

                                        if let Err(err) = connection.await {
                                            eprintln!("Error serving connection: {:?}", err);
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                eprintln!("Error accepting connection: {}", e);
                            }
                        }
                    }
                    _ = shutdown.as_mut() => {
                        break;
                    }
                }
            }
        } else {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let service = Arc::clone(&router_service);
                        let http2_enabled = http2_enabled;
                        let http2_config = http2_config.clone();

                        tokio::task::spawn(async move {
                            let request_service = match service.call(&stream).await {
                                Ok(svc) => svc,
                                Err(err) => {
                                    eprintln!("Error creating per-connection service: {:?}", err);
                                    return;
                                }
                            };

                            // Wrap the stream in TokioIo for hyper
                            let io = TokioIo::new(stream);

                            if http2_enabled {
                                // HTTP/1.1 + HTTP/2 (negotiated by Hyper).
                                // Enable HTTP/2 support with optional advanced tuning.
                                if let Some(cfg) = http2_config {
                                    if cfg.http2_only {
                                        let mut h2 = http2::Builder::new(TokioExecutor::new());

                                        if let Some(v) = cfg.max_concurrent_streams {
                                            h2.max_concurrent_streams(v);
                                        }
                                        if let Some(v) = cfg.initial_stream_window_size {
                                            h2.initial_stream_window_size(v);
                                        }
                                        if let Some(v) = cfg.initial_connection_window_size {
                                            h2.initial_connection_window_size(v);
                                        }
                                        if let Some(v) = cfg.adaptive_window {
                                            h2.adaptive_window(v);
                                        }
                                        if let Some(v) = cfg.max_frame_size {
                                            h2.max_frame_size(v);
                                        }
                                        if let Some(v) = cfg.max_header_list_size {
                                            h2.max_header_list_size(v);
                                        }
                                        if let Some(v) = cfg.keep_alive_interval {
                                            h2.keep_alive_interval(v);
                                        }
                                        if let Some(v) = cfg.keep_alive_timeout {
                                            h2.keep_alive_timeout(v);
                                        }

                                        h2.enable_connect_protocol();

                                        let connection = h2.serve_connection(io, request_service);
                                        if let Err(err) = connection.await {
                                            eprintln!("Error serving connection: {:?}", err);
                                        }
                                    } else {
                                        let mut builder = Builder::new(TokioExecutor::new());
                                        builder.http1().keep_alive(true);
                                        let mut h2 = builder.http2();
                                        if let Some(v) = cfg.max_concurrent_streams {
                                            h2.max_concurrent_streams(v);
                                        }
                                        if let Some(v) = cfg.initial_stream_window_size {
                                            h2.initial_stream_window_size(v);
                                        }
                                        if let Some(v) = cfg.initial_connection_window_size {
                                            h2.initial_connection_window_size(v);
                                        }
                                        if let Some(v) = cfg.adaptive_window {
                                            h2.adaptive_window(v);
                                        }
                                        if let Some(v) = cfg.max_frame_size {
                                            h2.max_frame_size(v);
                                        }
                                        if let Some(v) = cfg.max_header_list_size {
                                            h2.max_header_list_size(v);
                                        }
                                        if let Some(v) = cfg.keep_alive_interval {
                                            h2.keep_alive_interval(v);
                                        }
                                        if let Some(v) = cfg.keep_alive_timeout {
                                            h2.keep_alive_timeout(v);
                                        }
                                        h2.enable_connect_protocol();

                                        let connection = builder
                                            .serve_connection_with_upgrades(io, request_service);
                                        if let Err(err) = connection.await {
                                            eprintln!("Error serving connection: {:?}", err);
                                        }
                                    };

                                    // Note: hyper 1.x `Http2Builder` does not currently expose
                                    // an explicit `keep_alive_while_idle` toggle; this flag is
                                    // reserved for future use or more advanced wiring.
                                } else {
                                    let mut builder = Builder::new(TokioExecutor::new());

                                    builder.http1().keep_alive(true);

                                    let connection =
                                        builder.serve_connection_with_upgrades(io, request_service);
                                    if let Err(err) = connection.await {
                                        eprintln!("Error serving connection: {:?}", err);
                                    }
                                }
                            } else {
                                let mut builder = http1::Builder::new();
                                builder.keep_alive(true);
                                let connection = builder
                                    .serve_connection(io, request_service)
                                    .with_upgrades();

                                if let Err(err) = connection.await {
                                    eprintln!("Error serving connection: {:?}", err);
                                }
                            }

                            // Serve the connection with upgrades enabled for WebSocket support
                        });
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection: {}", e);
                    }
                }
            }
        }
    }

    /// Internal error handler for the router.
    ///
    /// This method processes routing errors and converts them into appropriate HTTP responses.
    /// It handles both generic API errors and unexpected system errors.
    pub(crate) async fn error_handler(
        err: routerify_ng::RouteError,
    ) -> Response<Full<hyper::body::Bytes>> {
        let api_err = err.downcast::<ApiError>().unwrap_or_else(|_| {
            return Box::new(ApiError::Generic(
                HttpResponse::new()
                    .internal_server_error()
                    .text("Unhandled error"),
            ));
        });

        // For WebSocket upgrades, we need to take ownership to avoid breaking the upgrade mechanism
        // Cloning the response breaks the upgrade connection, so we must move it
        match *api_err {
            ApiError::WebSocketUpgrade(response) => {
                // Return the response directly without cloning to preserve the upgrade mechanism
                response
            }
            ApiError::Generic(res) => {
                let hyper_res = <HttpResponse as Clone>::clone(&res)
                    .to_hyper_response()
                    .await
                    .map_err(ApiError::from)
                    .unwrap();

                hyper_res
            }
        }
    }

    /// Internal method for serving static files with proper headers and caching support.
    ///
    /// This method handles the complex logic of serving static files, including:
    /// - URL path rewriting to map mount points to filesystem paths
    /// - ETag-based conditional requests (304 Not Modified responses)
    /// - Proper caching headers
    /// - Error handling for missing files
    ///
    /// ## Arguments
    ///
    /// * `req` - The incoming HTTP request
    /// * `mount_root` - The URL path where static files are mounted
    /// * `fs_root` - The filesystem directory containing the static files
    ///
    /// ## Returns
    ///
    /// * `Ok(Response<Body>)` - Successfully served file or 304 Not Modified
    /// * `Err(std::io::Error)` - File not found or other I/O error
    pub(crate) async fn serve_static_with_headers<B>(
        req: Request<B>,
        mount_root: String,
        fs_root: String,
    ) -> Result<Response<Full<hyper::body::Bytes>>, std::io::Error>
    where
        B: hyper::body::Body<Data = hyper::body::Bytes> + Send + 'static,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        // Rewrite the request URI by stripping the mount_root prefix so that
        // "/static/index.html" maps to "fs_root/index.html" rather than
        // "fs_root/static/index.html".
        let (mut parts, body) = req.into_parts();
        let original_uri = parts.uri.clone();
        let original_path = original_uri.path();
        let if_none_match = parts
            .headers
            .get(header::IF_NONE_MATCH)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let trimmed_path = if mount_root == "/" {
            // If mounting at root, serve the path as-is
            original_path
        } else if original_path.starts_with(&mount_root) {
            // Strip the mount root prefix, but ensure we don't create an empty path
            let remaining = &original_path[mount_root.len()..];
            if remaining.is_empty() {
                "/"
            } else {
                remaining
            }
        } else {
            // Path doesn't match mount root - this shouldn't happen in normal routing
            original_path
        };

        let normalized_path = if trimmed_path.is_empty() {
            "/"
        } else {
            trimmed_path
        };

        let new_path_and_query = if let Some(query) = original_uri.query() {
            format!("{}?{}", normalized_path, query)
        } else {
            normalized_path.to_string()
        };

        parts.uri = match new_path_and_query.parse() {
            Ok(uri) => uri,
            Err(e) => {
                eprintln!(
                    "Error parsing URI: {} (original: {}, mount_root: {}, trimmed: {}, normalized: {})",
                    e, original_path, mount_root, trimmed_path, normalized_path
                );
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid URI after rewriting: {}", e),
                ));
            }
        };

        let rewritten_req = Request::from_parts(parts, body);

        let static_service = Static::new(Path::new(fs_root.as_str()));

        match static_service.serve(rewritten_req).await {
            Ok(mut response) => {
                response
                    .headers_mut()
                    .insert("Cache-Control", "public, max-age=86400".parse().unwrap());
                response
                    .headers_mut()
                    .insert("X-Served-By", "hyper-staticfile".parse().unwrap());
                // Handle conditional request with If-None-Match since hyper-staticfile 0.9
                // does not evaluate it. If ETag matches, return 304 with empty body.
                if let Some(if_none_match_value) = if_none_match {
                    if let Some(etag) = response.headers().get(header::ETAG) {
                        if let Ok(etag_value) = etag.to_str() {
                            if if_none_match_value == etag_value {
                                let mut builder =
                                    Response::builder().status(StatusCode::NOT_MODIFIED);
                                if let Some(h) = builder.headers_mut() {
                                    // carry forward ETag, Cache-Control, Last-Modified, etc.
                                    for (k, v) in response.headers().iter() {
                                        h.insert(k.clone(), v.clone());
                                    }
                                    h.remove(header::CONTENT_LENGTH);
                                }
                                return Ok(builder.body(Full::from(Bytes::new())).unwrap());
                            }
                        }
                    }
                }
                // Convert hyper_staticfile::Body to Full<Bytes>
                let (parts, body) = response.into_parts();
                let collected = body.collect().await.map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to collect body: {}", e),
                    )
                })?;
                let body_bytes = collected.to_bytes();
                let full_body = Full::from(body_bytes);
                Ok(Response::from_parts(parts, full_body))
            }
            Err(e) => Err(e),
        }
    }

    /// Internal method for building a router instance.
    ///
    /// This is used internally for testing and development purposes.
    pub(crate) fn _build_router(&self) -> routerify_ng::Router<ApiError> {
        routerify_ng::Router::builder()
            .err_handler(Self::error_handler)
            .build()
            .unwrap()
    }
}
