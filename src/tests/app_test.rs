use crate::{context::HttpResponse, request::HttpRequest};

async fn _test_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    return res.ok();
}

#[cfg(test)]
mod tests {
    

    use crate::types::{Middleware, Next};
    use crate::{
        app::{box_future, App},
        context::HttpResponse,
        tests::app_test::_test_handler,
    };
    use std::future::Future;
    use std::pin::Pin;
    use std::time::Duration;

    #[test]
    pub fn test_add_get_route() {
        let mut app = App::new();
        app.get("/user/{id}", _test_handler);
        assert!(app
            .get_routes("/user/{id}", crate::types::HttpMethods::GET)
            .is_some());
    }

    #[test]
    pub fn test_add_post_route() {
        let mut app = App::new();
        app.post("/user/{id}", _test_handler);

        assert!(app
            .get_routes("/user/{id}", crate::types::HttpMethods::POST)
            .is_some());
    }
    #[test]
    pub fn test_add_delete_route() {
        let mut app = App::new();
        app.delete("/user/{id}", _test_handler);
        assert!(app
            .get_routes("/user/{id}", crate::types::HttpMethods::DELETE)
            .is_some());
    }

    #[test]
    pub fn test_add_patch_route() {
        let mut app = App::new();
        app.patch("/user/{id}", _test_handler);
        assert!(app
            .get_routes("/user/{id}", crate::types::HttpMethods::PATCH)
            .is_some());
    }

    #[test]
    pub fn test_add_put_route() {
        let mut app = App::new();
        app.put("/user/{id}", _test_handler);
        assert!(app
            .get_routes("/user/{id}", crate::types::HttpMethods::PUT)
            .is_some());
    }

    #[test]
    pub fn test_add_all_route() {
        let mut app = App::new();
        app.all("/user/{id}", _test_handler);

        assert!(app
            .get_routes("/user/{id}", crate::types::HttpMethods::POST)
            .is_some());
        assert!(app
            .get_routes("/user/{id}", crate::types::HttpMethods::GET)
            .is_some());

        assert!(app
            .get_routes("/user/{id}", crate::types::HttpMethods::PUT)
            .is_some());

        assert!(app
            .get_routes("/user/{id}", crate::types::HttpMethods::PATCH)
            .is_some());

        assert!(app
            .get_routes("/user/{id}", crate::types::HttpMethods::DELETE)
            .is_some());
    }

    #[tokio::test]
    async fn test_box_future() {
        async fn test_handler() -> HttpResponse {
            HttpResponse::new().ok().text("test")
        }

        let boxed = box_future(test_handler());

        let response = boxed.await;
        assert_eq!(response.get_status_code(), 200);
    }

    #[tokio::test]
    async fn test_listen() {
        let mut app = App::new();
        app.get("/", _test_handler);
        app.post("/", _test_handler);
        app.patch("/", _test_handler);
        app.put("/", _test_handler);
        app.delete("/", _test_handler);

        app.all("/all", _test_handler);

        let handle = tokio::spawn(async move {
            app.listen(3000, || {}).await;
        });

        tokio::time::sleep(Duration::from_secs(5)).await;
        handle.abort();
    }

    #[test]
    fn test_use_middleware() {
        let mut app = App::new();

        struct LoggingMiddleware;

        impl Middleware for LoggingMiddleware {
            fn handle<'a>(
                &'a self,
                req: crate::request::HttpRequest,
                res: HttpResponse,
                next: Next<'a>,
            ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'a>> {
                Box::pin(async move {
                    println!(
                        "Request received: {:?} {:?}",
                        req.get_method(),
                        req.get_path()
                    );

                    let response = next.run(req, res).await;

                    println!("Response status: {}", response.get_status_code());

                    response
                })
            }

            fn clone_box(&self) -> Box<dyn Middleware> {
                Box::new(LoggingMiddleware)
            }
        }

        app.use_middleware(LoggingMiddleware);

        assert!(!app.middlewares.is_empty());
    }
}
