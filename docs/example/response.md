# Response Examples

The `HttpResponse` object in Ripress provides various methods for handling responses, including sending text, JSON, status codes, and cookies. This document demonstrates different response-handling scenarios.

## Basic Responses

### Sending a Plain Text Response

Use `.text()` to send a plain text response.

```rust
async fn text_response(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello, World!")
}
```

### Sending a JSON Response

To return a JSON response, use `.json()` with a serializable Rust struct.

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

## Status Codes

### Setting a Custom Status Code

You can manually set a status code using `.status()`.

```rust
async fn custom_status(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.status(418).text("I'm a teapot")
}
```

### Status Code Helpers

Ripress provides convenient helper methods for common status codes.

#### **200 OK**

```rust
async fn ok_response(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Request successful")
}
```

#### **400 Bad Request**

```rust
async fn bad_request(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.bad_request().text("Invalid request")
}
```

#### **404 Not Found**

```rust
async fn not_found(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.not_found().text("Resource not found")
}
```

#### **500 Internal Server Error**

```rust
async fn internal_error(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.internal_server_error().text("An unexpected error occurred")
}
```

---

## Headers and Cookies

### Setting a Response Header

Use `.set_header()` to modify the response headers.

```rust
async fn custom_header(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_header("X-Custom-Header", "MyValue")
        .ok()
        .text("Header added")
}
```

### Sending Cookies

Use `.set_cookie()` to attach a cookie to the response.

```rust
async fn cookie_response(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.set_cookie("session", "abc123; HttpOnly")
        .ok()
        .json(json!({ "message": "Cookie set" }))
}
```

---
