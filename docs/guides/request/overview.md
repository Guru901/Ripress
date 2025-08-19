# Request Handling

In Ripress, the `HttpRequest` object is your gateway to understanding and processing incoming HTTP requests. It provides comprehensive access to all aspects of the request, from basic metadata like the HTTP method and path to complex data extraction from headers, parameters, and request bodies.

## What is HttpRequest?

The `HttpRequest` struct is automatically created by Ripress for each incoming request and passed as the first parameter to your route handlers. It encapsulates all the information about the client's request and provides convenient methods to extract and work with that data.

```rust
async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // The req parameter contains everything about the incoming request
    let method = req.method;        // GET, POST, PUT, etc.
    let path = req.path;           // "/users/123"
    let ip = req.ip;               // Client's IP address

    res.ok().text("Hello!")
}
```

## Core Request Information

Every `HttpRequest` provides access to fundamental request data:

- **HTTP Method** - GET, POST, PUT, DELETE, etc.
- **Request Path** - The URL path without query parameters
- **Origin URL** - The complete original URL
- **Client IP** - The requesting client's IP address
- **Protocol** - HTTP or HTTPS
- **Security Status** - Whether the connection is secure (HTTPS)

## Data Extraction Capabilities

Ripress makes it easy to extract data from different parts of the request:

### URL Components

- **Route Parameters** - Dynamic segments in your URL patterns (`/users/:id`)
- **Query Parameters** - Key-value pairs from the URL query string (`?search=term&page=1`)

### Headers and Metadata

- **HTTP Headers** - Access any request header by name
- **Cookies** - Extract cookie values with built-in parsing
- **Content-Type Detection** - Check the request's content type

### Request Body

- **JSON** - Automatic deserialization to your Rust structs
- **Form Data** - Parse URL-encoded form submissions
- **BINARY** - Access raw bytes from the request body
- **Plain Text** - Raw text content from the request body

## Key Features

### Type Safety

Ripress leverages Rust's type system to provide safe data extraction:

```rust
#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn create_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.json::<CreateUser>() {
        Ok(user_data) => {
            // user_data is strongly typed
            println!("Creating user: {}", user_data.name);
            res.ok().text("User created")
        }
        Err(_) => res.bad_request().text("Invalid user data")
    }
}
```

### Error Handling

All data extraction methods return `Result` types, making error handling explicit and safe:

```rust
async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.query.get("page") {
        Some(page_str) => match page_str.parse::<u32>() {
            Ok(page) => res.ok().text(format!("Page: {}", page)),
            Err(_) => res.bad_request().text("Invalid page number")
        },
        None => res.bad_request().text("Page parameter required")
    }
}
```

### Middleware Integration

The request object can carry data between middleware and handlers:

```rust
// In middleware
let mut req = req.clone();
req.set_data("user_id", &user_id);

// In handler
async fn protected_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    if let Some(user_id) = req.get_data("user_id") {
        // Access user data set by middleware
        res.ok().text(format!("Welcome, user {}", user_id))
    } else {
        res.unauthorized().text("Access denied")
    }
}
```

## Request Lifecycle

1. **Request Arrives** - Ripress receives an HTTP request
2. **Parsing** - The request is parsed into an `HttpRequest` object
3. **Middleware Processing** - Request flows through middleware chain
4. **Route Matching** - Ripress finds the appropriate handler
5. **Handler Execution** - Your handler receives the `HttpRequest`
6. **Data Extraction** - You extract needed data using request methods
7. **Response Generation** - You return an `HttpResponse`

## Common Patterns

### Parameter Validation

```rust
async fn get_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.params.get("id") {
        Some(id) => match id.parse::<u64>() {
            Ok(user_id) if user_id > 0 => {
                // Valid user ID
                res.ok().text(format!("User {}", user_id))
            }
            _ => res.bad_request().text("Invalid user ID")
        },
        None => res.bad_request().text("User ID required")
    }
}
```

### Content Negotiation

```rust
use ripress::types::RequestBodyType;

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    if req.is(RequestBodyType::JSON) {
        // Handle JSON request
        match req.json::<serde_json::Value>() {
            Ok(json) => res.ok().json(json),
            Err(_) => res.bad_request().text("Invalid JSON")
        }
    } else {
        res.unsupported_media_type().text("JSON required")
    }
}
```

## What's Next?

Now that you understand the basics of request handling, explore these specific topics:

- **[Route Parameters](route-params.md)** - Extract dynamic values from URLs
- **[Query Parameters](query-params.md)** - Parse and validate query strings
- **[Request Headers](request-headers.md)** - Work with HTTP headers and metadata
- **[Request Body](request-body.md)** - Handle JSON, forms, and file uploads
- **[Request Data](request-data.md)** - Handle JSON, forms, and file uploads

The `HttpRequest` object is designed to make request handling in Ripress both powerful and ergonomic, letting you focus on your application logic rather than HTTP parsing details.
