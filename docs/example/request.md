# Request Object Examples

## Overview

The `HttpRequest` object in Ripress provides various methods to extract and manipulate incoming request data. This document covers examples for different request handling scenarios.

## Retrieving Request Data

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

    app.listen(3000, || {}).await;
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

    app.get("/", handler);

    app.listen(3000, || {}).await;
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

    app.listen(3000, || {}).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let origin_url = req.origin_url;
    res.ok().text(format!("Origin URL: {}", origin_url))
}
```

## Handling Query Parameters

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

    app.listen(3000, || {}).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.query.get("q") {
        Some(query) => res.ok().text(format!("Search query: {}", query)),
        None => res.bad_request().text("Query parameter 'q' is missing"),
    }
}
```

## Getting Client's IP Address

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

    app.listen(3000, || {}).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let ip = req.ip;
    res.ok().text(format!("Client IP: {}", ip))
}
```

## Handling Route Parameters

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/:id", handler);

    app.listen(3000, || {}).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.params.get("id") {
        Some(user_id) => res.ok().text(format!("User ID: {}", user_id)),
        None => res.bad_request().text("User ID is missing"),
    }
}
```

## Handling Headers

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

    app.listen(3000, || {}).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.headers.get("authorization") {
        Some(auth) => res.ok().text(format!("Auth Header: {}", auth)),
        None => res.unauthorized().text("Missing Authorization header"),
    }
}
```

## Handling Cookies

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

    app.listen(3000, || {}).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.get_cookie("sessionId") {
        Some(cookie) => res.ok().text(format!("Session ID: {}", cookie)),
        None => res.unauthorized().text("Session cookie missing"),
    }
}
```

## Handling JSON Body

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
}

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || {}).await;
}
async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.json::<User>() {
        Ok(user) => res.ok().text(format!("Received user: {}", user.name)),
        Err(e) => res.bad_request().text(format!("Invalid JSON body: {}", e)),
    }
}
```

## Handling BINARY Body

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.post("/", handler);

    app.listen(3000, || {}).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.bytes() {
        Ok(bytes) => res.ok().bytes(bytes),
        Err(e) => res.bad_request().text(format!("Invalid binary body: {}", e)),
    }
}
```

## Handling Form Data

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

    app.listen(3000, || {}).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.form_data() {
        Ok(form) => res.ok().text(format!("Received form data: {:?}", form)),
        Err(e) => res.bad_request().text(format!("Invalid form data: {}", e)),
    }
}
```

## Handling Text Body

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

    app.listen(3000, || {}).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.text() {
        Ok(body) => res.ok().text(format!("Received text: {}", body)),
        Err(e) => res.bad_request().text(format!("Invalid text body: {}", e)),
    }
}
```

## Checking Content Type

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::{RequestBodyType, RouterFns},
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || {}).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    if req.is(RequestBodyType::JSON) {
        res.ok().text("Request is JSON")
    } else {
        res.bad_request().text("Unsupported content type")
    }
}
```

## Checking Protocol

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

    app.listen(3000, || {}).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let protocol = req.protocol;
    res.ok().text(format!("Protocol: {}", protocol))
}
```

## Checking Is Secure

````rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", handler);

    app.listen(3000, || {}).await;
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let is_secure: bool = req.is_secure;
    res.ok().text(format!("Is HTTPS: {}", is_secure))
}

## Working with Middleware Data

### File Upload Middleware

The file upload middleware processes binary file uploads and makes file information available via request data:

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
    // Check if file was uploaded successfully
    if let Some(uploaded_file) = req.get_data("uploaded_file") {
        let uploaded_file_path = req.get_data("uploaded_file_path").unwrap_or_default();

        res.ok().json(serde_json::json!({
            "success": true,
            "filename": uploaded_file,
            "path": uploaded_file_path,
            "message": "File uploaded successfully"
        }))
    } else {
        // No file was uploaded, but request continues normally
        res.ok().json(serde_json::json!({
            "success": false,
            "message": "No file uploaded or upload failed"
        }))
    }
}
````

### Custom Middleware with Data

You can also create custom middleware that sets data for route handlers:

```rust
use ripress::{
    app::App,
    context::{HttpRequest, HttpResponse},
    types::RouterFns,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Custom middleware that sets user data
    app.use_middleware("/api/", |req, res| async move {
        let mut req = req.clone();

        // Set user information from headers or other sources
        req.set_data("user_id", "12345");
        req.set_data("user_role", "admin");

        (req, None) // Continue processing
    });

    app.get("/api/profile", profile_handler);

    app.listen(3000, || {}).await;
}

async fn profile_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let user_id = req.get_data("user_id").unwrap_or_default();
    let user_role = req.get_data("user_role").unwrap_or_default();

    res.ok().json(serde_json::json!({
        "user_id": user_id,
        "user_role": user_role,
        "message": "Profile data retrieved from middleware"
    }))
}
```
