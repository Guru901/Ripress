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

#[cfg(feature = "compression")]
use crate::middlewares::compression::{compression, CompressionConfig};

#[cfg(feature = "logger")]
use crate::middlewares::logger::{logger, LoggerConfig};

#[cfg(feature = "with-wynd")]
use crate::middlewares::WyndMiddleware;
#[cfg(feature = "with-wynd")]
use crate::types::WyndMiddlewareHandler;

use crate::{
    helpers::{exec_post_middleware, exec_pre_middleware},
    middlewares::{Middleware, MiddlewareType},
    req::HttpRequest,
    res::HttpResponse,
    router::Router,
    types::{HttpMethods, RouterFns, Routes},
};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{header, http::StatusCode, Method, Request, Response};
use hyper_staticfile::Static;
use routerify_ng::{ext::RequestExt, RouterService};
use std::{collections::HashMap, net::SocketAddr, path::Path, sync::Arc, time::Duration};
use tokio::net::TcpListener;

pub(crate) mod api_error;

/// Handler module for managing server connections, HTTP/2/1 serving logic, and connection-level configuration.
pub mod handler;
/// Middleware support for the App struct, including common and user-defined middleware functionality.
pub mod middlewares;

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
    /// Stores all registered HTTP route handlers for this application.
    routes: Routes,

    /// The host address or interface the server will bind to (e.g., `"0.0.0.0"` or `"127.0.0.1"`).
    /// This field determines which IP address or hostname the HTTP server accepts connections on.
    pub(crate) host: String,

    /// Enables or disables HTTP/2 support for the server.
    /// By default, HTTP/2 is **enabled**.
    pub(crate) http2: bool,

    /// Optional advanced configuration for HTTP/2 behavior.
    pub(crate) http2_config: Option<Http2Config>,

    /// The list of middleware functions to be applied to requests.
    pub(crate) middlewares: Vec<Arc<Middleware>>,

    /// Static file mappings from mount path to filesystem path.
    pub(crate) static_files: HashMap<&'static str, &'static str>,

    /// Enables or disables graceful shutdown support for the server.
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
    fn routes(&mut self) -> &mut Routes {
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
            routes: HashMap::new(),
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
    pub fn router(&mut self, mut router: Router) {
        let base_path = router.base_path;
        for (path, methods) in router.routes() {
            for (method, handler) in methods.to_owned() {
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
        for (path, methods) in &self.routes {
            for (method, handler) in methods {
                let handler = Arc::clone(handler);

                let method = match method {
                    HttpMethods::GET => Method::GET,
                    HttpMethods::POST => Method::POST,
                    HttpMethods::PUT => Method::PUT,
                    HttpMethods::DELETE => Method::DELETE,
                    HttpMethods::PATCH => Method::PATCH,
                    HttpMethods::HEAD => Method::HEAD,
                    HttpMethods::OPTIONS => Method::OPTIONS,
                };

                router = router.add(path, vec![method], move |mut req| {
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

        let mut shutdown = if self.graceful_shutdown {
            Some(Box::pin(tokio::signal::ctrl_c()))
        } else {
            None
        };

        loop {
            let accept_result = if let Some(ref mut sig) = shutdown {
                tokio::select! {
                    result = listener.accept() => Some(result),
                    _ = sig.as_mut() => None,
                }
            } else {
                Some(listener.accept().await)
            };

            match accept_result {
                Some(Ok((stream, _))) => {
                    let service = Arc::clone(&router_service);
                    let http2_enabled = http2_enabled;
                    let http2_config = http2_config.clone();

                    tokio::task::spawn(async move {
                        Self::handle_connection(stream, service, http2_enabled, http2_config).await;
                    });
                }
                Some(Err(e)) => {
                    eprintln!("Error accepting connection: {}", e);
                }
                None => {
                    // Shutdown signal received
                    break;
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
