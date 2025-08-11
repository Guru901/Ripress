# RequestHeaders Documentation

A case-insensitive HTTP header management struct for Rust web applications.

## Overview

`RequestHeaders` provides a convenient interface for managing HTTP request headers with support for multiple values per header, case-insensitive key handling, and common header convenience methods.

## Features

- **Case-insensitive**: All header names are automatically converted to lowercase
- **Multiple values**: Supports headers with multiple values (e.g., `Accept-Encoding: gzip, deflate`)
- **Type flexibility**: Generic methods accept any type implementing `AsRef<str>`
- **Common headers**: Built-in convenience methods for frequently used headers
- **Iterator support**: Multiple ways to iterate over headers
- **Display formatting**: Pretty-print headers for debugging

## Basic Usage

```rust
use ripress::req::request_headers::RequestHeaders;

// Create new empty headers
let mut headers = RequestHeaders::new();

// Add headers
headers.insert("Content-Type", "application/json");
headers.insert("User-Agent", "MyApp/1.0");

// Append additional values (useful for headers that can have multiple values)
headers.append("Accept-Encoding", "gzip");
headers.append("Accept-Encoding", "deflate");

// Get header values
if let Some(content_type) = headers.get("content-type") {
    println!("Content-Type: {}", content_type);
}

// Check if header exists
if headers.contains_key("authorization") {
    println!("Request is authorized");
}
```

## API Reference

### Construction

#### `new() -> Self`

Creates a new empty `RequestHeaders` collection.

```rust
let headers = RequestHeaders::new();
```

#### `default() -> Self`

Creates a new empty `RequestHeaders` collection (implements `Default` trait).

```rust
let headers = RequestHeaders::default();
```

### Header Manipulation

#### `insert<K, V>(&mut self, key: K, value: V)`

Inserts a single header value, replacing any existing values for that key.

**Parameters:**

- `key`: Header name (case-insensitive)
- `value`: Header value

```rust
headers.insert("Content-Type", "application/json");
headers.insert("X-Custom-Header", "custom-value");
```

#### `append<K, V>(&mut self, key: K, value: V)`

Appends a header value, preserving existing values (useful for multi-value headers).

**Parameters:**

- `key`: Header name (case-insensitive)
- `value`: Header value to append

```rust
headers.append("Accept", "text/html");
headers.append("Accept", "application/json"); // Now has both values
```

#### `remove<K>(&mut self, key: K) -> Option<Vec<String>>`

Removes all values for a header, returning them if they existed.

**Parameters:**

- `key`: Header name (case-insensitive)

**Returns:** `Option<Vec<String>>` - The removed values, if any

```rust
let removed = headers.remove("X-Debug-Info");
if let Some(values) = removed {
    println!("Removed debug headers: {:?}", values);
}
```

### Header Access

#### `get<K>(&self, key: K) -> Option<&str>`

Gets the first value for a header (most common use case).

**Parameters:**

- `key`: Header name (case-insensitive)

**Returns:** `Option<&str>` - The first header value, if present

```rust
if let Some(auth) = headers.get("Authorization") {
    println!("Auth header: {}", auth);
}
```

#### `get_all<K>(&self, key: K) -> Option<&Vec<String>>`

Gets all values for a header.

**Parameters:**

- `key`: Header name (case-insensitive)

**Returns:** `Option<&Vec<String>>` - All header values, if present

```rust
if let Some(encodings) = headers.get_all("Accept-Encoding") {
    for encoding in encodings {
        println!("Accepts: {}", encoding);
    }
}
```

#### `contains_key<K>(&self, key: K) -> bool`

Checks if a header exists (regardless of its value).

**Parameters:**

- `key`: Header name (case-insensitive)

**Returns:** `bool` - True if the header exists

```rust
if headers.contains_key("Authorization") {
    // Handle authenticated request
}
```

### Convenience Methods for Common Headers

#### `content_type(&self) -> Option<&str>`

Gets the `Content-Type` header value.

```rust
match headers.content_type() {
    Some("application/json") => handle_json(),
    Some("text/html") => handle_html(),
    _ => handle_other(),
}
```

#### `authorization(&self) -> Option<&str>`

Gets the `Authorization` header value.

```rust
if let Some(auth) = headers.authorization() {
    if auth.starts_with("Bearer ") {
        let token = &auth[7..];
        validate_token(token);
    }
}
```

#### `user_agent(&self) -> Option<&str>`

Gets the `User-Agent` header value.

```rust
if let Some(ua) = headers.user_agent() {
    log_user_agent(ua);
}
```

#### `accept(&self) -> Option<&str>`

Gets the `Accept` header value.

```rust
let response_format = match headers.accept() {
    Some(accept) if accept.contains("application/json") => "json",
    Some(accept) if accept.contains("text/html") => "html",
    _ => "text",
};
```

#### `host(&self) -> Option<&str>`

Gets the `Host` header value.

```rust
if let Some(host) = headers.host() {
    println!("Request host: {}", host);
}
```

#### `x_forwarded_for(&self) -> Option<&str>`

Gets the `X-Forwarded-For` header value (useful for real IP behind proxies).

```rust
let client_ip = headers.x_forwarded_for()
    .unwrap_or("unknown");
```

### Content Type Checking

#### `accepts_json(&self) -> bool`

Checks if the request accepts JSON responses.

```rust
if headers.accepts_json() {
    return json_response();
} else {
    return html_response();
}
```

#### `accepts_html(&self) -> bool`

Checks if the request accepts HTML responses.

```rust
if headers.accepts_html() {
    render_template()
} else {
    return_api_response()
}
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
println!("Request has {} headers", headers.len());
```

#### `is_empty(&self) -> bool`

Checks if there are no headers.

```rust
if headers.is_empty() {
    println!("No headers in request");
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
    println!("{}: {:?}", name, values);
}
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
println!("All headers:\n{}", headers);
// Output:
// content-type: application/json
// user-agent: MyApp/1.0
// accept-encoding: gzip
// accept-encoding: deflate
```

## Example: Middleware Usage

```rust
fn auth_middleware(headers: &RequestHeaders) -> Result<(), AuthError> {
    match headers.authorization() {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = &auth[7..];
            validate_jwt_token(token)
        },
        Some(auth) if auth.starts_with("Basic ") => {
            let credentials = &auth[6..];
            validate_basic_auth(credentials)
        },
        _ => Err(AuthError::MissingAuth),
    }
}

fn content_negotiation(headers: &RequestHeaders) -> ResponseFormat {
    if headers.accepts_json() {
        ResponseFormat::Json
    } else if headers.accepts_html() {
        ResponseFormat::Html
    } else {
        ResponseFormat::Text
    }
}

fn request_logging(headers: &RequestHeaders) {
    let ip = headers.x_forwarded_for()
        .or_else(|| headers.get("x-real-ip"))
        .unwrap_or("unknown");

    let user_agent = headers.user_agent()
        .unwrap_or("unknown");

    println!("Request from {} using {}", ip, user_agent);
}
```

## Thread Safety

`RequestHeaders` derives `Clone` and `Debug` (after adding the derives above). Its inner `HashMap<String, Vec<String>>` is also `Send + Sync`, so the struct can be shared across threads via standard Rust ownership rules. For mutation, clone per thread or wrap in your preferred synchronization primitive.

## Performance Considerations

- Header names are converted to lowercase on insertion for case-insensitive matching
- Multiple values are stored as `Vec<String>` for efficient appending
- Uses `HashMap` internally for O(1) average-case lookup performance
- Memory usage scales linearly with the number of unique headers and their values
