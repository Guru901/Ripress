# Request Object (HttpRequest)

## Overview

`HttpRequest` represents an incoming HTTP request and provides utilities for accessing query parameters, request headers, body content, and more.

## Creating a Request Object

HttpRequest is automatically passed to route handlers.

Example:

```rust
async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    let body = req.text().unwrap_or("No body".to_string());
    res.ok().text(format!("Received: {}", body))
}
```

## Checking Content-Type

Checks if the `Content-Type` of the request matches the specified type.

- Example

```rust
use ripress::types::RequestBodyType;
let req = ripress::context::HttpRequest::new();

if req.is(RequestBodyType::JSON) {
    println!("Request is JSON");
}
```

Returns `true` if the `Content-Type` matches, otherwise `false`.

## Getting Request Method

Returns the request's method (GET, POST, etc.)

- Example

```rust
let req = ripress::context::HttpRequest::new();
req.get_method(); // returns (GET, POST, etc.)
```

## Getting Request Origin URL

Returns the request's origin URL.

- Example

```rust
let req = ripress::context::HttpRequest::new();
req.get_origin_url();
```

### Example Cases:

- For request: GET /user/123
- origin_url → /user/123

- For request: GET /user/123?q=hello
- origin_url → /user/123?q=hello

- Returns: Option<String>

Some(url) if available, or None if it cannot be determined.

## Getting Request Path

Returns the request's path.

- Example

```rust
let req = ripress::context::HttpRequest::new();
req.get_path();
```

### Example Cases:

- For request: GET /user/123?q=hello
- path → /user/123

- Returns: Option<String>

- Some(path) if available, or None if it cannot be determined.

## Getting Client IP Address

Returns the client's IP address.

- Example

```rust
let ip = req.ip();
println!("Client IP: {:?}", ip);
```

This function retrieves the IP address of the client making the request.

Returns an `Option<String>`, where `Some(ip)` contains the IP if available, or `None` if it cannot be determined.

## Accessing URL Parameters

```rust
let value = req.get_params("key");
```

- Example Route: GET /user/{id}
- For request GET /user/123, get_params("id") returns "123".

- Usage:

```rust
let user_id = req.get_params("id").unwrap_or("unknown".to_string());
```

## Accessing Query Parameters

```rust
let value = req.get_query("key");
```

- Returns: `Some(value)` if the query parameter exists, otherwise None.
- Example URL: `GET /search?q=Ripress`
- Usage:

```rust
let search_query = req.get_query("q").unwrap_or("default".to_string());
```

## Reading Request Body

### JSON Body

```rust
let data = req.json::<MyDataType>();
```

- Returns: `Ok(value)` if the URL parameter exists, otherwise `Err(error)`.
- Example: `POST /submit/json` with `{"name": "John Doe", "age": 30}`
- Usage:

```rust
#[derive(serde::Deserialize)]
struct User {
    name: String,
    age: u8,
}

let user = req.json::<User>().unwrap();
println!("Name: {}", user.name); // Prints "John Doe"
println!("Age : {}", user.age); // Prints "30"
```

### Text Body

```rust
let data = req.text();
```

- Returns: `Ok(value)` if the URL parameter exists, otherwise `Err(error)`.
- Example: `POST /submit/text` with `"Hello, world!"`
- Usage:

```rust
let text = req.text().unwrap_or("No text".to_string());
println!("Text: {}", text); // Prints "Hello, world!"
```

### Form Data

This function parses the request body as form-encoded data (application/x-www-form-urlencoded) and returns a HashMap of key-value pairs.

```rust
let data = req.form_data();
```

- Returns: `Ok(value)` if the URL parameter exists, otherwise `Err(error)`.
- Example: `POST /submit/form` with `key=value&key2=value2`
- Usage:

```rust
let form_data = req.form_data().unwrap_or(HashMap::new());
println!("Key: {}", form_data.get("key").unwrap_or("No key")); // Prints "value"
println!("Key2: {}", form_data.get("key2").unwrap_or("No key2")); // Prints "value2"
```
