# RouteParams Documentation

A type-safe URL parameter extraction system for Rust web applications supporting automatic parsing and validation.

## Overview

`RouteParams` provides a convenient interface for extracting and parsing parameters from URL routes like `/users/:id` or `/posts/:slug/comments/:comment_id`. It handles type conversion, validation, and provides helpful error messages when parameters are missing or invalid.

## Features

- **Type-safe parsing**: Automatic conversion from string parameters to any type implementing `FromStr`
- **Comprehensive error handling**: Detailed error messages for missing parameters and parse failures
- **Convenience methods**: Built-in helpers for common parameter types and patterns
- **Flexible API**: Multiple ways to access parameters with different error handling strategies
- **Macro support**: Easy bulk parameter extraction with validation
- **Backward compatibility**: Seamless conversion to/from `HashMap<String, String>`

## Basic Usage

```rust
use ripress::req::route_params::{ParamError, RouteParams};

// Create from route matching (typically done by your router)
let mut params = RouteParams::new();
params.insert("id", "123");
params.insert("slug", "hello-world");
params.insert("page", "2");

// Get parameters as strings
let id_str = params.get("id"); // Some("123")
let missing = params.get("missing"); // None

// Parse parameters to specific types
let id: i32 = params.get_parsed("id")?; // 123
let page: u32 = params.get_parsed("page")?; // 2

// Use convenience methods
let user_id = params.get_int("id")?; // 123 as i32
let article_slug = params.slug(); // Some("hello-world")
```

## Error Handling

```rust
use ripress::req::route_params::{RouteParams, ParamError};

let params = RouteParams::new();

// Handle missing parameters
match params.get_int("id") {
    Ok(id) => println!("User ID: {}", id),
    Err(ParamError::NotFound(param)) => {
        println!("Missing required parameter: {}", param);
    },
    Err(ParamError::ParseError { param, value, target_type }) => {
        println!("Invalid {}: '{}' cannot be parsed as {}", param, value, target_type);
    },
}

// Or use defaults for optional parameters
let page = params.get_or_default("page", 1); // Default to page 1
```

## API Reference

### Construction

#### `new() -> Self`

Creates a new empty `RouteParams` collection.

```rust
let params = RouteParams::new();
```

#### `from_map(map: HashMap<String, String>) -> Self`

Creates `RouteParams` from an existing `HashMap` (useful for migration from older code).

```rust
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert("id".to_string(), "123".to_string());
let params = RouteParams::from_map(map);
```

#### `default() -> Self`

Creates a new empty `RouteParams` collection (implements `Default` trait).

```rust
let params = RouteParams::default();
```

### Parameter Manipulation

#### `insert<K, V>(&mut self, key: K, value: V)`

Inserts a parameter into the collection.

**Parameters:**

- `key`: Parameter name (converted to `String`)
- `value`: Parameter value (converted to `String`)

```rust
params.insert("id", "123");
params.insert("slug", "my-article");
params.insert("category_id", 42); // Auto-converted to string
```

### Parameter Access

#### `get(&self, name: &str) -> Option<&str>`

Gets a parameter value as a string reference.

**Parameters:**

- `name`: Parameter name

**Returns:** `Option<&str>` - The parameter value if present

```rust
if let Some(id_str) = params.get("id") {
    println!("ID as string: {}", id_str);
}
```

#### `get_parsed<T>(&self, name: &str) -> Result<T, ParamError>`

Gets a parameter and parses it to a specific type.

**Parameters:**

- `name`: Parameter name
- `T`: Target type (must implement `FromStr`)

**Returns:** `Result<T, ParamError>` - Parsed value or detailed error

```rust
let id: i32 = params.get_parsed("id")?;
let user_id: u64 = params.get_parsed("user_id")?;
let active: bool = params.get_parsed("active")?;

// Works with any type implementing FromStr
use std::net::IpAddr;
let ip: IpAddr = params.get_parsed("client_ip")?;
```

#### `get_int(&self, name: &str) -> Result<i32, ParamError>`

Convenience method to get a parameter as a 32-bit signed integer.

```rust
let user_id = params.get_int("id")?;
```

#### `get_uint(&self, name: &str) -> Result<u32, ParamError>`

Convenience method to get a parameter as a 32-bit unsigned integer.

```rust
let page_number = params.get_uint("page")?;
```

#### `get_or_default<T>(&self, name: &str, default: T) -> T`

Gets a parameter with a default value if missing or parsing fails.

**Parameters:**

- `name`: Parameter name
- `default`: Default value to return if parameter is missing or invalid

**Returns:** `T` - Parsed value or the default

```rust
let page = params.get_or_default("page", 1); // Default to page 1
let limit = params.get_or_default("limit", 10); // Default to 10 items
let debug = params.get_or_default("debug", false); // Default to false
```

#### `get_or_parse_default<T>(&self, name: &str, default: T) -> Result<T, ParamError>`

Gets a parameter with a default value only if parsing fails (still returns error if missing).

**Parameters:**

- `name`: Parameter name
- `default`: Default value if parsing fails

**Returns:** `Result<T, ParamError>` - Parsed value, default, or NotFound error

```rust
// Returns error if "limit" parameter is missing, but uses default if it's invalid
let limit = params.get_or_parse_default("limit", 10)?;
```

### Parameter Inspection

#### `contains(&self, name: &str) -> bool`

Checks if a parameter exists.

```rust
if params.contains("optional_param") {
    // Handle optional parameter
}
```

#### `names(&self) -> impl Iterator<Item = &String>`

Returns an iterator over all parameter names.

```rust
for name in params.names() {
    println!("Parameter: {}", name);
}
```

#### `len(&self) -> usize`

Returns the number of parameters.

```rust
println!("Route has {} parameters", params.len());
```

#### `is_empty(&self) -> bool`

Checks if there are no parameters.

```rust
if params.is_empty() {
    println!("No route parameters");
}
```

#### `iter(&self) -> impl Iterator<Item = (&String, &String)>`

Iterates over all parameters as (name, value) pairs.

```rust
for (name, value) in params.iter() {
    println!("{}: {}", name, value);
}
```

### Convenience Methods for Common Patterns

#### `id(&self) -> Result<i32, ParamError>`

Gets the 'id' parameter as an integer (very common in REST APIs).

```rust
let user_id = params.id()?; // Equivalent to params.get_int("id")
```

#### `slug(&self) -> Option<&str>`

Gets the 'slug' parameter (common for SEO-friendly URLs).

```rust
if let Some(article_slug) = params.slug() {
    println!("Article slug: {}", article_slug);
}
```

### Conversion Methods

#### `into_map(self) -> HashMap<String, String>`

Converts `RouteParams` back to a `HashMap` (for backward compatibility).

```rust
let hash_map: HashMap<String, String> = params.into_map();
```

#### `extract<F>(&self, extractor: F) -> Result<(), Vec<ParamError>>`

Validates multiple parameters at once using a custom extractor function.

```rust
params.extract(|p| {
    let id = p.get_int("id")?;
    let user_id = p.get_int("user_id")?;

    if id <= 0 || user_id <= 0 {
        return Err(vec![ParamError::ParseError {
            param: "id/user_id".to_string(),
            value: "negative".to_string(),
            target_type: "positive integer".to_string(),
        }]);
    }

    Ok(())
})?;
```

### Special Syntax

#### Index Access

You can use bracket notation to access parameters (panics if parameter doesn't exist):

```rust
// This will panic if "id" parameter is missing!
let id_str = &params["id"];

// Safer approach:
if params.contains("id") {
    let id_str = &params["id"];
}
```

#### Display Formatting

The struct implements `Display` for easy debugging:

```rust
println!("Route parameters: {}", params);
// Output: id=123, slug=hello-world, page=2
```

## Error Types

### `ParamError`

#### `NotFound(String)`

Parameter was not found in the route.

```rust
match params.get_int("missing") {
    Err(ParamError::NotFound(param)) => {
        println!("Required parameter '{}' is missing", param);
    },
    _ => {}
}
```

#### `ParseError { param: String, value: String, target_type: String }`

Parameter exists but couldn't be parsed to the requested type.

```rust
match params.get_int("invalid_number") {
    Err(ParamError::ParseError { param, value, target_type }) => {
        println!("Parameter '{}' has invalid value '{}' for type {}",
                param, value, target_type);
    },
    _ => {}
}
```

## Bulk Parameter Extraction

### `extract_params!` Macro

For extracting multiple parameters with validation:

```rust
use ripress::extract_params;

// Extract multiple parameters at once
let result = extract_params!(params, {
    id: i32,
    user_id: u32,
    active: bool
});

match result {
    Ok((id, user_id, active)) => {
        // All parameters parsed successfully
        handle_request(id, user_id, active);
    },
    Err(errors) => {
        // One or more parameters failed to parse
        for error in errors {
            eprintln!("Parameter error: {}", error);
        }
    }
}
```

## Real-World Examples

### REST API Handler

```rust
use ripress::req::route_params::{ParamError, RouteParams};

fn get_user_posts(params: RouteParams) -> Result<Vec<Post>, ApiError> {
    // Extract required ID
 let user_id = params.id().map_err(|e| ApiError::Generic(
   HttpResponse::new().bad_request().text(e.to_string())
    ))?;

    // Extract optional pagination parameters
    let page = params.get_or_default("page", 1);
    let limit = params.get_or_default("limit", 10);

    // Validate reasonable limits
    if limit > 100 {
        return Err(ApiError::InvalidParameter("limit too high".to_string()));
    }

    fetch_user_posts(user_id, page, limit)
}

// Route: GET /users/:id/posts?page=2&limit=20
```

### Blog Article Handler

```rust
fn get_article(params: RouteParams) -> Result<Article, ApiError> {
    // Support both ID and slug-based access
    if let Ok(id) = params.get_int("id") {
        fetch_article_by_id(id)
    } else if let Some(slug) = params.slug() {
        fetch_article_by_slug(slug)
    } else {
        Err(ApiError::MissingParameter("id or slug required".to_string()))
    }
}

// Route: GET /articles/:id or GET /articles/:slug
```

### Complex Parameter Validation

```rust
fn update_user_settings(params: RouteParams) -> Result<(), ApiError> {
    // Use the extract macro for complex validation
    let (user_id, theme_id, notifications) = extract_params!(params, {
        user_id: u32,
        theme_id: u8,
        notifications: bool
    }).map_err(ApiError::ParameterErrors)?;

    // Additional business logic validation
    if theme_id > 10 {
        return Err(ApiError::InvalidTheme);
    }

    update_settings(user_id, theme_id, notifications)
}
```

### Migration from HashMap

```rust
use std::collections::HashMap;
use ripress::req::route_params::RouteParams;

// Before: using HashMap directly
fn old_handler(params: HashMap<String, String>) -> Result<Response, Error> {
    let id_str = params.get("id").ok_or(Error::MissingId)?;
    let id: i32 = id_str.parse().map_err(|_| Error::InvalidId)?;

    process_request(id)
}

// After: using RouteParams
fn new_handler(params: RouteParams) -> Result<Response, Error> {
    let id = params.id().map_err(Error::from)?;

    process_request(id)
}

// Easy migration path
fn transitional_handler(old_params: HashMap<String, String>) -> Result<Response, Error> {
    let params = RouteParams::from_map(old_params);
    new_handler(params)
}
```

## Performance Considerations

- Uses `HashMap<String, String>` internally for O(1) average-case lookup
- Parameter parsing is lazy - only happens when requested
- String allocations only occur during insertion and error formatting
- Zero-cost abstractions for type conversion through `FromStr` trait
- Efficient iteration without additional allocations

## Thread Safety

`RouteParams` implements `Clone` and `Debug`, making it suitable for use in multi-threaded web servers. The struct can be safely shared across threads by cloning when needed.
