use std::sync::Arc;

#[cfg(feature = "with-wynd")]
use http_body_util::Full;

use crate::app::App;
use crate::helpers::box_future_middleware;
#[cfg(feature = "compression")]
use crate::middlewares::compression::CompressionConfig;
#[cfg(feature = "logger")]
use crate::middlewares::logger::LoggerConfig;
#[cfg(feature = "with-wynd")]
use crate::middlewares::WyndMiddleware;
use crate::middlewares::{
    body_limit::body_limit,
    cors::{cors, CorsConfig},
    rate_limiter::{rate_limiter, RateLimiterConfig},
    shield::{config::ShieldConfig, shield},
    Middleware, MiddlewareType,
};
use crate::req::HttpRequest;
use crate::res::HttpResponse;
use crate::types::HandlerMiddleware;

#[cfg(feature = "with-wynd")]
use crate::types::WyndMiddlewareHandler;
#[cfg(feature = "with-wynd")]
use bytes::Bytes;

impl App {
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
        use crate::middlewares::logger::logger;

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
        use crate::middlewares::compression::compression;

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
}
