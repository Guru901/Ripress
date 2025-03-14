# Request Object Examples

## Overview

The `HttpRequest` object in Ripress provides various methods to extract and manipulate incoming request data. This document covers examples for different request handling scenarios.

---

## Retrieving Request Data

### Getting HTTP Method

```rust
async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let method = req.get_method();
    res.ok().text(format!("Request method: {}", method))
}
```

### Getting Request Path

```rust
async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let path = req.get_path();
    res.ok().text(format!("Request path: {}", path))
}
```

### Getting Full Request URL

```rust
async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let url = req.get_origin_url();
    res.ok().text(format!("Full URL: {}", url))
}
```

---

## Handling Query Parameters

```rust
async fn search(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let query = req.get_query("q").unwrap_or_else(|| "No query provided".to_string());
    res.ok().text(format!("Search query: {}", query))
}
```

---

## Getting Client's IP Address

```rust
async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let ip = req.ip();
    res.ok().text(format!("Client IP: {}", ip))
}
```

---

## Handling Route Parameters

```rust
async fn get_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    if let Some(user_id) = req.get_params("id") {
        res.ok().text(format!("User ID: {}", user_id))
    } else {
        res.bad_request().text("User ID is missing")
    }
}
```

---

## Handling Headers

```rust
async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    if let Some(auth) = req.get_header("Authorization") {
        res.ok().text(format!("Auth Header: {}", auth))
    } else {
        res.unauthorized().text("Missing Authorization header")
    }
}
```

---

## Handling Cookies

```rust
async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    if let Some(cookie) = req.get_cookie("sessionId") {
        res.ok().text(format!("Session ID: {}", cookie))
    } else {
        res.unauthorized().text("Session cookie missing")
    }
}
```

---

## Handling JSON Body

```rust
#[derive(serde::Deserialize)]
struct User {
    name: String,
    email: String,
}

async fn save_user(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.json::<User>() {
        Ok(user) => res.ok().text(format!("Received user: {}", user.name)),
        Err(_) => res.bad_request().text("Invalid JSON body"),
    }
}
```

---

## Handling Form Data

```rust
async fn upload(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.form_data() {
        Ok(form) => res.ok().text(format!("Received form data: {:?}", form)),
        Err(_) => res.bad_request().text("Invalid form data"),
    }
}
```

---

## Handling Text Body

```rust
async fn log_text(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.text() {
        Ok(body) => res.ok().text(format!("Received text: {}", body)),
        Err(_) => res.bad_request().text("Invalid text body"),
    }
}
```

---

## Checking Content Type

```rust
async fn check_content_type(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    if req.is("application/json") {
        res.ok().text("Request is JSON")
    } else {
        res.bad_request().text("Unsupported content type")
    }
}
```

## Checking Protocol

```rust
async fn check_protocol(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let protocol = req.get_protocol();
}
```

## Checking Is Secure

```rust
async fn check_is_secure(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let is_secure = req.is_secure();
}
```
