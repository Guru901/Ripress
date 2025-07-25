# Request Object (HttpRequest)

## Overview

`HttpRequest` represents an incoming HTTP request and provides utilities for accessing query parameters, request headers, body content, and more.

## Creating a Request Object

HttpRequest is automatically passed to route handlers.

Example:

```rust
use ripress::{context::{HttpRequest, HttpResponse}};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.text().unwrap_or("No body".to_string());
    res.ok().text(format!("Received: {}", body))
}
```

## Checking Content-Type

Checks if the `Content-Type` of the request matches the specified type.

```rust
use ripress::{req::HttpRequest, res::HttpResponse, types::RequestBodyType};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    req.is(RequestBodyType::JSON);

    res.ok()
}
```

Returns `true` if the `Content-Type` matches, otherwise `false`.

## Getting Request Method

Returns the request's HTTP method.

```rust
use ripress::{req::HttpRequest, res::HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let req = HttpRequest::new();
    let method = req.method;

    res.ok()
}
```

Returns a reference to `HttpMethods` enum.

## Getting Request Origin URL

Returns the request's origin URL.

```rust
use ripress::{req::HttpRequest, res::HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let origin_url = req.origin_url;

    res.ok()
}
```

### Example Cases:

- For request: `GET /user/123`
  - origin_url → `/user/123`
- For request: `GET /user/123?q=hello`
  - origin_url → `/user/123?q=hello`

## Getting Request Path

Returns the request's path.

```rust
use ripress::{req::HttpRequest, res::HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let req = HttpRequest::new();
    let path = req.path;

    res.ok()
}
```

### Example Cases:

- For request: `GET /user/123?q=hello`
  - path → `/user/123`

## Getting Request Cookies

Returns the specified cookie value.

```rust
use ripress::{req::HttpRequest, res::HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.get_cookie("session_id") {
        Ok(value) => println!("Cookie: {}", value),
        Err(e) => println!("Error: {:?}", e),
    }

    res.ok()
}
```

Returns `Result<&str, HttpRequestError>`.

## Getting Client's IP Address

Returns the client's IP address.

```rust
use ripress::{req::HttpRequest, res::HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let ip = req.ip;

    res.ok()
}
```

## Getting Request Headers

Returns the specified header value.

```rust
use ripress::{req::HttpRequest, res::HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.get_header("content-type") {
        Ok(value) => println!("Header: {}", value),
        Err(e) => println!("Error: {:?}", e),
    }

    res.ok()
}
```

Returns `Result<&str, HttpRequestError>`.

## Accessing URL Parameters

```rust
use ripress::{req::HttpRequest, res::HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.get_params("id") {
        Ok(value) => println!("ID: {}", value),
        Err(e) => println!("Error: {:?}", e),
    }

    res.ok()
}
```

Example:

- Route: `GET /user/{id}`
- Request: `GET /user/123`
- `get_params("id")` returns `Ok("123")`

Returns `Result<&str, HttpRequestError>`.

## Accessing Query Parameters

```rust
use ripress::{req::HttpRequest, res::HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.get_query("q") {
        Ok(value) => println!("Query: {}", value),
        Err(e) => println!("Error: {:?}", e),
    }

    res.ok()
}
```

Example:

- URL: `GET /search?q=Ripress`
- `get_query("q")` returns `Ok("Ripress")`

Returns `Result<&str, HttpRequestError>`.

## Getting Request Protocol

Returns the request's protocol.

```rust
use ripress::{req::HttpRequest, res::HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let protocol = req.protocol;
    println!("Protocol: {}", protocol); // "http" or "https"

    res.ok()
}
```

Returns `String` containing the protocol.

## Checking If Request Is Secure

Returns whether the request was made over HTTPS.

```rust
use ripress::{req::HttpRequest, res::HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let is_secure = req.is_secure;
    println!("Is Secure: {}", is_secure);

    res.ok()
}
```

Returns `bool`.

## Get data from request that is inserted by middleware

```rust
use ripress::{req::HttpRequest, res::HttpResponse};

async fn handler(mut req: HttpRequest, res: HttpResponse) -> HttpResponse {
    req.set_data("id", "123");
    let id = req.get_data("id");
    println!("Id: {:?}", id);

    res.ok()
}
```

Returns `Option<&String>` with the data value if found, or `None` if not found.

## Reading Request Body

### JSON Body

```rust
use ripress::{req::HttpRequest, res::HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct User {
    name: String,
    age: u8,
}

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.json::<User>() {
        Ok(user) => println!("Name: {}, Age: {}", user.name, user.age),
        Err(e) => println!("Error: {}", e),
    }

    res.ok()
}
```

Returns `Result<J, String>` where `J` is your deserialized type.

### Text Body

```rust
use ripress::{req::HttpRequest, res::HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.text() {
        Ok(text) => println!("Text: {}", text),
        Err(e) => println!("Error: {}", e),
    }

    res.ok()
}
```

Returns `Result<String, String>`.

### Form Data

```rust
use ripress::{req::HttpRequest, res::HttpResponse};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    match req.form_data() {
        Ok(form) => {
            println!("Key: {:?}", form.get("key"));
            println!("Key2: {:?}", form.get("key2"));
        }
        Err(e) => println!("Error: {}", e),
    }
    res.ok()
}
```

Returns `Result<HashMap<String, String>, String>`.
