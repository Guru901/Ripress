# Response Examples

## Basic Responses

### Sending a Plain Text Response

```rust
async fn text_response(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello, World!")
}
```

### Sending a JSON Response

```rust
#[derive(serde::Serialize)]
struct Message {
    message: String,
}

async fn json_response(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let response_body = Message {
        message: "Success".to_string(),
    };
    res.ok().json(&response_body)
}
```

---

## Sending Status Codes

```rust
async fn example(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(404)
}
```

## Status code helpers

### Sending a 200 Ok Response

```rust
async fn ok(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Success")
}
```

### Sending a 404 Not Found Response

```rust
async fn not_found(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.not_found().text("Page not found")
}
```

### Sending a 400 Bad Request Response

```rust
async fn bad_request(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.bad_request().text("Invalid request")
}
```

### Sending a 500 Internal Server Error Response

```rust
async fn internal_error(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.internal_server_error().text("Something went wrong")
}
```

---
