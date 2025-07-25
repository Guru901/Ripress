# Request Object Examples

## Overview

The `HttpRequest` object in Ripress provides various methods to extract and manipulate incoming request data. This document covers examples for different request handling scenarios.

## Retrieving Request Data

### Getting HTTP Method

```rust
use ripress::{context::{HttpRequest, HttpResponse}, types::HttpMethods};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let method: &HttpMethods = req.get_method();
    res.ok().text(format!("Request method: {:?}", method))
}
```

### Getting Request Path

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let path: &str = req.get_path();
    res.ok().text(format!("Request path: {}", path))
}
```

### Getting Full Request URL

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.get_origin_url() {
        Ok(url) => res.ok().text(format!("Full URL: {}", url)),
        Err(e) => res.internal_server_error().text(format!("Error: {}", e))
    }
}
```

## Handling Query Parameters

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn search(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.get_query("q") {
        Ok(query) => res.ok().text(format!("Search query: {}", query)),
        Err(_) => res.bad_request().text("Query parameter 'q' is missing")
    }
}
```

## Getting Client's IP Address

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.ip() {
        Ok(ip) => res.ok().text(format!("Client IP: {}", ip)),
        Err(e) => res.internal_server_error().text(format!("Error: {}", e))
    }
}
```

## Handling Route Parameters

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn get_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.get_params("id") {
        Ok(user_id) => res.ok().text(format!("User ID: {}", user_id)),
        Err(_) => res.bad_request().text("User ID is missing")
    }
}
```

## Handling Headers

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.get_header("authorization") {
        Ok(auth) => res.ok().text(format!("Auth Header: {}", auth)),
        Err(_) => res.unauthorized().text("Missing Authorization header")
    }
}
```

## Handling Cookies

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.get_cookie("sessionId") {
        Ok(cookie) => res.ok().text(format!("Session ID: {}", cookie)),
        Err(_) => res.unauthorized().text("Session cookie missing")
    }
}
```

## Handling JSON Body

```rust
use ripress::context::{HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct User {
    name: String,
    email: String,
}

async fn save_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.json::<User>() {
        Ok(user) => res.ok().text(format!("Received user: {}", user.name)),
        Err(e) => res.bad_request().text(format!("Invalid JSON body: {}", e))
    }
}
```

## Handling Form Data

```rust
use ripress::context::{HttpRequest, HttpResponse};
use std::collections::HashMap;

async fn upload(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.form_data() {
        Ok(form) => res.ok().text(format!("Received form data: {:?}", form)),
        Err(e) => res.bad_request().text(format!("Invalid form data: {}", e))
    }
}
```

## Handling Text Body

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn log_text(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.text() {
        Ok(body) => res.ok().text(format!("Received text: {}", body)),
        Err(e) => res.bad_request().text(format!("Invalid text body: {}", e))
    }
}
```

## Checking Content Type

```rust
use ripress::{context::{HttpRequest, HttpResponse}, types::RequestBodyType};

async fn check_content_type(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    if req.is(RequestBodyType::JSON) {
        res.ok().text("Request is JSON")
    } else {
        res.bad_request().text("Unsupported content type")
    }
}
```

## Checking Protocol

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn check_protocol(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let protocol: &str = req.get_protocol();
    res.ok().text(format!("Protocol: {}", protocol))
}
```

## Checking Is Secure

```rust
use ripress::context::{HttpRequest, HttpResponse};

async fn check_is_secure(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let is_secure: bool = req.is_secure();
    res.ok().text(format!("Is HTTPS: {}", is_secure))
}
```
