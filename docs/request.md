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
use ripress::{context::HttpRequest, types::RequestBodyType};

let req = HttpRequest::new();
req.is(RequestBodyType::JSON);
```

Returns `true` if the `Content-Type` matches, otherwise `false`.

## Getting Request Method

Returns the request's HTTP method.

```rust
use ripress::{context::HttpRequest, types::HttpMethods};

let req = HttpRequest::new();
let method: &HttpMethods = req.get_method();
```

Returns a reference to `HttpMethods` enum.

## Getting Request Origin URL

Returns the request's origin URL.

```rust
use ripress::context::HttpRequest;

let req = HttpRequest::new();
match req.get_origin_url() {
    Ok(url) => println!("URL: {}", url),
    Err(e) => println!("Error: {}", e)
}
```

### Example Cases:

- For request: `GET /user/123`
  - origin_url → `/user/123`
- For request: `GET /user/123?q=hello`
  - origin_url → `/user/123?q=hello`

Returns `Result<&str, &str>`.

## Getting Request Path

Returns the request's path.

```rust
use ripress::context::HttpRequest;

let req = HttpRequest::new();
let path: &str = req.get_path();
```

### Example Cases:

- For request: `GET /user/123?q=hello`
  - path → `/user/123`

Returns `&str`.

## Getting Request Cookies

Returns the specified cookie value.

```rust
use ripress::context::HttpRequest;

let req = HttpRequest::new();
match req.get_cookie("session_id") {
    Ok(value) => println!("Cookie: {}", value),
    Err(e) => println!("Error: {:?}", e)
}
```

Returns `Result<&str, HttpRequestError>`.

## Getting Client's IP Address

Returns the client's IP address.

```rust
use ripress::context::HttpRequest;

let req = HttpRequest::new();
match req.ip() {
    Ok(ip) => println!("IP: {}", ip),
    Err(e) => println!("Error: {}", e)
}
```

Returns `Result<&str, &str>`.

## Getting Request Headers

Returns the specified header value.

```rust
use ripress::context::HttpRequest;

let req = HttpRequest::new();
match req.get_header("content-type") {
    Ok(value) => println!("Header: {}", value),
    Err(e) => println!("Error: {:?}", e)
}
```

Returns `Result<&str, HttpRequestError>`.

## Accessing URL Parameters

```rust
use ripress::context::HttpRequest;

let req = HttpRequest::new();
match req.get_params("id") {
    Ok(value) => println!("ID: {}", value),
    Err(e) => println!("Error: {:?}", e)
}
```

Example:

- Route: `GET /user/:id`
- Request: `GET /user/123`
- `get_params("id")` returns `Ok("123")`

Returns `Result<&str, HttpRequestError>`.

## Accessing Query Parameters

```rust
use ripress::context::HttpRequest;

let req = HttpRequest::new();
match req.get_query("q") {
    Ok(value) => println!("Query: {}", value),
    Err(e) => println!("Error: {:?}", e)
}
```

Example:

- URL: `GET /search?q=Ripress`
- `get_query("q")` returns `Ok("Ripress")`

Returns `Result<&str, HttpRequestError>`.

## Getting Request Protocol

Returns the request's protocol.

```rust
use ripress::context::HttpRequest;

let req = HttpRequest::new();
let protocol: &str = req.get_protocol();
println!("Protocol: {}", protocol); // "http" or "https"
```

Returns `&str` containing the protocol.

## Checking If Request Is Secure

Returns whether the request was made over HTTPS.

```rust
use ripress::context::HttpRequest;

let req = HttpRequest::new();
let is_secure: bool = req.is_secure();
println!("Is Secure: {}", is_secure);
```

Returns `bool`.

## Get data from request that is inserted by middleware

```rust
use ripress::context::HttpRequest;

let req = HttpRequest::new();
req.set_data("id", "123");
let id = req.get_data("id");
println!("Id: {:?}", id);
```

Returns `Option<&String>` with the data value if found, or `None` if not found.

## Set data to request from middleware

```rust
use ripress::context::HttpRequest;

let mut req = HttpRequest::new();
req.set_data("id", "123");
let id = req.get_data("id");
println!("Id: {:?}", id);
```

## Reading Request Body

### JSON Body

```rust
use ripress::context::HttpRequest;
use serde::Deserialize;

#[derive(Deserialize)]
struct User {
    name: String,
    age: u8,
}

let req = HttpRequest::new();
match req.json::<User>() {
    Ok(user) => println!("Name: {}, Age: {}", user.name, user.age),
    Err(e) => println!("Error: {}", e)
}
```

Returns `Result<J, String>` where `J` is your deserialized type.

### Text Body

```rust
use ripress::context::HttpRequest;

let req = HttpRequest::new();
match req.text() {
    Ok(text) => println!("Text: {}", text),
    Err(e) => println!("Error: {}", e)
}
```

Returns `Result<String, String>`.

### Form Data

```rust
use ripress::context::HttpRequest;
use std::collections::HashMap;

let req = HttpRequest::new();
match req.form_data() {
    Ok(form) => {
        println!("Key: {:?}", form.get("key"));
        println!("Key2: {:?}", form.get("key2"));
    },
    Err(e) => println!("Error: {}", e)
}
```

Returns `Result<HashMap<String, String>, String>`.
