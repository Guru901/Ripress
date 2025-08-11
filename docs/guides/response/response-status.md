# StatusCode Documentation

## Overview

The `StatusCode` enum provides a type-safe representation of HTTP status codes in Rust. It includes commonly used HTTP status codes as named variants and supports custom status codes through the `Custom` variant.

## Features

- **Type Safety**: Compile-time guarantees for valid HTTP status codes
- **Display Implementation**: Human-readable string representation with both code and description
- **Bidirectional Conversion**: Convert between `u16` values and `StatusCode` variants
- **Custom Status Codes**: Support for non-standard or application-specific status codes
- **Zero-Cost Abstractions**: Efficient representation with no runtime overhead

## Usage

### Basic Usage

```rust
use ripress::res::response_status::StatusCode;

// Create status codes
let success = StatusCode::Ok;
let not_found = StatusCode::NotFound;
let custom = StatusCode::Custom(418); // I'm a teapot

// Display status codes
println!("{}", success);        // "200 OK"
println!("{}", not_found);      // "404 Not Found"
println!("{}", custom);         // "418 Custom"
```

### Converting from u16

```rust
let code = StatusCode::from_u16(200);
assert_eq!(code, StatusCode::Ok);

let custom_code = StatusCode::from_u16(418);
assert_eq!(custom_code, StatusCode::Custom(418));
```

### Converting to u16

```rust
let status = StatusCode::NotFound;
assert_eq!(status.as_u16(), 404);

let custom = StatusCode::Custom(418);
assert_eq!(custom.as_u16(), 418);
```

## Status Code Categories

### 2xx Success

- `Ok` (200) - Request succeeded
- `Created` (201) - Resource created successfully
- `Accepted` (202) - Request accepted for processing
- `NoContent` (204) - Request succeeded with no content to return

### 3xx Redirection

- `PermanentRedirect` (301) - Resource permanently moved
- `Redirect` (302) - Resource temporarily moved

### 4xx Client Error

- `BadRequest` (400) - Invalid request syntax
- `Unauthorized` (401) - Authentication required
- `Forbidden` (403) - Access denied
- `NotFound` (404) - Resource not found
- `MethodNotAllowed` (405) - HTTP method not supported
- `Conflict` (409) - Request conflicts with current state

### 5xx Server Error

- `InternalServerError` (500) - Generic server error
- `NotImplemented` (501) - Server doesn't support the functionality
- `BadGateway` (502) - Invalid response from upstream server
- `ServiceUnavailable` (503) - Server temporarily unavailable

### Custom Status Codes

The `Custom(u16)` variant allows you to use any valid HTTP status code, including:

- Non-standard codes (e.g., 418 I'm a teapot)
- Application-specific codes
- Future HTTP status codes not yet included in the enum

## API Reference

### Methods

#### `as_u16(&self) -> u16`

Returns the numeric HTTP status code.

```rust
let status = StatusCode::Ok;
assert_eq!(status.as_u16(), 200);
```

#### `from_u16(code: u16) -> StatusCode`

Creates a `StatusCode` from a numeric value. Returns a named variant if the code is recognized, otherwise returns `Custom(code)`.

```rust
assert_eq!(StatusCode::from_u16(404), StatusCode::NotFound);
assert_eq!(StatusCode::from_u16(999), StatusCode::Custom(999));
```

### Traits

#### `Display`

Provides human-readable formatting with both the numeric code and description.

```rust
let status = StatusCode::InternalServerError;
println!("{}", status); // "500 Internal Server Error"
```

#### `Debug`

Provides debug formatting showing the variant name.

```rust
let status = StatusCode::Ok;
println!("{:?}", status); // "Ok"
```

#### `Clone`, `Copy`

The enum can be cheaply copied and cloned.

#### `PartialEq`, `Eq`

Status codes can be compared for equality.

```rust
assert_eq!(StatusCode::Ok, StatusCode::from_u16(200));
```

## Examples

### HTTP Response Handling

```rust
fn handle_response(status: StatusCode) -> String {
    match status {
        StatusCode::Ok => "Success!".to_string(),
        StatusCode::NotFound => "Resource not found".to_string(),
        StatusCode::InternalServerError => "Server error occurred".to_string(),
        StatusCode::Custom(code) => format!("Custom status: {}", code),
        _ => format!("Status: {}", status),
    }
}
```

### Status Code Categories

```rust
impl StatusCode {
    pub fn is_success(&self) -> bool {
        matches!(self.as_u16(), 200..=299)
    }

    pub fn is_client_error(&self) -> bool {
        matches!(self.as_u16(), 400..=499)
    }

    pub fn is_server_error(&self) -> bool {
        matches!(self.as_u16(), 500..=599)
    }
}
```

### Integration with HTTP Libraries

```rust
// Example with a hypothetical HTTP client
fn make_request() -> Result<String, StatusCode> {
    let response_code = 404; // From HTTP response
    let status = StatusCode::from_u16(response_code);

    match status {
        StatusCode::Ok => Ok("Response body".to_string()),
        error_status => Err(error_status),
    }
}
```

## Design Rationale

### Why an Enum?

- **Type Safety**: Prevents invalid status codes at compile time
- **Pattern Matching**: Enables exhaustive matching on common status codes
- **Performance**: Zero-cost abstraction with efficient representation
- **Extensibility**: Custom variant handles edge cases without breaking the type system

### Ordering of Variants

The variants are organized by HTTP status code categories (2xx, 3xx, 4xx, 5xx) for better readability and logical grouping.

### Display Implementation

The `Display` implementation follows the standard HTTP format of "code description" (e.g., "404 Not Found") to maintain compatibility with HTTP specifications and tooling.

## Performance Notes

- The enum uses `#[repr(u16)]` internally for efficient storage
- All operations are zero-cost abstractions
- Pattern matching compiles to efficient jump tables
- No heap allocations are required for any operations

## Thread Safety

`StatusCode` implements `Send` and `Sync` automatically, making it safe to use across threads without additional synchronization.

## Compatibility

This implementation follows RFC 7231 and other HTTP specifications for status code definitions and formatting.
