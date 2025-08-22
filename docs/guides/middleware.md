# Middleware Documentation

## File Upload Middleware

The file upload middleware handles file uploads by processing request bodies and saving them to a specified upload directory. It supports raw binary uploads and multipart form data with automatic file type detection and unique filename generation.

### Features

- **Automatic file extension detection** using the `infer` crate
- **Unique filename generation** with UUIDs to prevent conflicts
- **Graceful error handling** - continues request processing even if upload fails
- **Configurable upload directory** with fallback to "uploads"
- **Non-blocking operation** - doesn't short-circuit requests on upload failures
- **Supports multipart/form-data** - extracts ALL file parts and text fields
- **Multiple file support** - handles multiple files in a single request

### Usage

```rust
use ripress::{app::App, middlewares::file_upload::file_upload};

// Use default "uploads" directory
let mut app = App::new();
app.use_middleware("/upload", file_upload(None));

// Use custom directory
app.use_middleware("/files", file_upload(Some("user_files")));
```

### How File Processing Works

The middleware processes requests as follows:

1. **Content Detection**: Attempts to read the raw request body
2. **Multipart Parsing**: If Content-Type is `multipart/form-data`, extracts all parts
3. **Text Field Extraction**: Adds text fields to `req.form_data()`
4. **File Processing**: Saves all file content with UUID filenames and detected extensions
5. **Field Mapping**: Maps file input field names to generated UUID filenames in `req.form_data()`
6. **Data Injection**: Sets comprehensive file information in request data
7. **Error Handling**: Logs errors but continues request processing

### Form Field Behavior

**For multipart forms:**

- **Text fields**: Available via `req.form_data()` with original names and values
- **File fields**: Field names are mapped to generated UUID filenames in `req.form_data()`
- **Example**: `<input name="profile_pic" type="file">` → `req.form_data().get("profile_pic")` returns UUID filename like `"a1b2c3d4-e5f6-7890-abcd-ef1234567890.jpg"`

**For single binary uploads:**

- Uses "file" as the default field name
- No original filename preservation

### Request Data Available After Upload

**In req.get_data() (not form_data):**

- `uploaded_file_count` - Number of files successfully uploaded (as string)
- `uploaded_files` - JSON array of file information
- `uploaded_file` - First file's UUID filename (backwards compatibility)
- `uploaded_file_path` - First file's full path (backwards compatibility)
- `original_filename` - First file's original name if available from multipart

**In req.form_data():**

- Text field names → their string values
- File field names → their generated UUID filenames (strings)

### Examples

Processing uploaded files in a route handler:

```rust
async fn upload_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // Check if files were uploaded
    if let Some(count_str) = req.get_data("uploaded_file_count") {
        let count: usize = count_str.parse().unwrap_or(0);

        if count > 0 {
            // Get detailed file information
            if let Some(files_json) = req.get_data("uploaded_files") {
                res.ok().text(format!("Uploaded {} files: {}", count, files_json))
            } else {
                // Access individual file info (backwards compatibility)
                let filename = req.get_data("uploaded_file").unwrap_or("unknown");
                let path = req.get_data("uploaded_file_path").unwrap_or("unknown");
                res.ok().text(format!("Uploaded file: {} at {}", filename, path))
            }
        } else {
            res.ok().text("No files were uploaded")
        }
    } else {
        res.ok().text("Upload processing not completed")
    }
}

// Accessing form fields (including file field mappings)
async fn form_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // Get text fields
    if let Some(username) = req.form_data().get("username") {
        println!("Username: {}", username);
    }

    // Get file field mapping (returns UUID filename)
    if let Some(avatar_filename) = req.form_data().get("avatar") {
        println!("Avatar saved as: {}", avatar_filename);
    }

    res.ok().text("Form processed")
}
```

### Current Limitations

1. **Original filename preservation**: Due to tuple handling in the code, original filenames from multipart forms are not properly preserved in individual file processing
2. **Single binary uploads**: Always use "file" as the field name, no original filename
3. **Raw body access**: Requires successful `req.bytes()` call - may fail for some request types
4. **Field mapping**: Only maps field names to UUID filenames; original filename info is not available in form_data

### Error Handling

The middleware is designed to be non-blocking:

- **Upload failures**: Logged to stderr but don't stop request processing
- **Directory creation failures**: Logged but allow the request to continue
- **Body reading failures**: Logged but request continues normally
- **File write failures**: Logged per-file but don't short-circuit the request
- **Multipart parsing errors**: Gracefully fall back to single binary processing

---

## Rate Limiter Middleware

The Rate Limiter middleware implements a sliding window rate limiting algorithm that controls the number of requests clients can make within a specified time period. It's essential for protecting APIs from abuse, preventing DoS attacks, and ensuring fair resource usage across clients.

### Features

- **Sliding window algorithm** - More accurate than fixed windows, prevents burst traffic at window boundaries
- **Per-client tracking** - Individual rate limits based on client IP addresses
- **Proxy support** - Extracts real client IPs from `X-Forwarded-For` headers
- **Automatic cleanup** - Periodic cleanup of expired entries to prevent memory leaks
- **Standard headers** - Follows RFC-compliant rate limit headers for client guidance
- **Configurable responses** - Custom messages and limits for different use cases
- **Thread-safe** - Uses async mutexes for safe concurrent access
- **Memory efficient** - Lightweight tracking structure per client
- **Graceful degradation** - Continues operation even under high load

### Configuration

The `RateLimiterConfig` struct allows you to customize rate limiting behavior:

- `window_ms` - Duration of the rate limiting window (default: 10 seconds)
- `max_requests` - Maximum requests allowed per client per window (default: 10)
- `proxy` - Whether to extract real IP from proxy headers (default: false)
- `message` - Custom message returned when limit exceeded (default: "Too many requests")

### Usage

```rust
use ripress::{app::App, middlewares::rate_limiter::RateLimiterConfig};
use std::time::Duration;

// Use default rate limiting (10 requests per 10 seconds)
let mut app = App::new();
app.use_rate_limiter(None);

// API rate limiting for production (100 requests per minute)
let mut app = App::new();
let config = RateLimiterConfig {
    window_ms: Duration::from_secs(60), // 1 minute window
    max_requests: 100,
    proxy: true, // Behind load balancer
    message: "Rate limit exceeded. Please try again later.".to_string(),
};
app.use_rate_limiter(Some(config));

// Strict rate limiting for sensitive endpoints
let config = RateLimiterConfig {
    window_ms: Duration::from_secs(300), // 5 minute window
    max_requests: 5, // Very restrictive
    proxy: false,
    message: "Too many attempts. Please wait before trying again.".to_string(),
};
app.use_rate_limiter(Some(config));
```

### How It Works

The Rate Limiter middleware:

1. **Identifies client** by IP address (direct or via proxy headers)
2. **Tracks requests** in a sliding time window per client
3. **Rejects excess requests** with 429 Too Many Requests status
4. **Adds standard headers** to guide client retry behavior
5. **Cleans up expired data** automatically every 5 minutes
6. **Continues processing** for requests within limits

### Client Identification

**Direct Connection Mode (`proxy: false`)**

- Uses the direct client IP address from the TCP connection
- Suitable for applications directly facing the internet
- Most accurate when clients connect directly

**Proxy Mode (`proxy: true`)**

- Extracts the real client IP from `X-Forwarded-For` header
- Falls back to direct IP if header is missing or malformed
- Essential when behind load balancers, CDNs, or reverse proxies
- Takes the first IP from comma-separated list (closest to client)

### Response Headers

The middleware adds standard rate limiting headers to all responses:

- `X-RateLimit-Limit` - The maximum number of requests allowed in the window
- `X-RateLimit-Remaining` - Number of requests remaining in current window
- `X-RateLimit-Reset` - Seconds until the current window resets
- `Retry-After` - Seconds to wait before retrying (only when rate limited)

### Rate Limiting Algorithm

The middleware uses a **sliding window** approach:

1. **First request** from a client starts a new window
2. **Subsequent requests** within the window are counted against the limit
3. **Window expiry** resets the counter and starts a new window
4. **Requests over limit** are rejected with 429 status until window resets

This approach is more accurate than fixed windows because it doesn't allow burst traffic at window boundaries.

### Examples

Different rate limiting strategies:

```rust
// Development-friendly configuration
let config = RateLimiterConfig {
    window_ms: Duration::from_secs(10),
    max_requests: 1000, // Very permissive for development
    proxy: false,
    message: "Development rate limit exceeded".to_string(),
};

// Authentication endpoint protection
let auth_config = RateLimiterConfig {
    window_ms: Duration::from_secs(300), // 5 minutes
    max_requests: 5,
    proxy: true,
    message: "Too many login attempts. Please wait 5 minutes.".to_string(),
};

// Public API with generous limits
let api_config = RateLimiterConfig {
    window_ms: Duration::from_secs(60),
    max_requests: 200,
    proxy: true,
    message: "API rate limit exceeded. See headers for retry timing.".to_string(),
};
```

### Best Practices

- Set `window_ms` based on your API's usage patterns
- Use generous limits initially and adjust based on monitoring
- Enable `proxy: true` when behind load balancers or CDNs
- Provide clear error messages to guide client behavior
- Monitor rate limit violation patterns
- Consider different limits for different endpoint types

### Limitations

- Rate limits are per-application instance (not shared across load-balanced instances)
- All client data stored in memory (not suitable for millions of unique clients)
- IP-based limiting may affect multiple users behind NAT/proxy
- Background cleanup runs every 5 minutes

---

## Body Size Limit Middleware

The Body Size Limit middleware protects your application from excessively large request bodies that could consume server resources or cause denial-of-service attacks. It checks the incoming request body size and rejects requests that exceed the configured limit.

### Features

- **Configurable size limits** - Set custom maximum body sizes per route or globally
- **Early rejection** - Rejects oversized requests before processing
- **Detailed error responses** - JSON error messages with size information
- **Memory protection** - Prevents memory exhaustion from large uploads
- **Performance optimized** - Lightweight check with minimal overhead
- **Standards compliant** - Returns proper HTTP 413 Payload Too Large status

### Configuration

The middleware accepts an optional maximum size in bytes:

- If `None` is provided, the default limit is 1 MB (1,048,576 bytes)
- Custom limits can be set based on your application's needs
- Different limits can be applied to different routes

### Usage

```rust
use ripress::app::App;

// Use default 1 MB limit
let mut app = App::new();
app.use_body_limit(None);

// Set custom limit (2 MB for file uploads)
app.use_body_limit(Some(2 * 1024 * 1024));

// Different limits for different endpoints
app.use_body_limit_on("/api/upload", Some(10 * 1024 * 1024)); // 10 MB for uploads
app.use_body_limit_on("/api/data", Some(100 * 1024));         // 100 KB for data API
```

### How It Works

The Body Size Limit middleware:

1. **Checks request body size** against the configured limit
2. **Allows normal processing** if body is within the limit
3. **Rejects immediately** if body exceeds the limit
4. **Returns detailed error** with size information
5. **Logs violation** for monitoring and debugging
6. **Prevents resource exhaustion** by early termination

### Error Response

When the body size limit is exceeded, the middleware returns a `413 Payload Too Large` response with a detailed JSON error message:

```json
{
  "error": "Request body too large",
  "message": "Request body exceeded the configured limit of 1048576 bytes",
  "limit": 1048576,
  "received": 2097152
}
```

### Common Size Limits

Choose appropriate limits based on your use case:

```rust
// Small data APIs (JSON payloads)
app.use_body_limit(Some(64 * 1024)); // 64 KB

// Standard web forms
app.use_body_limit(Some(1024 * 1024)); // 1 MB (default)

// File upload endpoints
app.use_body_limit(Some(50 * 1024 * 1024)); // 50 MB

// Image upload services
app.use_body_limit(Some(10 * 1024 * 1024)); // 10 MB

// Document upload services
app.use_body_limit(Some(100 * 1024 * 1024)); // 100 MB

// Video upload (for chunked uploads, consider streaming)
app.use_body_limit(Some(500 * 1024 * 1024)); // 500 MB
```

### Use Cases

**API Protection**

```rust
// Protect JSON APIs from oversized payloads
app.use_body_limit_on("/api/*", Some(256 * 1024)); // 256 KB for all API routes
```

**File Upload Services**

```rust
// Different limits for different file types
app.use_body_limit_on("/upload/images", Some(5 * 1024 * 1024));     // 5 MB for images
app.use_body_limit_on("/upload/documents", Some(20 * 1024 * 1024)); // 20 MB for documents
app.use_body_limit_on("/upload/videos", Some(100 * 1024 * 1024));   // 100 MB for videos
```

**Form Processing**

```rust
// Standard web forms with file attachments
app.use_body_limit_on("/forms/*", Some(2 * 1024 * 1024)); // 2 MB for form submissions
```

### Security Considerations

**Memory Protection**

- Large request bodies can exhaust server memory
- Set limits based on available system resources
- Monitor memory usage under load

**Denial of Service Prevention**

- Prevents attackers from sending extremely large requests
- Combines well with rate limiting for comprehensive protection
- Early rejection reduces processing overhead

**Resource Planning**

- Consider concurrent request limits when setting body size limits
- Account for temporary memory usage during request processing
- Plan for peak usage scenarios

### Best Practices

**Configuration Guidelines**

- Start with conservative limits and increase based on monitoring
- Set different limits for different endpoint types
- Consider your server's memory constraints
- Test limits with realistic payloads

**Monitoring and Alerting**

- Log body size limit violations for security monitoring
- Track patterns in rejected requests
- Monitor server memory usage
- Set up alerts for unusual rejection patterns

**Client Integration**

- Provide clear error messages to help clients understand limits
- Document size limits in API documentation
- Implement client-side validation where possible
- Consider chunked upload for large files

### Performance Impact

The middleware has minimal performance overhead:

- **Single comparison** operation per request
- **No body parsing** - works with raw body length
- **Early termination** - rejects oversized requests immediately
- **Memory efficient** - doesn't load oversized bodies into memory

### Integration with Other Middleware

**Order Matters**

```rust
// Place body limit early in middleware chain
app.use_rate_limiter(None);           // 1. Rate limiting first
app.use_body_limit(Some(1024 * 1024)); // 2. Body size check second
app.use_cors(None);                    // 3. CORS after security checks
```

**With File Uploads**

```rust
// Body limit should come before file upload processing
app.use_body_limit_on("/upload", Some(10 * 1024 * 1024)); // 10 MB limit
app.use_file_upload_on("/upload", Some("uploads"));        // File processing after limit check
```

### Error Handling

The middleware provides comprehensive error information:

- **HTTP Status**: 413 Payload Too Large (RFC compliant)
- **Error Details**: JSON response with limit and received size
- **Logging**: Errors logged to stderr for monitoring
- **Client Guidance**: Clear error messages for API consumers

---

## CORS Middleware

The CORS (Cross-Origin Resource Sharing) middleware handles CORS headers and preflight requests to control which origins can access your resources.

### Configuration

The `CorsConfig` struct allows you to customize CORS behavior:

- `allowed_origin` - The allowed origin for requests (default: "\*")
- `allowed_methods` - The allowed HTTP methods (default: "GET, POST, PUT, DELETE, OPTIONS, HEAD")
- `allowed_headers` - The allowed headers (default: "Content-Type, Authorization")
- `allow_credentials` - Whether to allow credentials (default: false)

### Usage

```rust
use ripress::app::App;
use ripress::middlewares::cors::CorsConfig;

// Use default CORS settings
let mut app = App::new();
app.use_cors(None);

// Use custom CORS settings
use ripress::middlewares::cors::{cors, CorsConfig};
app.Some(CorsConfig {
    allowed_origin: "https://example.com",
    allowed_methods: "GET, POST, PUT, DELETE, OPTIONS, HEAD",
    allowed_headers: "Content-Type, Authorization",
    allow_credentials: true,
});
```

### How It Works

The CORS middleware:

1. **Adds CORS headers** to all responses based on configuration
2. **Handles preflight requests** - automatically responds to OPTIONS requests with a 200 status
3. **Continues processing** for all non-OPTIONS requests after adding headers
4. **Sets credentials header** if `allow_credentials` is true

### Default Configuration

When using `app.use_cors(None)`, the middleware applies these defaults:

- **Origin**: `*` (allow all origins)
- **Methods**: `GET, POST, PUT, DELETE, OPTIONS, HEAD`
- **Headers**: `Content-Type, Authorization`
- **Credentials**: `false`

### Headers Added

The middleware automatically adds these headers to responses:

- `Access-Control-Allow-Origin`
- `Access-Control-Allow-Methods`
- `Access-Control-Allow-Headers`
- `Access-Control-Allow-Credentials` (if enabled)

### Preflight Handling

For OPTIONS requests (preflight), the middleware:

- Adds all CORS headers
- Returns a 200 OK response immediately
- Does not continue to other handlers

For all other requests:

- Adds CORS headers
- Continues to the next middleware or route handler

---

## Logger Middleware

The logger middleware logs HTTP request information for debugging and monitoring purposes.

### Configuration

The `LoggerConfig` struct controls what information gets logged:

- `method` - Whether to log the HTTP method (default: true)
- `path` - Whether to log the request path (default: true)
- `status` - Whether to log the response status code (default: true)
- `user_agent` - Whether to log the user agent (default: true)
- `ip` - Whether to log the IP address (default: true)
- `headers` - Which headers to log (default: empty)
- `body_size` - Whether to log the body size (default: true)
- `query_params` - Whether to log the query parameters (default: true)
- `exclude_paths` - Paths to exclude from logging (default: empty)

### Usage

```rust
use ripress::{app::App, middlewares::logger::LogerConfig};

// Use default logging (logs method, path, and duration)
let mut app = App::new();
app.use_logger(None);

// Use custom logging configuration
app.use_logger(Some(LoggerConfig {
    duration: true,
    method: true,
    path: false, // Don't log the path
    ..Default::default()
}));
```

### How It Works

The logger middleware:

1. **Records start time** when the request begins
2. **Captures request details** (method, path) from the request
3. **Continues processing** - doesn't interrupt the request flow
4. **Calculates duration** after processing
5. **Prints log information** to stdout based on configuration

### Log Format

The logger outputs information in this format:

```
path: /api/users, Time taken: 45ms, method: GET
```

The order and presence of fields depends on your configuration:

- If `path` is true: shows "path: {path}"
- If `duration` is true: shows "Time taken: {ms}ms"
- If `method` is true: shows "method: {method}"

### Default Configuration

When using `app.use_logger(None)`, all logging options are enabled:

- **Method**: true
- **Path**: true
- **Duration**: true

### Performance Impact

The logger middleware:

- Uses `std::time::Instant` for precise duration measurement
- Performs minimal string operations
- Does not block request processing
- Outputs synchronously to stdout

### Examples

Different configuration examples:

```rust
// Log everything (default)
app.use_logger(None);

// Only log duration and method
app.use_logger(Some(LoggerConfig {
    method: true,
    path: false,
    ..Default::default()
}));
```
