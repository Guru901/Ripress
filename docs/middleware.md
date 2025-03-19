# Middleware in Ripress

Middleware provides a powerful way to process HTTP requests and responses in a modular, reusable manner. This document explains how middleware works in Ripress and how to implement your own middleware components.

## Overview

In Ripress, middleware follows a chain-of-responsibility pattern where each middleware component can:

1. Process an incoming request
2. Pass control to the next middleware in the chain
3. Process the outgoing response after the next middleware returns

This enables cross-cutting concerns like logging, authentication, error handling, and more to be separated from your route handlers.

## The Middleware Trait

The core of Ripress's middleware system is the `Middleware` trait defined in `types.rs`:

```rust
pub trait Middleware: Send + Sync + 'static {
    fn handle<'a>(
        &'a self,
        req: HttpRequest,
        res: HttpResponse,
        next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'a>>;

    fn clone_box(&self) -> Box<dyn Middleware>;
}
```

This trait requires implementing two methods:

- `handle`: Processes the request/response and calls the next middleware
- `clone_box`: Enables cloning of boxed middleware instances

## The Next Struct

The `Next` struct represents the rest of the middleware chain:

```rust
pub struct Next<'a> {
    pub middleware: &'a [Box<dyn Middleware>],
    pub handler: Handler,
}
```

It contains:

- A slice of remaining middleware components
- The final route handler to be called when all middleware has been processed

The `run` method on `Next` executes the middleware chain:

```rust
impl<'a> Next<'a> {
    pub async fn run(self, req: HttpRequest, res: HttpResponse) -> HttpResponse {
        if let Some((current, rest)) = self.middleware.split_first() {
            // Call the next middleware
            let next = Next {
                middleware: rest,
                handler: self.handler.clone(),
            };
            current.handle(req, res, next).await
        } else {
            // No more middleware, call the handler
            (self.handler)(req, res).await
        }
    }
}
```

## Using Middleware

To add middleware to your application, use the `use_middleware` method on the `App` struct:

```rust
pub fn use_middleware<M: Middleware>(&mut self, middleware: M) -> &mut Self {
    self.middlewares.push(Box::new(middleware));
    self
}
```

This method adds the middleware to the application's middleware stack. Middleware is executed in the order it's added.

## Implementing Custom Middleware

Here's an example of how to implement a simple logging middleware:

```rust
use ripress::{
    context::{HttpRequest, HttpResponse},
    types::{Middleware, Next},
};
use std::pin::Pin;

pub struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn handle<'a>(
        &'a self,
        req: HttpRequest,
        res: HttpResponse,
        next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'a>> {
        Box::pin(async move {
            println!("Request received: {} {}", req.method(), req.path());

            // Call the next middleware in the chain
            let response = next.run(req, res).await;

            println!("Response status: {}", response.status_code());

            response
        })
    }

    fn clone_box(&self) -> Box<dyn Middleware> {
        Box::new(LoggingMiddleware)
    }
}
```

## Registering Middleware

To use your middleware in an application:

```rust
use ripress::app::App;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Add middleware
    app.use_middleware(LoggingMiddleware);

    // Add routes
    app.get("/", |_req, res| async move {
        res.ok().text("Hello, World!")
    });

    app.listen("127.0.0.1:3000").await;
}
```

## Middleware Execution Flow

1. When a request is received, the `App` creates a `Next` instance with all registered middleware and the matched route handler
2. The first middleware's `handle` method is called with the request, a new response, and the `Next` instance
3. The middleware processes the request and calls `next.run()`
4. This process repeats until all middleware has been executed
5. The final route handler is called and returns a response
6. The response flows back through each middleware in reverse order
7. Each middleware can modify the response before it's returned to the client

## Best Practices

1. **Keep middleware focused**: Each middleware should handle a single concern
2. **Order matters**: Add middleware in the order you want them to execute
3. **Be careful with mutable state**: Middleware should be thread-safe
4. **Error handling**: Middleware can catch and handle errors from subsequent middleware or handlers
5. **Performance**: Keep middleware lightweight to avoid adding unnecessary overhead

## Common Middleware Use Cases

- **Authentication**: Verify user credentials and set user information
- **Authorization**: Check if authenticated users have permission to access resources
- **Logging**: Record request and response information
- **Error handling**: Catch errors and format appropriate responses
- **CORS**: Handle Cross-Origin Resource Sharing headers
- **Rate limiting**: Prevent abuse by limiting request frequency
- **Request parsing**: Parse and validate incoming request data
- **Response compression**: Compress response bodies
- **Caching**: Cache responses to improve performance
