# Response Object (HttpResponse)

## Overview

`HttpResponse` represents an outgoing HTTP response and provides utilities for sending data back to the client.

## Creating a Response Object

HttpResponse is automatically passed to route handlers.
The return type of a route handler is HttpResponse.

Example:

```rust
async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.text().unwrap_or("No body".to_string());
    res.ok().text(format!("Received: {}", body))
}
```

## Sending Status code

Sends a status code to the client.

- Example

```rust
let res = ripress::context::HttpResponse::new();
res.status(code)
```

Sends the status code specified by the `code` parameter.

## Status code helpers

- Example

```rust
let res = ripress::context::HttpResponse::new();
res.ok(); // 200 OK - Request succeeded
res.bad_request(); // 400 Bad Request - Client sent invalid data
res.not_found(); // 404 Not Found - Resource does not exist
res.internal_server_error(); // 500 Internal Server Error - Something went wrong on the server
```

## Send data to the client

### JSON

Sends a JSON response to the client.

```rust
use serde_json::json;

let json_body = json!({"key": "value"});
let res = ripress::context::HttpResponse::new().json(json_body);
```

### Text

Sends a text response to the client.

```rust
let res = ripress::context::HttpResponse::new().text("Hello, World!");
```
