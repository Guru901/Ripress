# QueryParams Documentation

A comprehensive URL query string parameter parsing and management system for Rust web applications with support for multiple values and type-safe conversion.

## Overview

`QueryParams` provides a robust interface for parsing and working with URL query parameters. It handles complex query strings with multiple values for the same parameter, automatic type conversion, and includes convenience methods for common web application patterns like pagination, search, and filtering.

## Features

- **Multi-value support**: Handle parameters with multiple values (e.g., `?tags=web&tags=rust`)
- **Type-safe parsing**: Automatic conversion to any type implementing `FromStr`
- **Flexible boolean parsing**: Support for various boolean representations
- **URL decoding**: Proper handling of URL-encoded parameters
- **Common patterns**: Built-in methods for pagination, search, sorting, and filtering
- **Comprehensive error handling**: Detailed error messages for parsing failures
- **Backward compatibility**: Easy migration from `HashMap<String, String>`

## Basic Usage

```rust
use ripress::req::query::{QueryParamError, QueryParams};

// Parse from query string
let query = QueryParams::from_query_string("page=2&limit=10&tags=rust&tags=web&active=true");

// Get single values
let page = query.get("page"); // Some("2")
let limit = query.get_int("limit")?; // 10 as i32

// Get multiple values
let tags = query.get_all("tags"); // Some(&["rust", "web"])
let all_tags = query.get_all_parsed::<String>("tags")?; // Vec<String>

// Use convenience methods
let page_num = query.page(); // 2 (with default fallback)
let search_term = query.search_query(); // None in this example
```

## Query String Examples

```rust
// Simple parameters
let query = QueryParams::from_query_string("page=1&limit=20");

// Multiple values for same parameter
let query = QueryParams::from_query_string("tags=rust&tags=web&tags=backend");

// Boolean parameters
let query = QueryParams::from_query_string("active=true&debug=1&feature&disabled=no");

// Complex filtering
let query = QueryParams::from_query_string("filter[status]=active&filter[type]=premium");

// Search and sorting
let query = QueryParams::from_query_string("q=rust+tutorial&sort=date&order=desc");
```

## API Reference

### Construction

#### `new() -> Self`

Creates a new empty `QueryParams` collection.

```rust
let query = QueryParams::new();
```

#### `from_query_string(query_string: &str) -> Self`

Parses query parameters from a URL query string.

**Parameters:**

- `query_string`: Raw query string (without the leading `?`)

**Returns:** `QueryParams` with parsed parameters

```rust
let query = QueryParams::from_query_string("page=2&tags=rust&tags=web");
```

**Supported formats:**

- Standard key-value pairs: `key=value`
- Multiple values: `tags=rust&tags=web`
- Empty values: `debug=` or just `debug`
- URL encoding: `q=hello%20world`

#### `from_map(map: HashMap<String, String>) -> Self`

Creates `QueryParams` from a single-value `HashMap` (for backward compatibility).

```rust
let mut map = HashMap::new();
map.insert("page".to_string(), "2".to_string());
let query = QueryParams::from_map(map);
```

#### `default() -> Self`

Creates a new empty `QueryParams` collection (implements `Default` trait).

```rust
let query = QueryParams::default();
```

### Parameter Manipulation

#### `insert<K, V>(&mut self, key: K, value: V)`

Inserts a single parameter value, replacing any existing values.

**Parameters:**

- `key`: Parameter name
- `value`: Parameter value

```rust
query.insert("page", "1");
query.insert("active", "true");
```

#### `append<K, V>(&mut self, key: K, value: V)`

Appends a parameter value, preserving existing values.

**Parameters:**

- `key`: Parameter name
- `value`: Parameter value to append

```rust
query.append("tags", "rust");
query.append("tags", "web"); // Now has both values
```

#### `remove(&mut self, name: &str) -> Option<Vec<String>>`

Removes all values for a parameter.

**Parameters:**

- `name`: Parameter name

**Returns:** `Option<Vec<String>>` - The removed values, if any

```rust
let removed_tags = query.remove("tags");
```

### Parameter Access

#### `get(&self, name: &str) -> Option<&str>`

Gets the first value for a parameter.

**Parameters:**

- `name`: Parameter name

**Returns:** `Option<&str>` - The first parameter value, if present

```rust
if let Some(page) = query.get("page") {
    println!("Page: {}", page);
}
```

#### `get_all(&self, name: &str) -> Option<&Vec<String>>`

Gets all values for a parameter.

**Parameters:**

- `name`: Parameter name

**Returns:** `Option<&Vec<String>>` - All parameter values, if present

```rust
if let Some(tags) = query.get_all("tags") {
    for tag in tags {
        println!("Tag: {}", tag);
    }
}
```

### Type-Safe Parsing

#### `get_parsed<T>(&self, name: &str) -> Result<T, QueryParamError>`

Gets the first value and parses it to a specific type.

**Parameters:**

- `name`: Parameter name
- `T`: Target type (must implement `FromStr`)

**Returns:** `Result<T, QueryParamError>` - Parsed value or detailed error

```rust
let page: i32 = query.get_parsed("page")?;
let price: f64 = query.get_parsed("price")?;
let uuid: Uuid = query.get_parsed("id")?;
```

#### `get_all_parsed<T>(&self, name: &str) -> Result<Vec<T>, QueryParamError>`

Gets all values and parses them to a specific type.

**Parameters:**

- `name`: Parameter name
- `T`: Target type (must implement `FromStr`)

**Returns:** `Result<Vec<T>, QueryParamError>` - Vector of parsed values or error

```rust
let tag_ids: Vec<i32> = query.get_all_parsed("tag_ids")?;
let categories: Vec<String> = query.get_all_parsed("categories")?;
```

### Convenience Type Methods

#### `get_int(&self, name: &str) -> Result<i32, QueryParamError>`

Gets a parameter as a 32-bit signed integer.

```rust
let page = query.get_int("page")?;
```

#### `get_i64(&self, name: &str) -> Result<i64, QueryParamError>`

Gets a parameter as a 64-bit signed integer.

```rust
let user_id = query.get_i64("user_id")?;
```

#### `get_uint(&self, name: &str) -> Result<u32, QueryParamError>`

Gets a parameter as a 32-bit unsigned integer.

```rust
let limit = query.get_uint("limit")?;
```

#### `get_bool(&self, name: &str) -> Result<bool, QueryParamError>`

Gets a parameter as a boolean with flexible, case-insensitive parsing. Does **not** trim whitespace, so values with leading/trailing spaces will not match the expected tokens.

**Supported values:**

- **True**: `"true"`, `"1"`, `"yes"`, `"on"` (case-insensitive)
- **False**: `"false"`, `"0"`, `"no"`, `"off"`, `""` (empty string, case-insensitive)

**Note:** The `is_truthy` method uses different semantics - it returns `true` whenever the parameter exists, regardless of its value. So `is_truthy("debug")` can be `true` even when `get_bool("debug")` would return `false` (e.g., for `?debug=false`).

```rust
let is_active = query.get_bool("active")?; // ?active=TRUE → true
let debug_mode = query.get_bool("debug")?; // ?debug=false → false

// Contrast with is_truthy:
let has_debug = query.is_truthy("debug"); // ?debug=false → true (parameter exists)
```

#### `get_float(&self, name: &str) -> Result<f64, QueryParamError>`

Gets a parameter as a 64-bit floating point number.

```rust
let price = query.get_float("price")?;
let latitude = query.get_float("lat")?;
```

#### `get_or_default<T>(&self, name: &str, default: T) -> T`

Gets a parameter with a default value if missing or parsing fails.

**Parameters:**

- `name`: Parameter name
- `default`: Default value to return

**Returns:** `T` - Parsed value or the default

```rust
let page = query.get_or_default("page", 1);
let limit = query.get_or_default("limit", 20);
let debug = query.get_or_default("debug", false);
```

### Parameter Inspection

#### `contains(&self, name: &str) -> bool`

Checks if a parameter exists (even with empty value).

```rust
if query.contains("debug") {
    // Debug parameter was specified
}
```

#### `has_value(&self, name: &str) -> bool`

Checks if a parameter has a non-empty value.

```rust
if query.has_value("q") {
    // Search query is not empty
}
```

#### `is_truthy(&self, name: &str) -> bool`

Checks if parameter indicates a "true" value (flexible boolean parsing or just presence).

```rust
// Returns true for: ?debug, ?debug=true, ?debug=1, etc.
if query.is_truthy("debug") {
    enable_debug_mode();
}
```

#### `names(&self) -> impl Iterator<Item = &String>`

Returns an iterator over all parameter names.

```rust
for name in query.names() {
    println!("Parameter: {}", name);
}
```

#### `len(&self) -> usize`

Returns the number of unique parameters.

```rust
println!("Query has {} parameters", query.len());
```

#### `is_empty(&self) -> bool`

Checks if there are no parameters.

```rust
if query.is_empty() {
    println!("No query parameters");
}
```

### Iteration

#### `iter(&self) -> impl Iterator<Item = (&String, &str)>`

Iterates over parameters as (name, first_value) pairs.

```rust
for (name, value) in query.iter() {
    println!("{}: {}", name, value);
}
```

#### `iter_all(&self) -> impl Iterator<Item = (&String, &Vec<String>)>`

Iterates over all parameters including multiple values.

```rust
for (name, values) in query.iter_all() {
    println!("{}: {:?}", name, values);
}
```

### Common Web Patterns

#### `page(&self) -> i32`

Gets the 'page' parameter with default of 1.

```rust
let current_page = query.page(); // Default: 1
```

#### `limit(&self) -> i32`

Gets the 'limit' or 'per_page' parameter with default of 20.

```rust
let items_per_page = query.limit(); // Default: 20
```

#### `offset(&self) -> i32`

Gets the 'offset' parameter with default of 0.

```rust
let skip_items = query.offset(); // Default: 0
```

#### `search_query(&self) -> Option<&str>`

Gets the search query from 'q', 'query', or 'search' parameters.

```rust
if let Some(search_term) = query.search_query() {
    perform_search(search_term);
}
```

#### `sort(&self) -> Option<&str>`

Gets the sort field from 'sort' or 'order_by' parameters.

```rust
let sort_field = query.sort().unwrap_or("created_at");
```

#### `sort_direction(&self) -> SortDirection`

Gets the sort direction from 'order', 'dir', or 'direction' parameters.

```rust
use ripress::req::query::SortDirection;

let direction = query.sort_direction(); // SortDirection::Asc or SortDirection::Desc
match direction {
    SortDirection::Asc => println!("Ascending order"),
    SortDirection::Desc => println!("Descending order"),
}
```

#### `filters(&self) -> HashMap<String, Vec<String>>`

Extracts filter parameters using the pattern `filter[key]=value`.

```rust
// Query: ?filter[status]=active&filter[type]=premium&filter[tag]=rust
let filters = query.filters();

if let Some(statuses) = filters.get("status") {
    // Handle status filter
}
```

### Conversion Methods

#### `into_map(self) -> HashMap<String, String>`

Converts to a single-value `HashMap` (takes first value for each parameter).

```rust
let hash_map: HashMap<String, String> = query.into_map();
```

### Special Syntax

#### Index Access

You can use bracket notation to access parameters (panics if parameter doesn't exist):

```rust
// This will panic if "page" parameter is missing!
let page_str = &query["page"];

// Safer approach:
if query.contains("page") {
    let page_str = &query["page"];
}
```

#### Display Formatting

The struct implements `Display` to recreate query strings (NOTE: output is not percent-encoded and is intended primarily for debugging/logging):

```rust
println!("Query string: {}", query);
// Output: page=2&limit=10&tags=rust&tags=web
```

## Error Types

### `QueryParamError`

#### `NotFound(String)`

Parameter was not found in the query string.

```rust
match query.get_int("missing") {
    Err(QueryParamError::NotFound(param)) => {
        println!("Parameter '{}' is missing", param);
    },
    _ => {}
}
```

#### `ParseError { param: String, value: String, target_type: String }`

Parameter exists but couldn't be parsed to the requested type.

```rust
match query.get_int("invalid_number") {
    Err(QueryParamError::ParseError { param, value, target_type }) => {
        println!("Cannot parse '{}' as {} for parameter '{}'",
                value, target_type, param);
    },
    _ => {}
}
```

#### `MultipleValues { param: String, values: Vec<String> }`

Multiple values found when single value expected (currently not used but available for future features).

## Real-World Examples

### Pagination Handler

```rust
use ripress::req::query::{QueryParamError, QueryParams};

fn handle_pagination(query: QueryParams) -> Result<PaginationInfo, ApiError> {
    let page = query.page().max(1); // Ensure minimum page 1
    let limit = query.limit().clamp(1, 100); // Limit between 1-100
    let offset = query.offset();

    // Alternative: calculate offset from page
    let calculated_offset = (page - 1) * limit;
    let final_offset = if offset > 0 { offset } else { calculated_offset };

    Ok(PaginationInfo {
        page,
        limit,
        offset: final_offset,
    })
}

// Usage: GET /api/users?page=2&limit=25
```

### Search and Filtering

```rust
fn handle_search(query: QueryParams) -> SearchParams {
    let search_term = query.search_query().map(|s| s.to_string());
    let sort_field = query.sort().unwrap_or("created_at").to_string();
    let sort_direction = query.sort_direction();

    // Extract category filters
    let categories = query.get_all_parsed::<String>("category")
        .unwrap_or_default();

    // Extract price range
    let min_price = query.get_float("min_price").ok();
    let max_price = query.get_float("max_price").ok();

    // Extract boolean filters
    let in_stock = query.get_bool("in_stock").unwrap_or(false);
    let featured = query.is_truthy("featured");

    SearchParams {
        query: search_term,
        categories,
        price_range: (min_price, max_price),
        in_stock,
        featured,
        sort_field,
        sort_direction,
    }
}

// Usage: GET /search?q=laptop&category=electronics&category=computers&min_price=500&featured
```

### Advanced Filtering

```rust
fn handle_advanced_filters(query: QueryParams) -> FilterSet {
    // Extract structured filters: filter[status]=active&filter[role]=admin
    let filters = query.filters();

    // Extract tag filters: tags=web&tags=rust&tags=backend
    let tags = query.get_all_parsed::<String>("tags")
        .unwrap_or_default();

    // Extract date ranges
    let created_after = query.get("created_after")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let created_before = query.get("created_before")
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    FilterSet {
        structured_filters: filters,
        tags,
        date_range: (created_after, created_before),
    }
}

// Usage: GET /api/posts?filter[status]=published&filter[author]=john&tags=rust&created_after=2024-01-01
```

### Form Processing

```rust
fn handle_form_submission(query: QueryParams) -> Result<FormData, ValidationError> {
    // Required fields
    let name = query.get("name")
        .ok_or(ValidationError::Required("name"))?;

    let email = query.get("email")
        .ok_or(ValidationError::Required("email"))?;

    // Optional fields with defaults
    let age = query.get_or_default("age", 0);
    let newsletter = query.is_truthy("newsletter");

    // Multi-select fields
    let interests = query.get_all_parsed::<String>("interests")
        .unwrap_or_default();

    // Validate email format
    if !is_valid_email(email) {
        return Err(ValidationError::InvalidFormat("email"));
    }

    Ok(FormData {
        name: name.to_string(),
        email: email.to_string(),
        age,
        newsletter,
        interests,
    })
}
```

### API Query Validation

```rust
fn validate_api_query(query: QueryParams) -> Result<ApiQuery, Vec<QueryParamError>> {
    let mut errors = Vec::new();

    // Validate required parameters
    let user_id = match query.get_i64("user_id") {
        Ok(id) => Some(id),
        Err(e) => {
            errors.push(e);
            None
        }
    };

    // Validate pagination
    let page = query.page();
    if page < 1 {
        errors.push(QueryParamError::ParseError {
            param: "page".to_string(),
            value: page.to_string(),
            target_type: "positive integer".to_string(),
        });
    }

    let limit = query.limit();
    if limit > 1000 {
        errors.push(QueryParamError::ParseError {
            param: "limit".to_string(),
            value: limit.to_string(),
            target_type: "integer <= 1000".to_string(),
        });
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(ApiQuery {
        user_id: user_id.unwrap(),
        page,
        limit,
    })
}
```

## Performance Considerations

- Uses `HashMap<String, Vec<String>>` internally for efficient parameter lookup
- URL decoding is performed once during parsing
- Type conversion is lazy - only happens when requested
- Memory usage scales with the number of unique parameters and their values
- Multiple values are stored efficiently in vectors

## Thread Safety

`QueryParams` implements `Clone` and `Debug`, making it suitable for use in multi-threaded web servers. The struct can be safely shared across threads by cloning when needed.

## Migration from HashMap

```rust
use std::collections::HashMap;
use ripress::req::query::QueryParams;

// Before: using HashMap directly
fn old_handler(params: HashMap<String, String>) -> Result<Response, Error> {
    let page_str = params.get("page").unwrap_or("1");
    let page: i32 = page_str.parse().map_err(|_| Error::InvalidPage)?;

    let tags: Vec<&str> = params.get("tags")
        .map(|s| s.split(',').collect())
        .unwrap_or_default();

    process_request(page, tags)
}

// After: using QueryParams
fn new_handler(query: QueryParams) -> Result<Response, Error> {
    let page = query.page(); // Handles defaults and parsing
    let tags = query.get_all_parsed::<String>("tags").unwrap_or_default();

    process_request(page, tags)
}

// Migration helper
fn transitional_handler(old_params: HashMap<String, String>) -> Result<Response, Error> {
    let query = QueryParams::from(old_params);
    new_handler(query)
}
```
