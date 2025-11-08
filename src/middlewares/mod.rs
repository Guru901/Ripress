//! # Middlewares
//!
//! This module provides a comprehensive collection of middleware components for the Ripress web framework.
//! Middlewares are reusable, composable functions that can intercept, modify, or augment HTTP
//! requests and responses as they flow through your application. They are essential for implementing
//! cross-cutting concerns such as CORS, logging, file uploads, rate limiting, body size enforcement,
//! response compression, and security headers.
//!
//! ## Philosophy
//!
//! Ripress middlewares follow the Express.js middleware pattern, where each middleware:
//! - Receives the current request and response objects
//! - Can modify them or perform side effects
//! - Decides whether to continue processing or short-circuit with a response
//! - Can be applied globally or to specific route patterns
//!
//! ## Available Middlewares
//!
//! | Middleware | Purpose | Execution Type |
//! |------------|---------|----------------|
//! | [`cors`] | Cross-Origin Resource Sharing support | Pre-execution |
//! | [`logger`] | HTTP request/response logging | Post-execution |
//! | [`file_upload`] | File upload handling (binary and multipart) | Pre-execution |
//! | [`rate_limiter`] | Request rate limiting and DoS protection | Pre-execution |
//! | [`body_limit`] | Request body size enforcement | Pre-execution |
//! | [`compression`] | Response body compression (gzip) | Post-execution |
//! | [`shield`] | Comprehensive security headers | Pre-execution |
//!
//! ## Middleware Execution Order
//!
//! Middlewares are executed in a specific order during the request lifecycle:
//!
//! ```text
//! 1. Pre-execution middlewares (in registration order)
//!    ├── CORS handling
//!    ├── Rate limiting
//!    ├── Body size limits
//!    ├── Security headers
//!    └── File uploads
//!
//! 2. Route handler execution
//!
//! 3. Post-execution middlewares (in registration order)
//!    ├── Response compression
//!    └── Request/response logging
//! ```
//!
//! ## Basic Usage
//!
//! Middlewares can be registered globally or for specific routes using the [`App`] builder methods:
//!
//! ```no_run
//! use ripress::app::App;
//! use ripress::middlewares::cors::CorsConfig;
//! use ripress::middlewares::logger::LoggerConfig;
//! use ripress::types::RouterFns;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Initialize logging (required for logger middleware)
//!     tracing_subscriber::fmt::init();
//!
//!     let mut app = App::new();
//!
//!     // Add security and CORS (pre-execution)
//!     app.use_shield(None);
//!     app.use_cors(Some(CorsConfig {
//!         allowed_origin: "https://myapp.com",
//!         allowed_methods: "GET, POST, PUT, DELETE",
//!         ..Default::default()
//!     }));
//!
//!     // Add rate limiting and body limits
//!     app.use_rate_limiter(None);
//!     app.use_body_limit(Some(10 * 1024 * 1024)); // 10MB limit
//!
//!     // Add response optimization (post-execution)
//!     app.use_compression(None);
//!     app.use_logger(Some(LoggerConfig {
//!         method: true,
//!         path: true,
//!         status: true,
//!         ..Default::default()
//!     }));
//!
//!     // Add routes
//!     app.get("/", |_req, res| async move {
//!         res.ok().text("Hello, World!")
//!     });
//!
//!     app.listen(3000, || println!("Server running with middlewares")).await;
//! }
//! ```
//!
//! ## Custom Middlewares
//!
//! You can create custom middlewares using the pre/post middleware methods:
//!
//! ```no_run
//! use ripress::app::App;
//!
//! let mut app = App::new();
//!
//! // Custom authentication middleware
//! app.use_pre_middleware(Some("/api"), |req, res| async move {
//!     if req.headers.get("authorization").is_none() {
//!         return (req, Some(res.unauthorized().text("Missing auth header")));
//!     }
//!     (req, None) // Continue processing
//! });
//!
//! // Custom response timing middleware
//! app.use_post_middleware(None, |req, mut res| async move {
//!     res = res.set_header("X-Response-Time", "42ms");
//!     (req, Some(res))
//! });
//! ```
//!
//! ## Production Recommendations
//!
//! For production applications, consider this middleware stack:
//!
//! ```no_run
//! use ripress::app::App;
//!
//! let mut app = App::new();
//!
//! // Security (first priority)
//! app.use_shield(None);                    // Security headers
//! app.use_cors(None);                      // CORS handling
//! app.use_rate_limiter(None);             // DoS protection
//! app.use_body_limit(Some(1024 * 1024));  // 1MB body limit
//!
//! // Performance optimization
//! app.use_compression(None);               // Gzip compression
//!
//! // Monitoring (last, to capture everything)
//! app.use_logger(None);                    // Request logging
//! ```
//!
//! See each middleware's documentation for detailed configuration options and advanced usage patterns.

#![warn(missing_docs)]

/// Cross-Origin Resource Sharing (CORS) middleware
///
/// This module provides middleware for handling CORS headers and requests in your application.
/// CORS is essential for web applications that need to make cross-origin requests from browsers,
/// such as API calls from frontend applications hosted on different domains.
///
/// ## Features
///
/// - **Flexible Origin Control**: Allow specific origins, wildcards, or dynamic origin validation
/// - **Method Restrictions**: Control which HTTP methods are allowed for cross-origin requests
/// - **Header Management**: Specify which headers can be sent and received in cross-origin requests
/// - **Credentials Support**: Enable or disable credentials (cookies, authorization headers) for cross-origin requests
/// - **Preflight Handling**: Automatically handles OPTIONS preflight requests according to CORS specification
/// - **Max Age Control**: Set how long browsers can cache preflight responses
///
/// ## Security Considerations
///
/// - **Avoid Wildcards in Production**: Using `*` for `allowed_origin` with credentials is dangerous
/// - **Principle of Least Privilege**: Only allow the origins, methods, and headers you actually need
/// - **Regular Auditing**: Review your CORS configuration regularly as your application evolves
///
/// ## Usage Examples
///
/// ```rust
/// use ripress::app::App;
/// use ripress::middlewares::cors::CorsConfig;
///
/// let mut app = App::new();
///
/// // Permissive CORS for development
/// app.use_cors(None);
///
/// // Production CORS configuration
/// app.use_cors(Some(CorsConfig {
///     allowed_origin: "https://myapp.com",
///     allowed_methods: "GET, POST, PUT, DELETE, OPTIONS",
///     allowed_headers: "Content-Type, Authorization, X-Requested-With",
///     allow_credentials: true,
///     ..Default::default()
/// }));
///
/// // Multiple origins (comma-separated)
/// app.use_cors(Some(CorsConfig {
///     allowed_origin: "https://app1.com, https://app2.com, https://localhost:3000",
///     ..Default::default()
/// }));
/// ```
///
/// ## Default Behavior
///
/// When using `None` configuration, the middleware applies permissive defaults suitable for development:
/// - Allows all origins (`*`)
/// - Allows common HTTP methods
/// - Allows common headers
/// - Credentials disabled for security
/// - 24-hour preflight cache
///
/// ## Browser Compatibility
///
/// This middleware implements the W3C CORS specification and is compatible with all modern browsers.
/// Internet Explorer 8-9 have limited support and may require additional polyfills.
pub mod cors;

/// Request/Response Logger middleware
///
/// This module provides comprehensive logging middleware for HTTP requests and responses.
/// It integrates with the `tracing` ecosystem to provide structured, configurable logging
/// that's essential for monitoring, debugging, and analytics in production applications.
///
/// ## Features
///
/// - **Comprehensive Logging**: Captures method, path, status code, response time, and more
/// - **Structured Logs**: Uses `tracing` for structured, machine-parseable log output
/// - **Configurable Fields**: Choose which request/response fields to log
/// - **Performance Metrics**: Automatically measures and logs request duration
/// - **Error Tracking**: Logs error responses with additional context
/// - **Custom Formatting**: Supports custom log formats and levels
/// - **Async-Safe**: Designed for high-performance async applications
///
/// ## Prerequisites
///
/// The logger middleware requires a `tracing` subscriber to be initialized in your application:
///
/// ```no_run
/// // Simple console logging
/// tracing_subscriber::fmt::init();
///
/// // Or with basic customization
/// use tracing_subscriber::fmt;
/// fmt()
///     .with_target(false)
///     .compact()
///     .init();
/// ```
///
/// ## Usage Examples
///
/// ```rust
/// use ripress::app::App;
/// use ripress::middlewares::logger::LoggerConfig;
///
/// // Initialize tracing subscriber
/// tracing_subscriber::fmt::init();
///
/// let mut app = App::new();
///
/// // Basic logging with defaults
/// app.use_logger(None);
///
/// // Detailed logging configuration
/// app.use_logger(Some(LoggerConfig {
///     method: true,
///     path: true,
///     status: true,
///     user_agent: true,
///     ip: true,
///     headers: vec!["content-type".to_string()],
///     body_size: true,
///     query_params: true,
///     exclude_paths: vec!["/health".to_string()],
/// }));
/// ```
///
/// ## Log Output Format
///
/// The logger produces structured log entries like:
///
/// ```text
/// 2024-01-15T10:30:45.123Z INFO ripress::logger: HTTP Request
///   method: "GET"
///   path: "/api/users"
///   query: "page=1&limit=10"
///   status: 200
///   duration_ms: 45
///   user_agent: "Mozilla/5.0..."
///   remote_addr: "192.168.1.100"
/// ```
///
/// ## Performance Considerations
///
/// - **Post-Execution**: Runs after route handlers to capture complete response information
/// - **Minimal Overhead**: Optimized for high-throughput applications
/// - **Conditional Logging**: Can filter out successful requests to reduce log volume
/// - **Async-Safe**: Does not block request processing
///
/// ## Integration with Monitoring
///
/// The structured logs work well with log aggregation systems:
/// - **ELK Stack**: Elasticsearch, Logstash, Kibana
/// - **Grafana Loki**: For metrics and alerting
/// - **DataDog**: Application performance monitoring
/// - **New Relic**: Real-time monitoring and analytics
pub mod logger;

/// File Upload middleware
///
/// This module provides comprehensive file upload handling for both binary uploads and multipart
/// form data. It's designed to handle file uploads efficiently while providing flexibility for
/// various upload scenarios including single files, multiple files, and forms with mixed content.
///
/// ## Features
///
/// - **Multiple Upload Types**: Handles both raw binary uploads and `multipart/form-data`
/// - **Automatic Type Detection**: Uses the `infer` crate to detect file types from content
/// - **Unique Filename Generation**: UUID-based filenames prevent conflicts and improve security
/// - **Configurable Storage**: Customizable upload directory with automatic creation
/// - **Form Field Extraction**: Extracts both files and text fields from multipart forms
/// - **Metadata Preservation**: Maintains original filenames and content types
/// - **Error Resilience**: Continues processing even if individual file uploads fail
/// - **Security-First**: Prevents directory traversal and validates file types
/// - **Non-Blocking**: Asynchronous file operations don't block the request pipeline
///
/// ## Upload Types Supported
///
/// ### Binary Uploads
/// Direct binary file uploads via POST/PUT with `Content-Type: application/octet-stream` or specific MIME types.
///
/// ### Multipart Forms
/// Standard HTML form uploads with `Content-Type: multipart/form-data`, supporting:
/// - Multiple file fields
/// - Mixed text and file fields
/// - File metadata preservation
///
/// ## Configuration
///
/// ```no_run
/// use ripress::app::App;
/// use ripress::middlewares::file_upload::{FileUploadConfiguration, file_upload};
///
/// let mut app = App::new();
///
/// // Default configuration
/// app.use_pre_middleware(Some("/upload"), file_upload(None));
///
/// // Custom configuration
/// app.use_pre_middleware(Some("/upload"), file_upload(Some(FileUploadConfiguration {
///     upload_dir: "./uploads".to_string(),
///     max_file_size: 10 * 1024 * 1024, // 10MB per file
///     max_files: 100,
///     allowed_file_types: vec!["jpeg".to_string(), "png".to_string()],
/// })));
/// ```
///
/// ## Request Data Enhancement
///
/// After successful uploads, the middleware enriches the request with upload information:
///
/// ### Multiple Files Support
/// - `uploaded_file_count`: Total number of successfully uploaded files
/// - `uploaded_files`: JSON array with detailed information for each uploaded file
///
/// ### Legacy Single File Support (Backward Compatibility)
/// - `uploaded_file`: Generated filename of the first uploaded file
/// - `uploaded_file_path`: Full filesystem path of the first uploaded file
/// - `original_filename`: Original client-provided filename (if available)
///
/// ### Form Field Access
/// For multipart forms, text fields are automatically extracted and accessible via `req.form_data()`.
///
/// ## Usage Examples
///
/// ### Route Handler Example
/// ```rust
/// use ripress::app::App;
/// use ripress::types::RouterFns;
/// use serde_json::Value;
/// use ripress::middlewares::file_upload::file_upload;
///
/// let mut app = App::new();
///
/// app.use_pre_middleware(Some("/upload"), file_upload(None));
///
/// app.post("/upload", |req, res| async move {
///     if let Some(file_count) = req.get_data("uploaded_file_count") {
///         let count: usize = file_count.parse().unwrap_or(0);
///         
///         if count > 0 {
///             // Access detailed file information
///             if let Some(files_json) = req.get_data("uploaded_files") {
///                 let files: Value = serde_json::from_str(files_json.as_str()).unwrap();
///                 return res.ok().json(serde_json::json!({
///                     "message": "Files uploaded successfully",
///                     "count": count,
///                     "files": files
///                 }));
///             }
///         }
///     }
///     
///     res.bad_request().text("No files uploaded")
/// });
/// ```
///
/// ## Security Considerations
///
/// - **File Type Validation**: Always validate uploaded file types in your route handlers
/// - **Size Limits**: Configure appropriate file size limits to prevent DoS attacks
/// - **Filename Sanitization**: The middleware uses UUID filenames to prevent path traversal
/// - **Upload Directory**: Ensure upload directories are outside the web root
/// - **Virus Scanning**: Consider integrating virus scanning for user-uploaded content
/// - **Access Control**: Implement proper authentication/authorization for upload endpoints
///
/// ## Error Handling
///
/// The middleware handles errors gracefully:
/// - Individual file upload failures don't stop processing of other files
/// - Errors are logged to stderr with detailed information
/// - Request processing continues even if uploads fail
/// - Check `uploaded_file_count` to verify successful uploads
///
/// ## Performance Considerations
///
/// - **Streaming**: Files are processed in streaming fashion to minimize memory usage
/// - **Async I/O**: All file operations are non-blocking
/// - **Early Validation**: File type and size validation happens before writing to disk
/// - **Directory Structure**: Consider organizing uploads into subdirectories for large volumes
pub mod file_upload;

/// Rate Limiting middleware
///
/// This module provides sophisticated rate limiting functionality to protect your application
/// from abuse, denial-of-service attacks, and ensure fair resource usage among clients.
/// The middleware supports various rate limiting strategies and is designed to be both
/// flexible and performant.
///
/// ## Features
///
/// - **Flexible Client Identification**: Rate limit by IP address, custom headers, or user ID
/// - **Configurable Time Windows**: Support for fixed windows, sliding windows, and token buckets
/// - **Multiple Limiting Strategies**: Request count, bandwidth, or custom metrics
/// - **Proxy-Aware**: Proper IP extraction when behind reverse proxies or load balancers
/// - **Custom Response Messages**: Configurable error messages and status codes
/// - **Header Information**: Provides rate limit status in response headers
/// - **Memory Efficient**: Uses efficient data structures for tracking client requests
/// - **Redis Integration**: Optional distributed rate limiting across multiple servers
///
/// ## Rate Limiting Strategies
///
/// ### Fixed Window
/// Resets the counter at fixed intervals (e.g., every minute).
///
/// ### Sliding Window
/// More accurate than fixed windows, considers requests within the last N time units.
///
/// ### Token Bucket
/// Allows bursts while maintaining average rate limits over time.
///
/// ## Configuration Examples
///
/// ```no_run
/// use ripress::app::App;
/// use ripress::middlewares::rate_limiter::RateLimiterConfig;
/// use std::time::Duration;
///
/// let mut app = App::new();
///
/// // Basic rate limiting (100 requests per minute)
/// app.use_rate_limiter(None);
///
/// // Strict API rate limiting
/// app.use_rate_limiter(Some(RateLimiterConfig {
///     max_requests: 10,                    // Allow 10 requests
///     window_ms: Duration::from_secs(60),     // Per minute
///     message: "API rate limit exceeded. Try again later.".to_string(),
///     proxy: true,                         // Behind a proxy
///     ..Default::default()
/// }));
///
/// // Generous web app rate limiting
/// app.use_rate_limiter(Some(RateLimiterConfig {
///     max_requests: 1000,                  // Allow 1000 requests
///     window_ms: Duration::from_secs(3600),   // Per hour
///     ..Default::default()
/// }));
/// ```
///
/// ## Client Identification
///
/// The middleware identifies clients using different strategies:
///
/// ### IP Address (Default)
/// ```rust
/// use ripress::middlewares::rate_limiter::RateLimiterConfig;
///
/// // Automatically extracts client IP, considering X-Forwarded-For when proxy=true
/// RateLimiterConfig {
///     proxy: true, // Enable when behind nginx, CloudFlare, etc.
///     ..Default::default()
/// };
/// ```
///
/// ### Custom Header
/// ```rust
/// use ripress::middlewares::rate_limiter::RateLimiterConfig;
///
/// // Rate limit by API key or user ID
/// RateLimiterConfig {
///     ..Default::default()
/// };
/// ```
///
/// ## Response Headers
///
/// The middleware adds informative headers to all responses:
///
/// ```text
/// X-RateLimit-Limit: 100
/// X-RateLimit-Remaining: 95
/// X-RateLimit-Reset: 1642617600
/// X-RateLimit-Window: 60
/// ```
///
/// When rate limit is exceeded, additional headers are added:
/// ```text
/// Retry-After: 45
/// X-RateLimit-Exceeded: true
/// ```
///
///
/// ### Custom Error Responses
/// ```no_run
/// use ripress::middlewares::rate_limiter::RateLimiterConfig;
/// use ripress::app::App;
///
/// let mut app = App::new();
///
/// app.use_rate_limiter(Some(RateLimiterConfig {
///     max_requests: 50,
///     message: serde_json::json!({
///         "error": "rate_limit_exceeded",
///         "message": "Too many requests. Please slow down.",
///         "retry_after": 60,
///         "documentation": "https://api.example.com/docs/rate-limits"
///     }).to_string(),
///     ..Default::default()
/// }));
/// ```
///
/// ## Production Deployment
///
/// ### Single Server
/// The default in-memory rate limiting works well for single-server deployments:
///
/// ```no_run
/// use ripress::middlewares::rate_limiter::RateLimiterConfig;
/// use std::time::Duration;
/// use ripress::app::App;
///
/// let mut app = App::new();
///
/// // Handles server restarts gracefully
/// app.use_rate_limiter(Some(RateLimiterConfig {
///     max_requests: 1000,
///     window_ms: Duration::from_secs(3600),
///     ..Default::default()
/// }));
/// ```
///
/// ### Multiple Servers (Future)
/// For distributed deployments, consider Redis-backed rate limiting:
///
/// ```no_run
/// use ripress::middlewares::rate_limiter::RateLimiterConfig;
/// use ripress::app::App;
///
/// let mut app = App::new();
///
/// // Future feature - distributed rate limiting
/// app.use_rate_limiter(Some(RateLimiterConfig {
///     max_requests: 1000,
///     ..Default::default()
/// }));
/// ```
///
/// ## Security Best Practices
///
/// - **Authentication First**: Apply rate limiting after authentication when possible
/// - **Different Limits**: Use different limits for authenticated vs. anonymous users
/// - **IP Whitelisting**: Consider whitelisting trusted IPs or ranges
/// - **Monitoring**: Monitor rate limit violations for potential attacks
/// - **Gradual Rollout**: Start with generous limits and tighten based on usage patterns
///
/// ## Performance Characteristics
///
/// - **Memory Usage**: O(n) where n is the number of unique clients in the current window
/// - **Lookup Time**: O(1) for client identification and counter updates
/// - **Cleanup**: Automatic cleanup of expired entries to prevent memory leaks
/// - **Throughput**: Minimal overhead, suitable for high-traffic applications
pub mod rate_limiter;

/// Request Body Size Limit middleware
///
/// This middleware provides protection against excessively large request payloads by enforcing
/// configurable size limits on incoming HTTP request bodies. It's a crucial security measure
/// that helps prevent resource exhaustion attacks and ensures predictable memory usage.
///
/// ## Features
///
/// - **Configurable Limits**: Set custom maximum body sizes in bytes with sensible defaults
/// - **Early Rejection**: Requests exceeding limits are rejected before full parsing
/// - **Memory Protection**: Prevents memory exhaustion from malicious large payloads
/// - **Standard Compliance**: Returns proper HTTP 413 (Payload Too Large) status codes
/// - **Integration-Friendly**: Works seamlessly with other body-processing middlewares
/// - **Performance-Optimized**: Minimal overhead for compliant requests
///
/// ## Security Benefits
///
/// - **DoS Protection**: Prevents denial-of-service attacks via large payloads
/// - **Memory Management**: Ensures predictable server memory usage patterns
/// - **Resource Conservation**: Protects server resources from abuse
/// - **Early Detection**: Rejects oversized requests before expensive processing
///
/// ## Default Behavior
///
/// - **Default Limit**: 1 MiB (1,048,576 bytes) when no configuration is provided
/// - **Status Code**: Returns HTTP 413 Payload Too Large for violations
/// - **Scope**: Applied globally to all routes unless path-specific configuration is used
/// - **Execution**: Runs as pre-middleware before route handlers and body parsing
///
/// ## Configuration Examples
///
/// ```rust
/// use ripress::app::App;
///
/// let mut app = App::new();
///
/// // Use default 1 MiB limit for all routes
/// app.use_body_limit(None);
///
/// // Custom limits for different scenarios
///
/// // Strict limit for API endpoints (100 KB)
/// app.use_body_limit(Some(100 * 1024));
///
/// // Generous limit for file upload endpoints (50 MB)
/// app.use_body_limit(Some(50 * 1024 * 1024));
///
/// // Very large limit for data processing endpoints (1 GB)
/// app.use_body_limit(Some(1024 * 1024 * 1024));
/// ```
///
/// ## Path-Specific Limits
///
/// You can apply different limits to different routes using custom middleware:
///
/// ```rust
/// use ripress::app::App;
///
/// let mut app = App::new();
///
/// // Default limit for most routes
/// app.use_body_limit(Some(1024 * 1024)); // 1 MB
/// ```
///
/// ## Common Size Guidelines
///
/// | Use Case | Recommended Limit | Reasoning |
/// |----------|-------------------|-----------|
/// | JSON APIs | 1 MB - 10 MB | Most JSON payloads are small |
/// | File Uploads | 10 MB - 1 GB | Depends on expected file types |
/// | Form Submissions | 1 MB - 5 MB | Text forms plus small files |
/// | Webhooks | 1 KB - 1 MB | Usually small notification payloads |
/// | Log Ingestion | 10 MB - 100 MB | May need larger limits for batch logs |
/// | Authentication | 1 KB - 10 KB | Credentials are typically small |
///
/// ## Error Response Format
///
/// When the limit is exceeded, the middleware returns:
///
/// ```text
/// HTTP/1.1 413 Payload Too Large
/// Content-Type: text/plain
/// Content-Length: 26
///
/// Request body too large
/// ```
///
/// For API applications, you might want to add custom error handling:
///
/// ```rust
/// use ripress::app::App;
///
/// let mut app = App::new();
///
/// // Custom body limit with better error handling
/// app.use_pre_middleware(None, |req, res| async move {
///     const MAX_SIZE: usize = 1024 * 1024; // 1 MB
///     
///     if let Some(content_length) = req.headers.get("content-length") {
///         if let Ok(size) = content_length.parse::<usize>() {
///             if size > MAX_SIZE {
///                 let error_response = res.status(413).json(serde_json::json!({
///                     "error": "payload_too_large",
///                     "message": "Request body exceeds maximum allowed size",
///                     "max_size": MAX_SIZE,
///                     "received_size": size
///                 }));
///                 return (req, Some(error_response));
///             }
///         }
///     }
///     
///     (req, None) // Continue processing
/// });
/// ```
///
/// ## Best Practices
///
/// ### Middleware Order
/// Place body limit middleware **before** any middleware that reads the request body:
///
/// ```rust
/// use ripress::app::App;
/// use ripress::middlewares::file_upload::file_upload;
///
/// let mut app = App::new();
///
/// // 1. Body limits (first)
/// app.use_body_limit(Some(10 * 1024 * 1024));
///
/// // 2. Then file upload handling
/// app.use_pre_middleware(Some("/upload"), file_upload(None));
///
/// // 3. Other body-processing middleware
/// // ...
/// ```
///
/// ### Progressive Limits
/// Use progressively stricter limits for more sensitive endpoints:
///
/// ```rust
/// use ripress::app::App;
///
/// // Global lenient limi
/// let mut app = App::new();
///
/// app.use_body_limit(Some(10 * 1024 * 1024)); // 10 MB
/// ```
///
/// ### Content-Type Considerations
/// Different content types have different typical sizes:
///
/// - **JSON**: Usually small (< 1 MB), but data imports can be larger
/// - **Form Data**: Moderate (1-10 MB), includes text and small files
/// - **File Uploads**: Large (10 MB - 1 GB+), depends on use case
/// - **Binary Data**: Variable, set limits based on expected data
///
/// ### Monitoring and Alerting
/// Monitor 413 responses to detect:
/// - Legitimate users hitting limits (increase limits)
/// - Attack patterns (implement additional protections)
/// - Application bugs (unexpected large payloads)
///
/// ## Integration with Load Balancers
///
/// When using load balancers or reverse proxies, ensure consistency:
///
/// ```nginx
/// # nginx configuration
/// client_max_body_size 10M;  # Should match or exceed your app's limit
/// ```
///
/// This prevents the proxy from rejecting requests before they reach your application.
///
/// ## Performance Impact
///
/// - **Overhead**: Minimal for compliant requests
/// - **Memory**: Does not buffer entire body, checks incrementally
/// - **CPU**: Negligible impact on request processing
/// - **Latency**: Early rejection reduces processing time for oversized requests
pub mod body_limit;

/// Response Compression middleware
///
/// This module provides intelligent response compression using gzip encoding to reduce
/// bandwidth usage and improve response times. The middleware automatically compresses
/// eligible responses based on content type, size, and client capabilities while
/// maintaining high performance and compatibility.
///
/// ## Features
///
/// - **Intelligent Compression**: Automatically determines when compression is beneficial
/// - **Content-Type Aware**: Only compresses text-based content types by default
/// - **Size Threshold**: Configurable minimum size to avoid compressing tiny responses
/// - **Client Negotiation**: Respects `Accept-Encoding` headers and client capabilities
/// - **Compression Levels**: Configurable gzip compression levels (0-9)
/// - **Header Management**: Automatically sets appropriate response headers
/// - **Performance Optimized**: Streaming compression to minimize memory usage
/// - **Cache-Friendly**: Proper `Vary` header handling for caching proxies
///
/// ## Compression Benefits
///
/// - **Bandwidth Reduction**: Typically 60-80% size reduction for text content
/// - **Faster Load Times**: Especially beneficial for users on slow connections
/// - **Cost Savings**: Reduced data transfer costs for cloud deployments
/// - **Better User Experience**: Faster page loads and API responses
/// - **SEO Benefits**: Page speed is a ranking factor for search engines
///
/// ## Default Behavior
///
/// - **Threshold**: Only compresses responses larger than 1 KiB (1,024 bytes)
/// - **Compression Level**: Level 6 (balanced speed/compression ratio)
/// - **Content Types**: Automatically compresses text-based content types
/// - **Client Support**: Only compresses when client supports gzip encoding
/// - **Header Handling**: Adds `Content-Encoding: gzip` and appropriate `Vary` headers
///
/// ## Configuration Examples
///
/// ```rust
/// use ripress::app::App;
/// use ripress::middlewares::compression::CompressionConfig;
///
/// let mut app = App::new();
///
/// // Use default compression settings (recommended for most applications)
/// app.use_compression(None);
///
/// // High compression for bandwidth-critical applications
/// app.use_compression(Some(CompressionConfig {
///     threshold: 512,          // Compress responses > 512 bytes
///     level: 9,                // Maximum compression (slower)
/// }));
///
/// // Fast compression for high-traffic applications
/// app.use_compression(Some(CompressionConfig {
///     threshold: 2048,         // Only compress larger responses
///     level: 1,                // Fast compression (less CPU usage)
/// }));
///
/// // Conservative compression for legacy clients
/// app.use_compression(Some(CompressionConfig {
///     threshold: 4096,         // Conservative threshold
///     level: 4,                // Moderate compression
/// }));
/// ```
///
/// ## Content Type Handling
///
/// ### Automatically Compressed
/// The middleware compresses these content types by default:
///
/// - `text/*` (HTML, CSS, JavaScript, plain text)
/// - `application/json`
/// - `application/xml`
/// - `application/javascript`
/// - `application/x-javascript`
/// - `image/svg+xml`
/// - `application/rss+xml`
/// - `application/atom+xml`
///
/// ### Never Compressed
/// These content types are automatically excluded:
///
/// - `image/*` (JPEG, PNG, GIF - already compressed)
/// - `video/*` (MP4, WebM - already compressed)
/// - `audio/*` (MP3, AAC - already compressed)
/// - `application/zip`, `application/gzip`
/// - `application/pdf`
///
/// ### Custom Content Type Configuration
/// ```rust
/// use ripress::middlewares::compression::CompressionConfig;
/// let _cfg = CompressionConfig { threshold: 2048, level: 6 };
/// ```
///
/// ## Compression Level Guidelines
///
/// | Level | Speed | Compression Ratio | Use Case |
/// |-------|-------|------------------|----------|
/// | 1 | Fastest | ~60% reduction | High-traffic APIs, real-time apps |
/// | 3 | Fast | ~65% reduction | General web applications |
/// | 6 | Balanced | ~70% reduction | **Default** - good for most cases |
/// | 9 | Slowest | ~75% reduction | Static assets, bandwidth-critical |
///
/// ## Performance Considerations
///
/// ### CPU vs. Bandwidth Trade-off
/// ```rust
/// use ripress::middlewares::compression::CompressionConfig;
/// // For high-CPU, unlimited bandwidth environments
/// let _cfg_fast = CompressionConfig { level: 1, threshold: 8192 };
/// // For limited bandwidth, adequate CPU environments
/// let _cfg_tight = CompressionConfig { level: 8, threshold: 256 };
/// ```
///
/// ### Memory Usage
/// The middleware uses streaming compression to minimize memory usage:
///
/// ```rust
/// use ripress::middlewares::compression::CompressionConfig;
/// // Streaming is internal; configure via threshold/level.
/// let _cfg = CompressionConfig { threshold: 4096, level: 6 };
/// ```
///
/// ## Caching Integration
///
/// The middleware handles caching correctly by setting appropriate headers:
///
/// ```text
/// Content-Encoding: gzip
/// Vary: Accept-Encoding
/// ```
///
/// ### CDN and Proxy Configuration
/// ```rust
/// use ripress::middlewares::compression::CompressionConfig;
/// // Ensure proxies preserve Accept-Encoding; compression is negotiated per-request.
/// let _cfg = CompressionConfig::default();
/// ```
///
/// ## Route-Specific Compression
///
/// Apply different compression strategies to different routes:
///
/// ```rust
/// use ripress::app::App;
/// use ripress::middlewares::compression::CompressionConfig;
///
/// let mut app = App::new();
///
/// // Default compression for most routes
/// app.use_compression(Some(CompressionConfig { level: 6, threshold: 1024 }));
///
/// // Route-specific skip example (identity)
/// app.use_post_middleware(Some("/files"), |req, res| async move {
///     (req, Some(res.set_header("Content-Encoding", "identity")))
/// });
/// ```
///
/// ## Browser Compatibility
///
/// - **Modern Browsers**: Full gzip support
/// - **Internet Explorer 6+**: Supported with proper headers
/// - **Mobile Browsers**: Universally supported
/// - **HTTP/1.1 Clients**: Required to support gzip per RFC
///
/// ## Security Considerations
///
/// ### BREACH Attack Mitigation
/// When compressing responses that include user input and secrets:
///
/// ```rust
/// use ripress::app::App;
/// use ripress::middlewares::compression::CompressionConfig;
///
/// let mut app = App::new();
///
/// app.use_compression(Some(CompressionConfig {
///     ..Default::default()
/// }));
/// ```
///
/// ### Content-Type Validation
/// The middleware validates content types to prevent compression of sensitive binary content:
///
/// ```
/// use ripress::app::App;
/// use ripress::middlewares::compression::CompressionConfig;
///
/// let mut app = App::new();
///
/// app.use_compression(Some(CompressionConfig {
///     ..Default::default()
/// }));
/// ```
pub mod compression;

/// Comprehensive Security Headers (Shield) middleware
///
/// This module provides a comprehensive security middleware that sets multiple HTTP security
/// headers to protect web applications from various attacks and vulnerabilities. The shield
/// middleware implements defense-in-depth principles by applying multiple layers of security
/// controls through HTTP headers.
///
/// ## Security Headers Overview
///
/// The shield middleware can set the following security headers to protect your application:
///
/// ### Core Security Headers
/// - **Content Security Policy (CSP)**: Prevents XSS, code injection, and other attacks
/// - **HTTP Strict Transport Security (HSTS)**: Enforces HTTPS connections
/// - **X-Frame-Options**: Prevents clickjacking attacks (legacy fallback)
/// - **X-Content-Type-Options**: Prevents MIME type sniffing attacks
/// - **Referrer-Policy**: Controls referrer information leakage
///
/// ### Modern Security Headers
/// - **Permissions-Policy**: Controls browser features and APIs access
/// - **Cross-Origin-Embedder-Policy (COEP)**: Enables cross-origin isolation
/// - **Cross-Origin-Opener-Policy (COOP)**: Isolates browsing context groups
/// - **Cross-Origin-Resource-Policy (CORP)**: Controls cross-origin resource sharing
/// - **Origin-Agent-Cluster**: Requests origin-keyed agent clustering
///
/// ### Legacy/Compatibility Headers
/// - **X-XSS-Protection**: Legacy XSS filtering (disabled by default)
/// - **X-Permitted-Cross-Domain-Policies**: Flash/Silverlight policy control
/// - **X-Download-Options**: Internet Explorer download behavior
/// - **X-DNS-Prefetch-Control**: DNS prefetching control
///
/// ## Default Configuration
///
/// The shield middleware applies secure defaults suitable for most web applications:
///
/// ```rust
/// use ripress::app::App;
///
/// let mut app = App::new();
///
/// // Apply secure defaults (recommended for most applications)
/// app.use_shield(None);
/// ```
///
/// Default settings include:
/// - **CSP**: Restrictive policy with `'self'` as default source
/// - **HSTS**: 1 year max-age with subdomain inclusion (HTTPS only)
/// - **Frame Options**: `DENY` to prevent all framing
/// - **Content Type Options**: `nosniff` to prevent MIME sniffing
/// - **Referrer Policy**: `strict-origin-when-cross-origin`
///
/// ## Custom Configuration Examples
///
/// ### Basic Web Application
/// ```rust
/// use ripress::app::App;
/// use ripress::middlewares::shield::{ShieldConfig, Hsts, ContentSecurityPolicy};
/// use std::collections::HashMap;
///
/// let mut app = App::new();
///
/// let mut directives = HashMap::new();
/// directives.insert("default-src".to_string(), "'self'".to_string());
/// directives.insert("script-src".to_string(), "'self' https://cdn.jsdelivr.net https://unpkg.com".to_string());
/// directives.insert("style-src".to_string(), "'self' 'unsafe-inline' https://fonts.googleapis.com".to_string());
/// directives.insert("img-src".to_string(), "'self' data: https:".to_string());
/// directives.insert("font-src".to_string(), "'self' https://fonts.gstatic.com".to_string());
///
/// app.use_shield(Some(ShieldConfig {
///     hsts: Hsts { enabled: true, max_age: 31536000, include_subdomains: true, preload: false },
///     content_security_policy: ContentSecurityPolicy { enabled: true, directives, report_only: false },
///     ..Default::default()
/// }));
/// ```
///
/// ### API-Only Application
/// ```rust
/// use ripress::middlewares::shield::{ShieldConfig, ContentSecurityPolicy, Hsts};
/// use std::collections::HashMap;
/// use ripress::app::App;
///
/// let mut app = App::new();
///
/// let mut directives = HashMap::new();
/// directives.insert("default-src".to_string(), "'none'".to_string());
///
/// app.use_shield(Some(ShieldConfig {
///     hsts: Hsts { enabled: true, max_age: 63072000, include_subdomains: true, preload: true },
///     content_security_policy: ContentSecurityPolicy { enabled: true, directives, report_only: false },
///     ..Default::default()
/// }));
/// ```
///
/// ### Development-Friendly Configuration
/// ```rust
/// use ripress::middlewares::shield::{ShieldConfig, ContentSecurityPolicy, Hsts};
/// use std::collections::HashMap;
/// use ripress::app::App;
///
/// let mut app = App::new();
///
/// let mut directives = HashMap::new();
/// directives.insert("default-src".to_string(), "'self' 'unsafe-inline'".to_string());
/// directives.insert("script-src".to_string(), "'self' 'unsafe-eval'".to_string());
///
/// app.use_shield(Some(ShieldConfig {
///     content_security_policy: ContentSecurityPolicy { enabled: true, directives, report_only: true },
///     hsts: Hsts { enabled: false, ..Default::default() },
///     ..Default::default()
/// }));
/// ```
///
/// ## Content Security Policy (CSP) Examples
///
/// ### Strict CSP for High-Security Applications
/// ```rust
/// use std::collections::HashMap;
/// use ripress::middlewares::shield::ContentSecurityPolicy;
/// let mut directives = HashMap::new();
/// directives.insert("default-src".to_string(), "'none'".to_string());
/// directives.insert("script-src".to_string(), "'self' 'nonce-{NONCE}'".to_string());
/// directives.insert("style-src".to_string(), "'self' 'nonce-{NONCE}'".to_string());
/// directives.insert("img-src".to_string(), "'self' data:".to_string());
/// directives.insert("connect-src".to_string(), "'self'".to_string());
/// directives.insert("font-src".to_string(), "'self'".to_string());
/// directives.insert("object-src".to_string(), "'none'".to_string());
/// directives.insert("media-src".to_string(), "'self'".to_string());
/// directives.insert("frame-src".to_string(), "'none'".to_string());
/// let _csp = ContentSecurityPolicy { enabled: true, directives, report_only: false };
/// ```
///
/// ### CSP for Single Page Applications (SPA)
/// ```rust
/// use std::collections::HashMap;
/// use ripress::middlewares::shield::ContentSecurityPolicy;
/// let mut directives = HashMap::new();
/// directives.insert("default-src".to_string(), "'self'".to_string());
/// directives.insert("script-src".to_string(), "'self' https://api.example.com".to_string());
/// directives.insert("style-src".to_string(), "'self' 'unsafe-inline'".to_string());
/// directives.insert("img-src".to_string(), "'self' https://images.example.com data:".to_string());
/// directives.insert("connect-src".to_string(), "'self' https://api.example.com wss://websocket.example.com".to_string());
/// let _csp = ContentSecurityPolicy { enabled: true, directives, report_only: false };
/// ```
///
/// ## HSTS Configuration Guidelines
///
/// ### Progressive HSTS Rollout
/// ```rust
/// use ripress::middlewares::shield::Hsts;
/// // Phase 1
/// let _h1 = Hsts { enabled: true, max_age: 86400, include_subdomains: false, preload: false };
/// // Phase 2
/// let _h2 = Hsts { enabled: true, max_age: 31536000, include_subdomains: true, preload: false };
/// // Phase 3
/// let _h3 = Hsts { enabled: true, max_age: 63072000, include_subdomains: true, preload: true };
/// ```
///
/// ## Cross-Origin Policies
///
/// ### Enabling Cross-Origin Isolation
/// ```rust
/// use ripress::app::App;
/// use ripress::  middlewares::shield::{CrossOriginEmbedderPolicy, CrossOriginOpenerPolicy, CrossOriginResourcePolicy, ShieldConfig};
///
/// let mut app = App::new();
///
/// app.use_shield(Some(ShieldConfig {
///     cross_origin_embedder_policy: CrossOriginEmbedderPolicy::default(),
///     cross_origin_opener_policy: CrossOriginOpenerPolicy::default(),
///     cross_origin_resource_policy: CrossOriginResourcePolicy::default(),
///     ..Default::default()
/// }));
/// ```
///
/// **Note**: Cross-origin isolation enables powerful browser features like `SharedArrayBuffer`
/// but requires all cross-origin resources to have appropriate CORP headers.
///
/// ## Security Best Practices
///
/// ### Implementation Order
/// 1. **Start with Report-Only**: Use CSP report-only mode to identify violations
/// 2. **Gradual Enforcement**: Gradually tighten policies based on reports
/// 3. **Monitor Violations**: Set up CSP violation reporting and monitoring
/// 4. **Regular Updates**: Keep security headers updated with evolving threats
/// 5. **Test Thoroughly**: Test all functionality with security headers enabled
///
/// ### Header Precedence
/// The shield middleware should be applied early in the middleware chain:
///
/// ```no_run
/// use ripress::app::App;
///
/// let mut app = App::new();
///
/// // 1. Security headers first
/// app.use_shield(None);
///
/// // 2. Then CORS (may override some headers)
/// app.use_cors(None);
///
/// // 3. Other middleware...
/// app.use_rate_limiter(None);
/// ```
///
/// ### Environment-Specific Configuration
/// ```rust
/// use ripress::app::App;
/// use ripress::middlewares::shield::{ShieldConfig, Hsts, ContentSecurityPolicy};
/// use std::collections::HashMap;
///
/// let mut app = App::new();
///
/// let shield_config = if cfg!(debug_assertions) {
///     let mut directives = HashMap::new();
///     directives.insert("default-src".to_string(), "'self'".to_string());
///     ShieldConfig {
///         content_security_policy: ContentSecurityPolicy { report_only: true, enabled: true, directives },
///         hsts: Hsts { enabled: false, ..Default::default() },
///         ..Default::default()
///     }
/// } else {
///     ShieldConfig::default()
/// };
///
/// app.use_shield(Some(shield_config));
/// ```
///
/// ## Common Pitfalls and Solutions
///
/// ### CSP Breaking Third-Party Scripts
/// ```rust
/// // Add specific domains to script-src in the CSP directives map
/// ```
///
/// ### HSTS on Non-HTTPS Development
/// ```rust
/// use ripress::app::App;
/// use ripress::middlewares::shield::{Hsts, ShieldConfig};
///
/// let mut app = App::new();
///
/// // Problem: HSTS breaks HTTP development
/// // Solution: Disable HSTS in development
/// app.use_shield(Some(ShieldConfig {
///     hsts: Hsts {
///         enabled: !cfg!(debug_assertions), // Only enable in release builds
///         ..Default::default()
///     },
///     ..Default::default()
/// }));
/// ```
///
/// ### Frame Options vs CSP Conflict
/// ```rust
/// // Prefer CSP frame-ancestors via ResponseHeaders helpers
/// ```
///
/// ## Monitoring and Compliance
///
/// ### CSP Violation Reporting
/// ```rust
/// use ripress::app::App;
/// use ripress::types::RouterFns;
///
/// let mut app = App::new();
/// app.post("/csp-report", |req, res| async move {
///     if let Ok(violation_report) = req.json::<serde_json::Value>() {
///         eprintln!("CSP Violation: {}", violation_report);
///     }
///     res.ok().text("OK")
/// });
/// ```
///
/// ### Security Headers Validation
/// Use tools like securityheaders.com or Mozilla Observatory to validate your configuration.
pub mod shield;

use crate::{
    context::{HttpRequest, HttpResponse},
    types::FutMiddleware,
};
use std::sync::Arc;

// #[cfg(feature = "with-wynd")]
// use crate::types::WyndMiddlewareHandler;

/// Represents a middleware in the Ripress application.
///
/// A `Middleware` consists of a function and an associated path. The function is an
/// asynchronous closure or function that takes an [`HttpRequest`] and [`HttpResponse`],
/// and returns a future resolving to a tuple of the potentially modified request and an
/// optional response. If the middleware returns `Some(response)`, the response is sent
/// immediately and further processing is halted. If it returns `None`, the request
/// continues through the middleware chain and to the route handler.
///
/// The `path` field specifies the route prefix or pattern for which this middleware
/// should be applied. Middlewares are matched in the order they are added to the app.
///
/// ## Middleware Execution Order
///
/// 1. Pre-middlewares (in registration order)
/// 2. Route handler
/// 3. Post-middlewares (in registration order)
#[derive(Clone)]
pub(crate) struct Middleware {
    /// The middleware function.
    ///
    /// This is an `Arc`-wrapped closure or function pointer that takes an [`HttpRequest`]
    /// and [`HttpResponse`], and returns a boxed future resolving to a tuple of the
    /// (possibly modified) request and an optional response.
    pub func: Arc<dyn Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static>,

    /// The path or route prefix this middleware applies to.
    ///
    /// If the incoming request path starts with this string, the middleware will be invoked.
    /// Use "/" to apply to all routes.
    pub path: String,

    /// The type of middleware (Pre or Post execution).
    pub(crate) middleware_type: MiddlewareType,
}

/// Defines when a middleware should be executed in the request lifecycle.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum MiddlewareType {
    /// Middleware executed before the route handler.
    Pre,
    /// Middleware executed after the route handler.
    Post,
}

// #[cfg(feature = "with-wynd")]
// /// WebSocket middleware container for the Wynd WebSocket implementation.
// ///
// /// This struct holds the WebSocket handler and the path it should be mounted on.
// /// Only available when the `with-wynd` feature is enabled.
// #[derive(Clone)]
// pub(crate) struct WyndMiddleware {
//     /// The WebSocket handler function.
//     pub func: WyndMiddlewareHandler,
//     /// The path where WebSocket connections should be accepted.
//     pub path: String,
// }
