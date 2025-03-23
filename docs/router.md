# Router Object (Router)

## Overview

The `Router` struct provides a simple way to group and manage routes under a common base path. It allows you to define routes for various HTTP methods (GET, POST, PUT, DELETE, and PATCH) and then register those routes with an `App` instance.

## Creating a Router Instance

To create a new router, use the `Router::new` method and specify the base path. For example:

```rust
use ripress::router::Router;

let mut router = Router::new("/api");
```

## Defining Routes

The Router offers methods corresponding to each HTTP method. Each method takes a static path and a handler function. The handler must be an async function that accepts an `HttpRequest` and an `HttpResponse`, then returns an `HttpResponse`.

### GET Route

```rust
use ripress::{router::Router, context::{HttpRequest, HttpResponse}};

async fn get_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("GET request handled")
}

let mut router = Router::new("/api");
router.get("/hello", get_handler);
```

### POST Route

```rust
use ripress::{router::Router, context::{HttpRequest, HttpResponse}};

async fn post_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("POST request handled")
}

let mut router = Router::new("/api");
router.post("/submit", post_handler);
```

### PUT Route

```rust
use ripress::{router::Router, context::{HttpRequest, HttpResponse}};

async fn put_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("PUT request handled")
}

let mut router = Router::new("/api");
router.put("/update", put_handler);
```

### DELETE Route

```rust
use ripress::{router::Router, context::{HttpRequest, HttpResponse}};

async fn delete_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("DELETE request handled")
}

let mut router = Router::new("/api");
router.delete("/remove", delete_handler);
```

### PATCH Route

```rust
use ripress::{router::Router, context::{HttpRequest, HttpResponse}};

async fn patch_handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("PATCH request handled")
}

let mut router = Router::new("/api");
router.patch("/modify", patch_handler);
```

## Registering the Router with an App

After defining the routes using your router, register the router with an `App` instance. This will add all the router’s routes to the application with their full path (combining the router’s base path and the route’s defined path):

```rust
use ripress::{app::App, router::Router, context::{HttpRequest, HttpResponse}};

async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res.ok().text("Hello from Router!")
}

let mut router = Router::new("/api");
// Define routes on the router
router.get("/hello", handler);

// Create an App instance and register the router
let mut app = App::new();
router.register(&mut app);
```

## How It Works

- **Route Storage:**  
  The router stores routes internally using a `HashMap`, with each route path mapping to a set of HTTP methods and their associated handlers.

- **Method Handlers:**  
  The helper methods (`get`, `post`, `put`, `delete`, and `patch`) wrap the provided handler functions and associate them with the corresponding HTTP method.

- **Registration:**  
  Calling `register` on the router iterates through all the defined routes, prepends the router's base path to each route, and then adds them to the provided `App` instance.

This modular design simplifies the management of routes when building larger applications by keeping related routes together under a shared base path.
