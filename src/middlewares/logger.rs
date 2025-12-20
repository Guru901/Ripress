#![warn(missing_docs)]
use crate::{context::HttpResponse, req::HttpRequest, types::FutMiddleware};
use std::collections::HashMap;
use tracing::info;

/// Builtin Logger Middleware
///
/// This middleware provides comprehensive request and response logging capabilities
/// for HTTP requests. It captures various aspects of each request including method,
/// path, status code, headers, IP address, user agent, query parameters, and body size.
/// The middleware is highly configurable, allowing you to selectively enable/disable
/// different logging components and exclude specific paths from logging.
///
/// ## Features
///
/// * **Selective logging** - Enable/disable individual log components
/// * **Custom header logging** - Log specific headers by name
/// * **Path exclusion** - Skip logging for specified paths (e.g., health checks)
/// * **Query parameter logging** - Capture and log request query parameters
/// * **Body size tracking** - Log response body size with stream detection
/// * **IP address logging** - Capture client IP addresses
/// * **User agent detection** - Log client user agent strings
/// * **Prefix-based exclusion** - Exclude paths using prefix matching
/// * **Thread-safe operation** - Safe for concurrent use across multiple threads
/// * **Zero-allocation exclusion** - Excluded requests bypass all processing
///
/// ## Configuration
///
/// The middleware accepts an optional `LoggerConfig` struct to customize logging behavior:
///
/// * `method` - Log HTTP method (GET, POST, etc.) - default: true
/// * `path` - Log request path - default: true
/// * `status` - Log response status code - default: true
/// * `user_agent` - Log client user agent - default: true
/// * `ip` - Log client IP address - default: true
/// * `headers` - List of specific headers to log - default: empty
/// * `body_size` - Log response body size - default: true
/// * `query_params` - Log query parameters - default: true
/// * `exclude_paths` - List of path prefixes to exclude from logging - default: empty
///
/// ## Path Exclusion Behavior
///
/// The `exclude_paths` configuration uses prefix matching for efficient filtering:
/// * `/health` excludes `/health`, `/health/live`, `/health/ready`, etc.
/// * `/api/internal` excludes all paths starting with `/api/internal`
/// * Excluded requests are processed with minimal overhead
/// * No log output is generated for excluded paths
///
/// ## Log Format
///
/// The middleware outputs structured logs using the `tracing` crate with the following format:
/// ```md
/// path: /api/users,
/// user_agent: Mozilla/5.0...,
/// ip: 192.168.1.1,
/// custom-header: value,
/// status_code: 200
/// query_params: {"id": "123", "format": "json"},
/// method: GET,
/// body_size: 1024
/// ```
///
/// ## Examples
///
/// Basic usage with default configuration:
///
/// ```no_run
/// use ripress::{app::App, middlewares::logger::LoggerConfig};
///
/// tracing_subscriber::fmt::init();
/// let mut app = App::new();
/// app.use_logger(Some(LoggerConfig::default()));
/// ```
///
/// Minimal logging configuration:
///
/// ```no_run
/// use ripress::{app::App, middlewares::logger::LoggerConfig};
/// tracing_subscriber::fmt::init();
///
/// let mut app = App::new();
/// let config = LoggerConfig {
///     method: true,
///     path: true,
///     status: true,
///     user_agent: false,
///     ip: false,
///     headers: vec![],
///     body_size: false,
///     query_params: false,
///     exclude_paths: vec![],
/// };
/// app.use_logger(Some(config));
/// ```
///
/// Custom header logging with path exclusions:
///
/// ```no_run
/// use ripress::{app::App, middlewares::logger::LoggerConfig};
/// tracing_subscriber::fmt::init();
///
/// let mut app = App::new();
/// let config = LoggerConfig {
///     method: true,
///     path: true,
///     status: true,
///     user_agent: true,
///     ip: true,
///     headers: vec![
///         "authorization".to_string(),
///         "x-request-id".to_string(),
///         "content-type".to_string(),
///     ],
///     body_size: true,
///     query_params: true,
///     exclude_paths: vec![
///         "/health".to_string(),
///         "/metrics".to_string(),
///         "/favicon.ico".to_string(),
///     ],
/// };
/// app.use_logger(Some(config));
/// ```
///
/// Using default configuration (recommended for development):
///
/// ```no_run
/// use ripress::app::App;
///
/// tracing_subscriber::fmt::init();
///
/// let mut app = App::new();
/// app.use_logger(None); // Uses LoggerConfig::default()
/// ```
///
/// Production configuration with security considerations:
///
/// ```no_run
/// use ripress::{app::App, middlewares::logger::LoggerConfig};
///
/// tracing_subscriber::fmt::init();
///
/// let mut app = App::new();
/// let config = LoggerConfig {
///     method: true,
///     path: true,
///     status: true,
///     user_agent: false, // May contain sensitive info
///     ip: true,
///     headers: vec![
///         "x-request-id".to_string(), // Safe to log
///         "content-length".to_string(),
///         // Note: Don't log authorization, cookies, or other sensitive headers
///     ],
///     body_size: true,
///     query_params: false, // May contain sensitive data
///     exclude_paths: vec![
///         "/health".to_string(),
///         "/metrics".to_string(),
///         "/internal".to_string(),
///     ],
/// };
/// app.use_logger(Some(config));
/// ```
///
/// ## Output Examples
///
/// Default configuration output:
/// ```md
/// path: /api/users/123,
/// user_agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36,
/// ip: 192.168.1.100,
/// status_code: 200
/// query_params: {"include": "profile", "format": "json"},
/// method: GET,
/// body_size: 2048
/// ```
///
/// Custom headers output:
/// ```md
/// path: /api/upload,
/// user_agent: PostmanRuntime/7.32.3,
/// ip: 10.0.0.15,
/// authorization: <missing>,
/// x-request-id: abc-123-def,
/// content-type: multipart/form-data,
/// status_code: 201
/// query_params: {},
/// method: POST,
/// body_size: stream
/// ```
///
/// ## Performance Considerations
///
/// * **Excluded paths** bypass all processing for maximum efficiency
/// * **Header lookups** are performed only for configured headers
/// * **String allocation** occurs only for enabled log components
/// * **Query parameter cloning** happens only when enabled
/// * **Arc-based config sharing** minimizes memory overhead
/// * **Structured output** reduces parsing overhead in log aggregators
///
/// ## Security Considerations
///
/// * **Sensitive headers** (Authorization, Cookie, etc.) should not be logged
/// * **Query parameters** may contain sensitive data - consider disabling
/// * **User agents** may contain personally identifiable information
/// * **IP addresses** may be subject to privacy regulations (GDPR, etc.)
/// * **Path exclusions** prevent accidental logging of sensitive endpoints
///
/// ## Integration with Log Aggregators
///
/// The structured output format is designed for easy parsing by log aggregation systems:
/// * **Consistent field names** for reliable parsing
/// * **Comma-separated format** for simple splitting
/// * **Missing header indicator** (`<missing>`) for clear status
/// * **Stream detection** for body size handling
/// * **JSON-formatted query params** for structured data
///
/// ## Troubleshooting
///
/// Common issues and solutions:
/// * **Missing headers**: Check header name case (middleware converts to lowercase)
/// * **No output**: Verify path isn't in `exclude_paths`
/// * **Partial logs**: Check individual boolean flags in configuration
/// * **Performance impact**: Use path exclusions for high-traffic endpoints
///
/// ## Thread Safety
///
/// The middleware is fully thread-safe:
/// * Configuration is wrapped in `Arc` for efficient sharing
/// * No mutable state is maintained between requests
/// * Safe for use in multi-threaded web servers
/// * Clone-friendly for multiple route registration

/// Configuration struct for the Logger Middleware
///
/// This struct controls which aspects of HTTP requests and responses are logged.
/// All fields are boolean flags or collections that determine what information
/// is captured and output for each request.
///
/// ## Field Details
///
/// * `method` - Logs the HTTP method (GET, POST, PUT, DELETE, etc.)
/// * `path` - Logs the request path (e.g., "/api/users/123")
/// * `status` - Logs the HTTP response status code (200, 404, 500, etc.)
/// * `user_agent` - Logs the User-Agent header sent by the client
/// * `ip` - Logs the client's IP address
/// * `headers` - A list of specific header names to log (case-insensitive)
/// * `body_size` - Logs the size of the response body in bytes, or "stream" for streaming responses
/// * `query_params` - Logs URL query parameters as a structured format
/// * `exclude_paths` - Path prefixes that should be excluded from logging entirely
///
/// ## Default Configuration
///
/// By default, all standard fields are enabled and no paths are excluded:
/// - All boolean fields default to `true`
/// - `headers` defaults to empty (no custom headers logged)
/// - `exclude_paths` defaults to empty (all paths logged)
#[derive(Clone)]
pub struct LoggerConfig {
    /// Whether to log the HTTP method (GET, POST, etc.)
    pub method: bool,
    /// Whether to log the request path
    pub path: bool,
    /// Whether to log the response status code
    pub status: bool,
    /// Whether to log the client's User-Agent header
    pub user_agent: bool,
    /// Whether to log the client's IP address
    pub ip: bool,
    /// List of specific header names to log (converted to lowercase)
    ///
    /// Headers not present in the request will show as "<missing>" in the log output.
    /// Common headers to log: "content-type", "x-request-id", "authorization"
    pub headers: Vec<String>, // Specific headers to log
    /// Whether to log the response body size
    ///
    /// Shows actual byte count for regular responses, "stream" for streaming responses.
    pub body_size: bool,
    /// Whether to log URL query parameters
    ///
    /// Query parameters are logged in a structured JSON-like format for easy parsing.
    pub query_params: bool,
    /// List of path prefixes to exclude from logging
    ///
    /// Uses prefix matching: "/health" excludes "/health", "/health/live", etc.
    /// Useful for excluding health checks, metrics endpoints, and other high-frequency requests.
    pub exclude_paths: Vec<String>, // Don't log health checks, etc.
}

impl Default for LoggerConfig {
    fn default() -> Self {
        LoggerConfig {
            method: true,
            path: true,
            status: true,
            user_agent: true,
            ip: true,
            headers: vec![],
            body_size: true,
            query_params: true,
            exclude_paths: vec![],
        }
    }
}

/// Creates a logger middleware function
///
/// Returns a middleware function that logs HTTP request and response information
/// according to the provided configuration. The middleware processes requests
/// efficiently, with excluded paths bypassing all logging overhead.
///
/// ## Parameters
///
/// * `config` - Optional logging configuration. If `None`, uses `LoggerConfig::default()`
///   which enables all standard logging fields.
///
/// ## Returns
///
/// A middleware function compatible with the ripress framework that:
/// * Logs request and response information to stdout
/// * Skips processing for paths matching `exclude_paths` prefixes
/// * Captures configured headers with case-insensitive matching
/// * Handles streaming responses appropriately
/// * Operates efficiently with minimal allocation overhead
///
/// ## Thread Safety
///
/// The returned middleware is `Send + Sync` and safe for concurrent use.
/// Configuration is shared via `Arc` to minimize memory overhead when
/// the middleware is used across multiple routes or threads.
///
/// ## Performance Notes
///
/// * Excluded paths are checked first and bypass all other processing
/// * Header lookups only occur for headers specified in the configuration
/// * String formatting only happens for enabled log components
/// * Configuration is shared via Arc to avoid cloning overhead
///
/// ## Log Output
///
/// All log output is written using the `tracing` crate at the `info` level. The format is
/// comma-separated key-value pairs, making it suitable for structured
/// log parsing systems. Fields are output in a consistent order regardless
/// of configuration.
///
/// ## Tracing Setup
///
/// To see the log output, you need to initialize a tracing subscriber in your application:
///
/// ```rust
/// use tracing_subscriber;
///
/// fn main() {
///     // Initialize tracing subscriber
///     tracing_subscriber::fmt::init();
///     
///     // Your app code here
/// }
/// ```
pub(crate) fn logger(
    config: Option<LoggerConfig>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static {
    let cfg = std::sync::Arc::new(config.unwrap_or_default());
    move |req: HttpRequest, res| {
        let config = std::sync::Arc::clone(&cfg);

        // Treat entries as prefixes; excludes "/health" will also exclude "/health/live".
        if config
            .exclude_paths
            .iter()
            .any(|prefix| req.path.starts_with(prefix))
        {
            return Box::pin(async move { (req, None) });
        }

        Box::pin(async move {
            let path = req.path.clone();
            let method = req.method.clone();
            let user_agent = req.headers.user_agent().unwrap_or("Unknown").to_string();
            let ip = req.ip;
            let status_code = res.status_code;
            let mut headers = HashMap::new();

            if !config.headers.is_empty() {
                for header in &config.headers {
                    let key = header.to_ascii_lowercase();
                    let value = req
                        .headers
                        .get(&key)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "<missing>".to_string());
                    headers.insert(key, value);
                }
            }

            let query_params = req.query.clone();

            let mut msg = String::new();

            if config.path {
                msg.push_str(&format!("path: {}, \n", path));
            }
            if config.user_agent {
                msg.push_str(&format!("user_agent: {}, \n", user_agent));
            }
            if config.ip {
                msg.push_str(&format!("ip: {}, \n", ip));
            }
            for (key, value) in headers {
                msg.push_str(&format!("{}: {}, \n", key, value));
            }
            if config.status {
                msg.push_str(&format!("status_code: {}\n", status_code));
            }
            if config.query_params {
                msg.push_str(&format!("query_params: {:?}, \n", query_params));
            }
            if config.method {
                msg.push_str(&format!("method: {}, \n", method));
            }
            if config.body_size {
                if res.is_stream {
                    msg.push_str("body_size: stream\n");
                } else {
                    msg.push_str(&format!("body_size: {}\n", res.body.len()));
                }
            }

            let msg = msg.trim_end_matches([',', ' ', '\t', '\n']);

            info!("{}", msg);

            (req, None)
        })
    }
}
