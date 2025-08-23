#![warn(missing_docs)]

//! # Middlewares
//!
//! This module provides a collection of middleware components for the Ripress web framework.
//! Middlewares are reusable, composable functions that can intercept, modify, or augment HTTP
//! requests and responses as they flow through your application. They are essential for implementing
//! cross-cutting concerns such as CORS, logging, file uploads, rate limiting, body size enforcement,
//! and response compression.
//!
//! ## Available Middlewares
//!
//! - [`cors`] - Cross-Origin Resource Sharing (CORS) support
//! - [`logger`] - HTTP request/response logging
//! - [`file_upload`] - File upload handling (binary and multipart forms)
//! - [`rate_limiter`] - Request rate limiting
//! - [`body_limit`] - Request body size enforcement
//! - [`compression`] - Response body compression (gzip)
//!
//! ## Usage
//!
//! Middlewares can be registered globally or for specific routes using the [`App`] builder methods.
//! For example, to enable CORS and logging:
//!
//! ```rust
//! use ripress::app::App;
//!
//! let mut app = App::new();
//! app.use_cors(None);
//! app.use_logger(None);
//! ```
//!
//! See each middleware's documentation for configuration options and advanced usage.

/// Cross-Origin Resource Sharing (CORS) middleware
///
/// This module provides middleware for handling CORS headers and requests in your application.
/// Use this middleware to control which origins are allowed to access your resources, set allowed
/// methods, headers, and handle preflight requests automatically.
///
/// ## Features
/// - Configurable allowed origins, methods, and headers
/// - Automatic handling of preflight (OPTIONS) requests
/// - Support for credentials and custom headers
///
/// ## Usage
/// ```rust
/// use ripress::app::App;
/// use ripress::middlewares::cors::CorsConfig;
///
/// let mut app = App::new();
/// app.use_cors(Some(CorsConfig {
///     allowed_origin: "https://example.com",
///     allowed_methods: "GET, POST, PUT, DELETE, OPTIONS, HEAD",
///     ..Default::default()
/// }));
/// ```
pub mod cors;

/// Logger middleware
///
/// This module provides middleware for logging HTTP requests and responses.
/// It can be used to log details such as the request method, path, status code,
/// and response time for debugging, monitoring, and analytics purposes.
///
/// ## Features
/// - Logs request method, path, and status code
/// - Measures and logs response time
/// - Customizable log format
///
/// ## Usage
/// ```rust
/// use ripress::app::App;
///
/// let mut app = App::new();
/// app.use_logger(None);
/// ```
pub mod logger;

/// File uploader middleware
///
/// This module provides middleware for handling file uploads in your application.
/// It processes binary request bodies and saves files to a configurable upload directory
/// with automatic extension detection and unique filename generation.
///
/// ## Features
/// - **Binary file processing**: Handles raw binary content uploads
/// - **Multipart form support**: Extracts all file parts and text fields from `multipart/form-data`
/// - **Automatic extension detection**: Uses the `infer` crate to detect file types
/// - **Unique filenames**: Generates UUID-based names to prevent conflicts
/// - **Configurable storage**: Customizable upload directory with fallback
/// - **Non-blocking operation**: Continues request processing even if uploads fail
/// - **Error logging**: Comprehensive logging for debugging upload issues
/// - **Form field mapping**: Maps form field names to generated UUID filenames
///
/// ## Usage
/// ```rust
/// use ripress::{app::App, middlewares::file_upload::{FileUploadConfiguration, file_upload}};
///
/// let mut app = App::new();
/// app.use_middleware("/upload", file_upload(Some(FileUploadConfiguration::default())));
/// ```
///
/// ## Request Data Added
/// When files are successfully uploaded, the middleware adds these fields to the request:
/// - `uploaded_file_count`: Number of files successfully uploaded
/// - `uploaded_files`: JSON array of uploaded file info (filenames, paths, original names)
/// - For backwards compatibility (first file only):
///   - `uploaded_file`: The generated filename of the first file
///   - `uploaded_file_path`: The full path where the first file was saved
///   - `original_filename`: Original filename if available
///
/// ## Form Field Access
/// For multipart forms, text fields are automatically extracted and available via `req.form_data()`.
/// File field names are mapped to their generated UUID filenames for easy access.
///
/// ## Limitations
/// - Works with `RequestBodyType::BINARY` content
/// - For `multipart/form-data`, all file parts are extracted and saved
/// - Files are saved with UUID-based names to prevent conflicts
/// - The middleware automatically handles directory creation
/// - Upload failures are logged to stderr for debugging
pub mod file_upload;

/// Rate Limiter middleware
///
/// This module provides middleware for rate limiting incoming requests to your application.
/// It helps prevent abuse and denial-of-service attacks by restricting the number of requests
/// a client can make within a specified time window.
///
/// ## Features
/// - **Configurable limits**: Set maximum requests per client per time window
/// - **IP-based or custom keying**: Limit by IP address or custom identifier
/// - **Flexible window durations**: Choose per-second, per-minute, etc.
/// - **Automatic response**: Returns HTTP 429 Too Many Requests when limit is exceeded
/// - **Easy integration**: Plug into your app with a single middleware call
///
/// ## Usage
/// ```no_run
/// use ripress::{app::App, middlewares::rate_limiter::RateLimiterConfig};
///
/// let mut app = App::new();
/// // Allow 100 requests per minute per IP
/// app.use_rate_limiter(Some(RateLimiterConfig {
///     max_requests: 100,
///     proxy: false,
///     message: "Too many requests".to_string(),
///     ..Default::default()
/// }));
/// ```
///
/// ## How It Works
/// The middleware tracks the number of requests from each client (by IP or custom key)
/// within a rolling or fixed time window. If the client exceeds the allowed number of requests,
/// further requests are rejected with a 429 status code until the window resets.
///
/// ## Customization
/// You can configure:
/// - The maximum number of requests allowed
/// - The duration of the time window (in seconds)
/// - The method of identifying clients (IP, header, or custom logic)
/// - Whether the server is behind a proxy (`proxy`) to adjust client IP extraction
/// - The rejection message body for 429 responses (`message`)
///
/// ## Limitations
/// - Rate limiting is per-process; distributed deployments require shared state (e.g., Redis)
/// - Identification is by IP by default; proxies may require custom logic
pub mod rate_limiter;

/// Body Size Limit Middleware
///
/// This middleware enforces a maximum size for incoming HTTP request bodies, helping to prevent
/// resource exhaustion and denial-of-service attacks due to excessively large payloads.
///
/// ## Features
/// - **Configurable limit**: Set a custom maximum body size in bytes, or use the default (1 MiB).
/// - **Early rejection**: Requests exceeding the limit are rejected with a `413 Payload Too Large` response.
/// - **Integration**: Easily enabled via [`App::use_body_limit`].
///
/// ## Usage
/// ```rust
/// use ripress::app::App;
///
/// let mut app = App::new();
/// // Use the default 1 MiB limit
/// app.use_body_limit(None);
///
/// // Set a custom limit (e.g., 2 MiB)
/// app.use_body_limit(Some(2 * 1024 * 1024));
/// ```
///
/// ## Best Practices
/// - **Order matters**: Register this middleware *before* any middleware that reads or processes the request body
///   (such as file uploads or JSON parsers) to ensure the limit is enforced on the raw payload.
/// - **Error handling**: Clients exceeding the limit receive a clear error response and the connection may be closed.
///
/// ## Security
/// Limiting body size is a key defense against certain classes of attacks and accidental misuse.
///
/// ## See Also
/// - [`App::use_body_limit`]
pub mod body_limit;

/// Compression middleware
///
/// This module provides middleware for compressing HTTP response bodies using gzip encoding.
/// It automatically compresses eligible responses when the client supports gzip and the response
/// meets the configured size and content type criteria.
///
/// ## Features
/// - **Automatic gzip compression**: Compresses responses for clients that accept gzip encoding
/// - **Configurable threshold**: Only compresses responses larger than a specified size (default: 1 KiB)
/// - **Content type filtering**: Only compresses compressible content types (e.g., text, JSON, XML, SVG)
/// - **Customizable compression level**: Choose the gzip compression level (0-9)
/// - **Transparent integration**: Adds appropriate `Content-Encoding` and `Vary` headers
///
/// ## Usage
/// ```rust
/// use ripress::app::App;
/// use ripress::middlewares::compression::CompressionConfig;
///
/// let mut app = App::new();
/// // Use default compression settings
/// app.use_compression(None);
///
/// // Use custom compression settings
/// app.use_compression(Some(CompressionConfig {
///     threshold: 2048, // Only compress responses larger than 2 KiB
///     level: 9,        // Maximum compression
/// }));
/// ```
///
/// ## Limitations
/// - Only compresses responses with compressible content types
/// - Does not compress very small responses (below threshold)
/// - Only supports gzip encoding (no brotli/deflate yet)
pub mod compression;

/// Shield middleware
///
/// This module provides a comprehensive set of HTTP security headers to protect web applications
/// from common vulnerabilities such as XSS, clickjacking, MIME sniffing, and more. The shield
/// middleware is highly configurable, allowing you to enable or disable individual protections
/// and customize header values as needed.
///
/// ## Features
/// - **Content Security Policy (CSP)**: Restricts sources for scripts, styles, images, and more
/// - **HTTP Strict Transport Security (HSTS)**: Enforces HTTPS usage
/// - **X-Frame-Options**: Prevents clickjacking by controlling frame embedding
/// - **X-Content-Type-Options**: Disables MIME type sniffing
/// - **X-XSS-Protection**: Enables browser XSS filters
/// - **Referrer Policy**: Controls referrer information sent with requests
/// - **DNS Prefetch Control**: Manages DNS prefetching for privacy
/// - **IE No Open**: Prevents Internet Explorer from executing downloads
/// - **Hide Powered-By**: Removes identifying server headers
/// - **Permissions Policy**: Restricts browser features and APIs
/// - **Cross-Origin Policies**: Controls resource, embedder, and opener policies
/// - **Origin Agent Cluster**: Requests origin-keyed agent clustering
/// - **Cross Domain Policy**: Restricts Flash/Silverlight cross-domain access
///
/// ## Usage
/// ```rust
/// use ripress::app::App;
/// use ripress::middlewares::shield::{ShieldConfig, Hsts};
///
/// let mut app = App::new();
/// // Use default secure settings
/// app.use_shield(None);
///
/// // Customize shield settings
/// app.use_shield(Some(ShieldConfig {
///     hsts: Hsts {
///         enabled: true,
///         ..Default::default()
///     },
///     ..Default::default()
/// }));
/// ```
///
/// ## Best Practices
/// - Use shield as early as possible in your middleware chain.
/// - Review and adjust the configuration to fit your application's needs.
/// - Regularly update your security headers as new threats emerge.
///
/// ## See Also
/// - [`App::use_shield`]
pub mod shield;
