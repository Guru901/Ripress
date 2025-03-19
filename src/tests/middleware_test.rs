use crate::{
    context::HttpResponse,
    request::HttpRequest,
    types::{Middleware, Next},
};
use std::{future::Future, pin::Pin};

// A simple test middleware that adds a header to the response
struct TestMiddleware {
    name: String,
}

impl TestMiddleware {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl Middleware for TestMiddleware {
    fn handle<'a>(
        &'a self,
        req: HttpRequest,
        res: HttpResponse,
        next: Next<'a>,
    ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'a>> {
        Box::pin(async move {
            // Add a header to mark that this middleware was called
            let res = res.set_header(format!("X-Middleware-{}", self.name).as_str(), "called");

            // Call the next middleware in the chain
            let res = next.run(req, res).await;

            // Return the response
            res
        })
    }

    fn clone_box(&self) -> Box<dyn Middleware> {
        Box::new(TestMiddleware {
            name: self.name.clone(),
        })
    }
}

// A simple handler that returns the response as is
async fn test_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    res
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::types::Handler;

    use super::*;

    #[tokio::test]
    async fn test_middleware_clone() {
        // Create a middleware
        let middleware = TestMiddleware::new("Test".to_string());
        let boxed_middleware: Box<dyn Middleware> = Box::new(middleware);

        // Clone the boxed middleware
        let cloned_middleware = boxed_middleware.clone();

        // Create request and response
        let req = HttpRequest::new();
        let res = HttpResponse::new();

        // Create a handler
        let handler: Handler = Arc::new(|req, res| Box::pin(test_handler(req, res)));

        // Create Next with both middlewares
        let middlewares: Vec<Box<dyn Middleware>> = vec![boxed_middleware, cloned_middleware];

        // Test the first middleware
        let next = Next {
            middleware: &middlewares[1..],
            handler: handler.clone(),
        };

        // Run the middleware chain
        let result = middlewares[0].handle(req, res, next).await;

        // Verify the middleware was called
        assert!(result.get_header("X-Middleware-Test").is_ok());
        assert_eq!(result.get_header("X-Middleware-Test").unwrap(), "called");
    }

    #[tokio::test]
    async fn test_next_run_with_middleware() {
        // Create middlewares
        let middleware1 = TestMiddleware::new("First".to_string());
        let middleware2 = TestMiddleware::new("Second".to_string());

        let middlewares: Vec<Box<dyn Middleware>> =
            vec![Box::new(middleware1), Box::new(middleware2)];

        // Create request and response
        let req = HttpRequest::new();
        let res = HttpResponse::new();

        // Create a handler
        let handler: Handler = Arc::new(|req, res| Box::pin(test_handler(req, res)));

        // Create Next with all middlewares
        let next = Next {
            middleware: &middlewares,
            handler: handler.clone(),
        };

        // Run the middleware chain
        let result = next.run(req, res).await;

        // Verify both middlewares were called
        assert!(result.get_header("X-Middleware-First").is_ok());
        assert_eq!(result.get_header("X-Middleware-First").unwrap(), "called");

        assert!(result.get_header("X-Middleware-Second").is_ok());
        assert_eq!(result.get_header("X-Middleware-Second").unwrap(), "called");
    }

    #[tokio::test]
    async fn test_next_run_without_middleware() {
        // Create an empty middleware list
        let middlewares: Vec<Box<dyn Middleware>> = vec![];

        // Create request and response
        let mut req = HttpRequest::new();
        let res = HttpResponse::new();

        // Add test data
        req.set_header("X-Test", "test-value");
        let res = res.set_header("X-Response", "original");

        // Create a handler that modifies the response
        let handler: Handler = Arc::new(|_req, res| {
            Box::pin(async move {
                let res = res.set_header("X-Handler", "called");
                res
            })
        });

        // Create Next with no middlewares
        let next = Next {
            middleware: &middlewares,
            handler,
        };

        // Run the handler directly (no middlewares)
        let result = next.run(req, res).await;

        // Verify the handler was called
        assert!(result.get_header("X-Response").is_ok());
        assert_eq!(result.get_header("X-Response").unwrap(), "original");

        assert!(result.get_header("X-Handler").is_ok());
        assert_eq!(result.get_header("X-Handler").unwrap(), "called");
    }

    #[tokio::test]
    async fn test_middleware_order() {
        // Create a middleware that adds an order number
        struct OrderMiddleware {
            order: usize,
        }

        impl Middleware for OrderMiddleware {
            fn handle<'a>(
                &'a self,
                req: HttpRequest,
                res: HttpResponse,
                next: Next<'a>,
            ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'a>> {
                Box::pin(async move {
                    // Add order before calling next
                    let res = res.set_header(format!("X-Before-{}", self.order).as_str(), "called");

                    // Call the next middleware
                    let res = next.run(req, res).await;

                    // Add order after calling next
                    let res = res.set_header(format!("X-After-{}", self.order).as_str(), "called");

                    res
                })
            }

            fn clone_box(&self) -> Box<dyn Middleware> {
                Box::new(OrderMiddleware { order: self.order })
            }
        }

        // Create middlewares with different orders
        let middleware1 = OrderMiddleware { order: 1 };
        let middleware2 = OrderMiddleware { order: 2 };
        let middleware3 = OrderMiddleware { order: 3 };

        let middlewares: Vec<Box<dyn Middleware>> = vec![
            Box::new(middleware1),
            Box::new(middleware2),
            Box::new(middleware3),
        ];

        // Create request and response
        let req = HttpRequest::new();
        let res = HttpResponse::new();

        // Create a simple handler
        let handler: Handler = Arc::new(|_req, res| {
            Box::pin(async move {
                let res = res.set_header("X-Handler", "called");
                res
            })
        });

        // Create Next with all middlewares
        let next = Next {
            middleware: &middlewares,
            handler,
        };

        // Run the middleware chain
        let result = next.run(req, res).await;

        // Verify the execution order
        // Before headers should be in order 1, 2, 3
        assert!(result.get_header("X-Before-1").is_ok());
        assert!(result.get_header("X-Before-2").is_ok());
        assert!(result.get_header("X-Before-3").is_ok());

        // Handler should be called
        assert!(result.get_header("X-Handler").is_ok());

        // After headers should be in reverse order 3, 2, 1
        assert!(result.get_header("X-After-3").is_ok());
        assert!(result.get_header("X-After-2").is_ok());
        assert!(result.get_header("X-After-1").is_ok());
    }
}
