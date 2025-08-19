# Ripress HttpRequest API Reference

## Overview

The `HttpRequest` object in Ripress provides various methods to extract and manipulate incoming request data. This document covers examples for different request handling scenarios.

## Basic Request Properties

### Getting HTTP Method

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let method = req.method;
    res.ok().text(format!("Request method: {:?}", method))
}
```

### Getting Request Path

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/*", handler); // Match any path

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let path = req.path;
    res.ok().text(format!("Request path: {}", path))
}
```

### Getting Origin URL

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let origin_url = req.origin_url;
    res.ok().text(format!("Origin URL: {}", origin_url))
}
```

### Getting Client IP Address

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let ip = req.ip;
    res.ok().text(format!("Client IP: {}", ip))
}
```

## Query Parameters

### Handling Single Query Parameter

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Example: GET /?q=rust&page=1
    app.get("/search", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.query.get("q") {
        Some(query) => res.ok().text(format!("Search query: {}", query)),
        None => res.bad_request().text("Query parameter 'q' is missing"),
    }
}
```

### Handling Multiple Query Parameters

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Example: GET /search?q=rust&page=1&limit=10
    app.get("/search", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let query = req.query.get("q").unwrap_or("default");
    let page = req.query.get("page").unwrap_or("1");
    let limit = req.query.get("limit").unwrap_or("10");

    res.ok().json(serde_json::json!({
        "query": query,
        "page": page,
        "limit": limit,
        "results": []
    }))
}
```

## Route Parameters

### Single Route Parameter

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Example: GET /users/123
    app.get("/users/:id", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.params.get("id") {
        Some(user_id) => res.ok().text(format!("User ID: {}", user_id)),
        None => res.bad_request().text("User ID is missing"),
    }
}
```

### Multiple Route Parameters

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Example: GET /users/123/posts/456
    app.get("/users/:userId/posts/:postId", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.params.get("userId").unwrap_or("unknown");
    let post_id = req.params.get("postId").unwrap_or("unknown");

    res.ok().json(serde_json::json!({
        "userId": user_id,
        "postId": post_id,
        "action": "get_user_post"
    }))
}
```

## Headers

### Reading Request Headers

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/protected", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.headers.get("authorization") {
        Some(auth) => res.ok().text(format!("Auth Header: {}", auth)),
        None => res.status(401).text("Missing Authorization header"),
    }
}
```

### Reading Multiple Headers

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/info", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_agent = req.headers.get("user-agent").unwrap_or("Unknown");
    let content_type = req.headers.get("content-type").unwrap_or("Not specified");
    let accept = req.headers.get("accept").unwrap_or("*/*");

    res.ok().json(serde_json::json!({
        "userAgent": user_agent,
        "contentType": content_type,
        "accept": accept
    }))
}
```

## Cookies

### Reading Single Cookie

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/dashboard", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.get_cookie("sessionId") {
        Some(cookie) => res.ok().text(format!("Session ID: {}", cookie)),
        None => res.status(401).text("Session cookie missing"),
    }
}
```

### Reading Multiple Cookies

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/profile", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let session_id = req.get_cookie("sessionId").unwrap_or("none");
    let user_prefs = req.get_cookie("userPrefs").unwrap_or("default");
    let theme = req.get_cookie("theme").unwrap_or("light");

    res.ok().json(serde_json::json!({
        "sessionId": session_id,
        "userPrefs": user_prefs,
        "theme": theme
    }))
}
```

## Request Body Handling

### JSON Body

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct User {
    name: String,
    email: String,
    age: Option<u32>,
}

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.post("/users", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.json::<User>() {
        Ok(user) => res.status(201).json(serde_json::json!({
            "message": format!("User {} created successfully", user.name),
            "user": user
        })),
        Err(e) => res.bad_request().json(serde_json::json!({
            "error": "Invalid JSON body",
            "details": e.to_string()
        })),
    }
}
```

### Form Data

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.post("/submit", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.form_data() {
        Ok(form) => {
            let name = form.get("name").unwrap_or(&"Anonymous".to_string());
            let email = form.get("email").unwrap_or(&"no-email@example.com".to_string());

            res.ok().json(serde_json::json!({
                "message": "Form submitted successfully",
                "data": {
                    "name": name,
                    "email": email
                }
            }))
        },
        Err(e) => res.bad_request().json(serde_json::json!({
            "error": "Invalid form data",
            "details": e.to_string()
        })),
    }
}
```

### Text Body

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.post("/echo", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.text() {
        Ok(body) => res.ok().json(serde_json::json!({
            "received": body,
            "length": body.len()
        })),
        Err(e) => res.bad_request().json(serde_json::json!({
            "error": "Invalid text body",
            "details": e.to_string()
        })),
    }
}
```

## Request Data Methods

### Middleware Data Communication

Middleware can set data that handlers can later access. This is commonly used by built-in middleware like file upload processing:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

// Example middleware that sets request data
async fn custom_middleware(mut req: HttpRequest, res: HttpResponse) -> (HttpRequest, Option<HttpResponse>) {
    // Middleware sets data for handlers
    req.set_data("processed_at", "2023-01-01T12:00:00Z");
    req.set_data("middleware_version", "v1.0");

    (req, None) // Continue to next middleware/handler
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // Handler reads data set by middleware
    let processed_at = req.get_data("processed_at").unwrap_or("unknown");
    let version = req.get_data("middleware_version").unwrap_or("unknown");

    res.ok().json(serde_json::json!({
        "processedAt": processed_at,
        "middlewareVersion": version
    }))
}
```

**Note:** `req.set_data()` is typically used by middleware, while `req.get_data()` is used by handlers to access middleware-provided data.

## Content Type Checking

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::{RequestBodyType, RouterFns},
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.post("/upload", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    if req.is(RequestBodyType::JSON) {
        res.ok().text("Processing JSON data...")
    } else if req.is(RequestBodyType::FormData) {
        res.ok().text("Processing form data...")
    } else {
        res.bad_request().text("Unsupported content type")
    }
}
```

## Protocol and Security

### Checking Protocol

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/info", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let protocol = req.protocol;
    res.ok().json(serde_json::json!({
        "protocol": protocol,
        "isSecure": req.is_secure
    }))
}
```

### Checking HTTPS

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/secure-only", handler);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    if req.is_secure {
        res.ok().json(serde_json::json!({
            "message": "Access granted to secure endpoint",
            "protocol": "HTTPS"
        }))
    } else {
        res.status(426).json(serde_json::json!({
            "error": "HTTPS required",
            "message": "This endpoint requires a secure connection"
        }))
    }
}
```

## Complete Request Information

### Request Inspector

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.all("/inspect", handler); // Handle all HTTP methods

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // Collect all available request information
    let request_info = serde_json::json!({
        "method": format!("{:?}", req.method),
        "path": req.path,
        "originUrl": req.origin_url,
        "protocol": req.protocol,
        "isSecure": req.is_secure,
        "clientIp": req.ip,
        "headers": req.headers,
        "queryParams": req.query,
        "routeParams": req.params,
    });

    res.ok().json(request_info)
}
```

## Working with File Upload Middleware

### File Upload Processing

The file upload middleware processes binary file uploads and makes file information available via request data. It supports both raw binary uploads and `multipart/form-data` from browsers:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    middlewares::file_upload::file_upload,
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Add file upload middleware
    app.use_middleware("/upload", file_upload(None));

    // Handle file uploads
    app.post("/upload", upload_handler);

    app.listen(3000, || {}).await;
}

async fn upload_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // Check if files were uploaded successfully using request data
    if let Some(count_str) = req.get_data("uploaded_file_count") {
        let count: usize = count_str.parse().unwrap_or(0);

        if count > 0 {
            // Get comprehensive file information from request data
            if let Some(files_json) = req.get_data("uploaded_files") {
                // For backward compatibility, first file info is also available
                let first_file = req.get_data("uploaded_file");
                let first_path = req.get_data("uploaded_file_path");

                res.ok().json(serde_json::json!({
                    "success": true,
                    "count": count,
                    "files": serde_json::from_str::<serde_json::Value>(&files_json).unwrap(),
                    "firstFile": first_file,
                    "firstPath": first_path
                }))
            } else {
                res.ok().json(serde_json::json!({
                    "success": true,
                    "count": count,
                    "message": format!("Successfully uploaded {} files", count)
                }))
            }
        } else {
            res.ok().json(serde_json::json!({
                "success": false,
                "message": "No files were uploaded"
            }))
        }
    } else {
        res.ok().json(serde_json::json!({
            "success": false,
            "message": "No files were uploaded"
        }))
    }
}
```

### Accessing Form Fields with File Upload

For multipart forms, the middleware extracts both text fields and file fields:

```rust
async fn upload_with_metadata(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // Access form fields (including file field to UUID mappings)
    if let Ok(form_data) = req.form_data() {
        // Text fields are available normally
        let description = form_data.get("description").unwrap_or(&"".to_string());
        let category = form_data.get("category").unwrap_or(&"uncategorized".to_string());

        // File fields are mapped to UUID filenames
        let profile_pic_uuid = form_data.get("profile_pic").unwrap_or(&"".to_string());

        println!("Upload description: {}", description);
        println!("Category: {}", category);
        println!("Profile pic saved as: {}", profile_pic_uuid);
    }

    // Get file upload details from request data
    if let Some(count_str) = req.get_data("uploaded_file_count") {
        let count: usize = count_str.parse().unwrap_or(0);

        res.ok().json(serde_json::json!({
            "filesUploaded": count,
            "message": "Upload processed successfully"
        }))
    } else {
        res.ok().json(serde_json::json!({
            "filesUploaded": 0,
            "message": "No files uploaded"
        }))
    }
}
```

**Important Notes:**

- File information is accessed via `req.get_data()` (set by middleware)
- Form field data is accessed via `req.form_data()` (from HTTP request)
- File input field names are mapped to generated UUID filenames in form data
- Original filenames may not be preserved due to current middleware limitations

## Error Handling Patterns

### Robust Parameter Extraction

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/users/:id", get_user);
    app.put("/users/:id", update_user);

    app.listen(3000, || println!("Server running on port 3000")).await;
}

async fn get_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = match req.params.get("id") {
        Some(id) => {
            // Validate the ID is numeric
            match id.parse::<u32>() {
                Ok(parsed_id) => parsed_id,
                Err(_) => {
                    return res.bad_request().json(serde_json::json!({
                        "error": "Invalid user ID format",
                        "message": "User ID must be a number"
                    }));
                }
            }
        }
        None => {
            return res.bad_request().json(serde_json::json!({
                "error": "Missing user ID",
                "message": "User ID parameter is required"
            }));
        }
    };

    res.ok().json(serde_json::json!({
        "userId": user_id,
        "name": "John Doe",
        "email": "john@example.com"
    }))
}

async fn update_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    // Extract and validate user ID
    let user_id = match req.params.get("id").and_then(|id| id.parse::<u32>().ok()) {
        Some(id) => id,
        None => {
            return res.bad_request().json(serde_json::json!({
                "error": "Invalid or missing user ID"
            }));
        }
    };

    // Parse JSON body
    #[derive(serde::Deserialize)]
    struct UpdateUser {
        name: Option<String>,
        email: Option<String>,
    }

    let update_data: UpdateUser = match req.json() {
        Ok(data) => data,
        Err(e) => {
            return res.bad_request().json(serde_json::json!({
                "error": "Invalid JSON body",
                "details": e.to_string()
            }));
        }
    };

    res.ok().json(serde_json::json!({
        "message": "User updated successfully",
        "userId": user_id,
        "updates": update_data
    }))
}
```

## Quick Reference

### Request Properties

- `req.method` - HTTP method (GET, POST, etc.)
- `req.path` - Request path
- `req.origin_url` - Full origin URL
- `req.ip` - Client IP address
- `req.protocol` - HTTP protocol version
- `req.is_secure` - Whether connection is HTTPS

### Parameter Access

- `req.query.get("param")` - Query parameter
- `req.params.get("param")` - Route parameter
- `req.headers.get("header")` - Request header
- `req.get_cookie("name")` - Cookie value

### Body Parsing

- `req.json::<T>()` - Parse JSON body to struct
- `req.form_data()` - Parse form data
- `req.text()` - Get raw text body
- `req.is(RequestBodyType::JSON)` - Check content type

### Request Data Methods (Middleware Communication)

- `req.get_data("key")` - Get data set by middleware
- `req.set_data("key", "value")` - Set data for handlers (middleware only)

### File Upload Data (When using file_upload middleware)

- `req.get_data("uploaded_file_count")` - Number of uploaded files
- `req.get_data("uploaded_files")` - JSON array of file information
- `req.get_data("uploaded_file")` - First file's UUID filename
- `req.get_data("uploaded_file_path")` - First file's full path
- Form fields map file input names to UUID filenames

### Return Types

- All header/query/param getters return `Option<&str>`
- Body parsers return `Result<T, Error>`
- Cookie getter returns `Option<String>`
- Request data methods return `Option<&str>`
