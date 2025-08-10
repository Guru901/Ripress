# ResponseHeaders Documentation

A comprehensive HTTP response header management system for Rust web applications with built-in security, CORS, and content handling features.

## Overview

`ResponseHeaders` provides a powerful interface for managing HTTP response headers with case-insensitive key handling, multi-value support, and specialized methods for common web development patterns including security headers, CORS configuration, content type management, and caching directives.

## Features

- **Case-insensitive**: All header names are automatically converted to lowercase
- **Multi-value support**: Handles headers with multiple values (e.g., `Set-Cookie`)
- **Security-focused**: Built-in methods for essential security headers
- **CORS support**: Complete Cross-Origin Resource Sharing configuration
- **Content management**: Convenient methods for common content types and dispositions
- **Builder pattern**: Fluent API for chaining header operations
- **Production-ready**: Security best practices built-in

## Basic Usage

```rust
use your_crate::ResponseHeaders;

// Create new headers
let mut headers = ResponseHeaders::new();

// Set basic headers
headers.content_type("application/json");
headers.content_length(1024);

// Set security headers
headers.security_headers();

// Set CORS headers
headers.cors_simple(Some("https://example.com"));

// Use builder pattern
let headers = ResponseHeaders::new()
    .with_content_type("text/html")
    .with_security()
    .with_cors(None);
```

## API Reference

### Construction

#### `new() -> Self`

Creates a new empty `ResponseHeaders` collection.

```rust
let headers = ResponseHeaders::new();
```

#### `from_static_map(map: HashMap<&'static str, &'static str>) -> Self`

Creates `ResponseHeaders` from a static string `HashMap` (for backward compatibility).

```rust
let mut map = HashMap::new();
map.insert("content-type", "application/json");
map.insert("server", "MyServer/1.0");
let headers = ResponseHeaders::from_static_map(map);
```

#### `default() -> Self`

Creates a new empty `ResponseHeaders` collection (implements `Default` trait).

```rust
let headers = ResponseHeaders::default();
```

### Header Manipulation

#### `insert<K, V>(&mut self, key: K, value: V)`

Inserts a single header value, replacing any existing values.

**Parameters:**

- `key`: Header name (case-insensitive)
- `value`: Header value (converted to `String`)

```rust
headers.insert("Content-Type", "application/json");
headers.insert("X-Custom-Header", "custom-value");
headers.insert("Cache-Control", "max-age=3600");
```

#### `append<K, V>(&mut self, key: K, value: V)`

Appends a header value, preserving existing values.

**Parameters:**

- `key`: Header name (case-insensitive)
- `value`: Header value to append

```rust
headers.append("Set-Cookie", "session=abc123; HttpOnly");
headers.append("Set-Cookie", "theme=dark; Path=/"); // Now has both cookies
```

#### `remove<K>(&mut self, key: K) -> Option<Vec<String>>`

Removes all values for a header.

**Parameters:**

- `key`: Header name (case-insensitive)

**Returns:** `Option<Vec<String>>` - The removed values, if any

```rust
let removed = headers.remove("X-Debug-Info");
```

### Header Access

#### `get<K>(&self, key: K) -> Option<&str>`

Gets the first value for a header.

**Parameters:**

- `key`: Header name (case-insensitive)

**Returns:** `Option<&str>` - The first header value, if present

```rust
if let Some(content_type) = headers.get("Content-Type") {
    println!("Content-Type: {}", content_type);
}
```

#### `get_all<K>(&self, key: K) -> Option<&Vec<String>>`

Gets all values for a header.

**Parameters:**

- `key`: Header name (case-insensitive)

**Returns:** `Option<&Vec<String>>` - All header values, if present

```rust
if let Some(cookies) = headers.get_all("Set-Cookie") {
    for cookie in cookies {
        println!("Cookie: {}", cookie);
    }
}
```

#### `contains_key<K>(&self, key: K) -> bool`

Checks if a header exists.

**Parameters:**

- `key`: Header name (case-insensitive)

**Returns:** `bool` - True if the header exists

```rust
if headers.contains_key("Authorization") {
    // Handle authenticated response
}
```

### Content Type Methods

#### `content_type<V>(&mut self, content_type: V)`

Sets the `Content-Type` header.

**Parameters:**

- `content_type`: MIME type string

```rust
headers.content_type("application/json");
headers.content_type("text/html; charset=utf-8");
headers.content_type("image/png");
```

#### `json(&mut self)`

Sets content type to JSON (`application/json`).

```rust
headers.json();
```

#### `html(&mut self)`

Sets content type to HTML with UTF-8 encoding (`text/html; charset=utf-8`).

```rust
headers.html();
```

#### `text(&mut self)`

Sets content type to plain text with UTF-8 encoding (`text/plain; charset=utf-8`).

```rust
headers.text();
```

#### `xml(&mut self)`

Sets content type to XML (`application/xml`).

```rust
headers.xml();
```

### Content Management

#### `content_length(&mut self, length: u64)`

Sets the `Content-Length` header.

**Parameters:**

- `length`: Content length in bytes

```rust
headers.content_length(1024);
headers.content_length(response_body.len() as u64);
```

#### `attachment<V>(&mut self, filename: V)`

Sets headers for file download with specified filename.

**Parameters:**

- `filename`: Filename for download

```rust
headers.attachment("report.pdf");
headers.attachment("data.csv");
```

#### `inline(&mut self)`

Sets headers for inline file display (browser should display, not download).

```rust
headers.inline();
```

### Redirect and Location

#### `location<V>(&mut self, url: V)`

Sets the `Location` header (typically used with redirect status codes).

**Parameters:**

- `url`: Target URL for redirect

```rust
headers.location("/dashboard");
headers.location("https://example.com/new-page");
```

### Caching Headers

#### `cache_control<V>(&mut self, value: V)`

Sets the `Cache-Control` header.

**Parameters:**

- `value`: Cache control directive

```rust
headers.cache_control("max-age=3600");
headers.cache_control("no-cache, no-store, must-revalidate");
headers.cache_control("public, max-age=86400");
```

#### `no_cache(&mut self)`

Sets headers to prevent caching.

```rust
headers.no_cache();
// Sets:
// Cache-Control: no-cache, no-store, must-revalidate
// Pragma: no-cache
// Expires: 0
```

#### `etag<V>(&mut self, etag: V)`

Sets the `ETag` header for cache validation.

**Parameters:**

- `etag`: Entity tag value

```rust
headers.etag("\"abc123\"");
headers.etag("W/\"weak-etag\"");
```

#### `last_modified<V>(&mut self, date: V)`

Sets the `Last-Modified` header.

**Parameters:**

- `date`: Last modification date (RFC 7231 format)

```rust
headers.last_modified("Wed, 21 Oct 2015 07:28:00 GMT");
headers.last_modified(&chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string());
```

### Server Information

#### `server<V>(&mut self, server: V)`

Sets the `Server` header.

**Parameters:**

- `server`: Server identification string

```rust
headers.server("MyWebServer/1.0");
headers.server("nginx/1.18.0");
```

#### `powered_by<V>(&mut self, value: V)`

Sets the `X-Powered-By` header.

**Parameters:**

- `value`: Technology identifier

```rust
headers.powered_by("Rust/Actix-Web");
```

#### `remove_powered_by(&mut self)`

Removes the `X-Powered-By` header (security best practice).

```rust
headers.remove_powered_by();
```

### CORS (Cross-Origin Resource Sharing) Headers

#### `cors_allow_origin<V>(&mut self, origin: V)`

Sets the `Access-Control-Allow-Origin` header.

**Parameters:**

- `origin`: Allowed origin or "\*" for all

```rust
headers.cors_allow_origin("https://example.com");
headers.cors_allow_origin("*");
```

#### `cors_allow_methods<V>(&mut self, methods: V)`

Sets the `Access-Control-Allow-Methods` header.

**Parameters:**

- `methods`: Comma-separated list of allowed HTTP methods

```rust
headers.cors_allow_methods("GET, POST, PUT, DELETE");
headers.cors_allow_methods("GET, POST, OPTIONS");
```

#### `cors_allow_headers<V>(&mut self, headers_list: V)`

Sets the `Access-Control-Allow-Headers` header.

**Parameters:**

- `headers_list`: Comma-separated list of allowed headers

```rust
headers.cors_allow_headers("Content-Type, Authorization");
headers.cors_allow_headers("X-Requested-With, Accept");
```

#### `cors_allow_credentials(&mut self, allow: bool)`

Sets the `Access-Control-Allow-Credentials` header.

**Parameters:**

- `allow`: Whether to allow credentials in CORS requests

```rust
headers.cors_allow_credentials(true);
headers.cors_allow_credentials(false);
```

#### `cors_simple(&mut self, origin: Option<&str>)`

Sets basic CORS headers for simple requests.

**Parameters:**

- `origin`: Optional specific origin, or None for "\*"

```rust
headers.cors_simple(Some("https://myapp.com"));
headers.cors_simple(None); // Allows all origins

// Sets:
// Access-Control-Allow-Origin: (specified or *)
// Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
// Access-Control-Allow-Headers: Content-Type, Authorization
```

### Security Headers

#### `frame_options<V>(&mut self, value: V)`

Sets the `X-Frame-Options` header to prevent clickjacking.

**Parameters:**

- `value`: Frame options directive

```rust
headers.frame_options("DENY");
headers.frame_options("SAMEORIGIN");
headers.frame_options("ALLOW-FROM https://trusted.com");
```

#### `no_sniff(&mut self)`

Sets the `X-Content-Type-Options` header to prevent MIME sniffing.

```rust
headers.no_sniff();
// Sets: X-Content-Type-Options: nosniff
```

#### `xss_protection(&mut self, enabled: bool)`

Sets the `X-XSS-Protection` header.

**Parameters:**

- `enabled`: Whether to enable XSS protection

```rust
headers.xss_protection(true);  // Sets: X-XSS-Protection: 1; mode=block
headers.xss_protection(false); // Sets: X-XSS-Protection: 0
```

#### `hsts(&mut self, max_age: u64, include_subdomains: bool)`

Sets the `Strict-Transport-Security` header (HSTS).

**Parameters:**

- `max_age`: Maximum age in seconds
- `include_subdomains`: Whether to include subdomains

```rust
headers.hsts(31536000, true); // 1 year with subdomains
headers.hsts(86400, false);   // 1 day without subdomains
// Sets: Strict-Transport-Security: max-age=31536000; includeSubDomains
```

#### `csp<V>(&mut self, policy: V)`

Sets the `Content-Security-Policy` header.

**Parameters:**

- `policy`: CSP policy directive

```rust
headers.csp("default-src 'self'; script-src 'self' 'unsafe-inline'");
headers.csp("default-src 'none'; img-src 'self'; style-src 'self'");
```

#### `security_headers(&mut self)`

Sets a collection of basic security headers.

```rust
headers.security_headers();
// Sets:
// X-Content-Type-Options: nosniff
// X-XSS-Protection: 1; mode=block
// X-Frame-Options: DENY
// Removes: X-Powered-By
```

### Builder Pattern Methods

#### `with_header<K, V>(self, key: K, value: V) -> Self`

Builder method to set a header and return self.

```rust
let headers = ResponseHeaders::new()
    .with_header("Content-Type", "application/json")
    .with_header("Cache-Control", "max-age=3600");
```

#### `with_content_type<V>(self, content_type: V) -> Self`

Builder method to set content type and return self.

```rust
let headers = ResponseHeaders::new()
    .with_content_type("text/html");
```

#### `with_cors(self, origin: Option<&str>) -> Self`

Builder method to set CORS headers and return self.

```rust
let headers = ResponseHeaders::new()
    .with_cors(Some("https://example.com"));
```

#### `with_security(self) -> Self`

Builder method to set security headers and return self.

```rust
let headers = ResponseHeaders::new()
    .with_security();
```

### Inspection and Iteration

#### `keys(&self) -> impl Iterator<Item = &String>`

Returns an iterator over all header names.

```rust
for header_name in headers.keys() {
    println!("Header: {}", header_name);
}
```

#### `len(&self) -> usize`

Returns the number of unique headers.

```rust
println!("Response has {} headers", headers.len());
```

#### `is_empty(&self) -> bool`

Checks if there are no headers.

```rust
if headers.is_empty() {
    println!("No headers set");
}
```

#### `iter(&self) -> impl Iterator<Item = (&String, &str)>`

Iterates over headers as (key, first_value) pairs.

```rust
for (name, value) in headers.iter() {
    println!("{}: {}", name, value);
}
```

#### `iter_all(&self) -> impl Iterator<Item = (&String, &Vec<String>)>`

Iterates over all headers including multiple values.

```rust
for (name, values) in headers.iter_all() {
    for value in values {
        println!("{}: {}", name, value);
    }
}
```

### Conversion Methods

#### `to_map(&self) -> HashMap<String, String>`

Converts to a single-value `HashMap` (takes first value for each header).

```rust
let map: HashMap<String, String> = headers.to_map();
```

#### `to_header_lines(&self) -> Vec<String>`

Generates all header lines for HTTP response formatting.

```rust
let header_lines = headers.to_header_lines();
for line in header_lines {
    println!("{}", line);
}
// Output:
// content-type: application/json
// cache-control: max-age=3600
```

### Special Syntax

#### Index Access

You can use bracket notation to access headers (panics if header doesn't exist):

```rust
// This will panic if "content-type" header is missing!
let content_type = &headers["content-type"];

// Safer approach:
if headers.contains_key("content-type") {
    let content_type = &headers["content-type"];
}
```

#### Display Formatting

The struct implements `Display` for easy debugging:

```rust
println!("Response headers:\n{}", headers);
// Output:
// content-type: application/json
// cache-control: max-age=3600
// x-frame-options: DENY
```

## Real-World Examples

### JSON API Response

```rust
use your_crate::ResponseHeaders;

fn json_api_response(data: &str) -> (ResponseHeaders, String) {
    let headers = ResponseHeaders::new()
        .with_content_type("application/json")
        .with_security()
        .with_cors(Some("https://myapp.com"))
        .with_header("Cache-Control", "no-cache");

    (headers, data.to_string())
}
```

### File Download Handler

```rust
fn download_file(filename: &str, content: Vec<u8>) -> (ResponseHeaders, Vec<u8>) {
    let mut headers = ResponseHeaders::new();

    // Set content type based on file extension
    let content_type = match filename.split('.').last() {
        Some("pdf") => "application/pdf",
        Some("csv") => "text/csv",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    };

    headers.content_type(content_type);
    headers.content_length(content.len() as u64);
    headers.attachment(filename);
    headers.security_headers();

    (headers, content)
}
```

### Cached Static Asset

```rust
fn static_asset_response(content: &[u8], etag: &str) -> ResponseHeaders {
    let mut headers = ResponseHeaders::new();

    headers.content_type("text/css");
    headers.content_length(content.len() as u64);
    headers.cache_control("public, max-age=31536000"); // 1 year
    headers.etag(etag);
    headers.security_headers();

    headers
}
```

### Redirect Response

```rust
fn redirect_response(target_url: &str, permanent: bool) -> ResponseHeaders {
    let mut headers = ResponseHeaders::new();

    headers.location(target_url);
    headers.no_cache(); // Don't cache redirects
    headers.security_headers();

    // Add HSTS for permanent redirects to HTTPS
    if permanent && target_url.starts_with("https://") {
        headers.hsts(31536000, true);
    }

    headers
}
```

### Secure HTML Page

```rust
fn secure_html_response(content: &str) -> (ResponseHeaders, String) {
    let headers = ResponseHeaders::new()
        .with_content_type("text/html; charset=utf-8")
        .with_security()
        .with_header("Content-Security-Policy",
                    "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'")
        .with_header("Referrer-Policy", "strict-origin-when-cross-origin")
        .with_header("Cache-Control", "private, max-age=300");

    (headers, content.to_string())
}
```

### API with Advanced CORS

```rust
fn api_with_advanced_cors(origin: Option<&str>) -> ResponseHeaders {
    let mut headers = ResponseHeaders::new();

    headers.json();
    headers.security_headers();

    match origin {
        Some(origin) if is_trusted_origin(origin) => {
            headers.cors_allow_origin(origin);
            headers.cors_allow_credentials(true);
            headers.cors_allow_methods("GET, POST, PUT, DELETE, PATCH, OPTIONS");
            headers.cors_allow_headers("Content-Type, Authorization, X-Requested-With");
        },
        _ => {
            headers.cors_simple(None);
            headers.cors_allow_credentials(false);
        }
    }

    headers
}

fn is_trusted_origin(origin: &str) -> bool {
    origin.ends_with(".mycompany.com") || origin == "https://myapp.com"
}
```

### Health Check Endpoint

```rust
fn health_check_response() -> (ResponseHeaders, String) {
    let headers = ResponseHeaders::new()
        .with_content_type("application/json")
        .with_header("Cache-Control", "no-cache, no-store")
        .with_header("X-Health-Check", "OK");

    let body = r#"{"status": "healthy", "timestamp": "2024-01-01T12:00:00Z"}"#;
    (headers, body.to_string())
}
```

### Error Response

```rust
fn error_response(status_code: u16, message: &str) -> (ResponseHeaders, String) {
    let mut headers = ResponseHeaders::new();

    headers.json();
    headers.no_cache();
    headers.security_headers();

    // Add error-specific headers
    if status_code >= 500 {
        headers.insert("X-Error-Type", "server-error");
    } else if status_code >= 400 {
        headers.insert("X-Error-Type", "client-error");
    }

    let body = format!(r#"{{"error": "{}", "status": {}}}"#, message, status_code);
    (headers, body)
}
```

## Integration with Web Frameworks

### With Actix Web

```rust
use actix_web::{HttpResponse, Result};
use your_crate::ResponseHeaders;

async fn api_handler() -> Result<HttpResponse> {
    let headers = ResponseHeaders::new()
        .with_content_type("application/json")
        .with_security()
        .with_cors(Some("https://myapp.com"));

    let mut response = HttpResponse::Ok();

    // Add headers to response
    for (name, value) in headers.iter() {
        response.insert_header((name.as_str(), value));
    }

    response.json(serde_json::json!({"message": "Hello, world!"}))
}
```

### With Warp

```rust
use warp::{Reply, reply::Response};
use your_crate::ResponseHeaders;

fn with_headers(reply: impl Reply, headers: ResponseHeaders) -> Response {
    let mut response = reply.into_response();

    for (name, value) in headers.iter() {
        response.headers_mut().insert(
            name.parse().unwrap(),
            value.parse().unwrap()
        );
    }

    response
}

async fn api_handler() -> Result<impl Reply, warp::Rejection> {
    let headers = ResponseHeaders::new()
        .with_json()
        .with_security();

    let data = serde_json::json!({"message": "Hello, world!"});
    Ok(with_headers(warp::reply::json(&data), headers))
}
```

### With Axum

```rust
use axum::{response::{Response, Json}, http::HeaderMap};
use your_crate::ResponseHeaders;

async fn api_handler() -> Response {
    let headers = ResponseHeaders::new()
        .with_content_type("application/json")
        .with_security();

    let mut header_map = HeaderMap::new();
    for (name, value) in headers.iter() {
        if let (Ok(name), Ok(value)) = (name.parse(), value.parse()) {
            header_map.insert(name, value);
        }
    }

    (header_map, Json(serde_json::json!({"message": "Hello, world!"}))).into_response()
}
```

## Security Best Practices

### Essential Security Headers

```rust
fn secure_response_headers() -> ResponseHeaders {
    let mut headers = ResponseHeaders::new();

    // Basic security headers
    headers.security_headers();

    // Additional security measures
    headers.hsts(31536000, true); // 1 year HSTS
    headers.csp("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'");
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin");
    headers.insert("Permissions-Policy", "camera=(), microphone=(), geolocation=()");

    headers
}
```

### Production-Ready Headers

```rust
fn production_headers() -> ResponseHeaders {
    ResponseHeaders::new()
        .with_security()
        .with_header("Strict-Transport-Security", "max-age=31536000; includeSubDomains; preload")
        .with_header("Content-Security-Policy",
                    "default-src 'self'; script-src 'self' 'unsafe-inline'; object-src 'none'")
        .with_header("Referrer-Policy", "strict-origin-when-cross-origin")
        .with_header("Permissions-Policy", "camera=(), microphone=(), geolocation=()")
        .with_header("Cross-Origin-Embedder-Policy", "require-corp")
        .with_header("Cross-Origin-Opener-Policy", "same-origin")
}
```

## Performance Considerations

- Headers are stored with lowercase keys for efficient case-insensitive lookup
- Uses `Vec<String>` for multi-value headers, optimized for typical single-value cases
- Builder pattern methods use move semantics for zero-cost chaining
- Memory usage scales linearly with the number of unique headers
- Iterator methods provide zero-allocation iteration over headers

## Thread Safety

`ResponseHeaders` implements `Clone` and `Debug`, making it suitable for use in multi-threaded web servers. The struct can be safely shared across threads by cloning when needed.
