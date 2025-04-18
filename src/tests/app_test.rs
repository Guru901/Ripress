use crate::{context::HttpResponse, request::HttpRequest};

async fn _test_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    return res.ok();
}

#[cfg(test)]
mod tests {

    use crate::{
        app::{box_future, App},
        context::HttpResponse,
        tests::app_test::_test_handler,
    };

    use crate::context::HttpRequest;
    use crate::types::{HttpMethods, Next};
    use std::time::Duration;

    #[test]
    pub fn test_add_get_route() {
        let mut app = App::new();
        app.get("/user/:id", _test_handler);
        assert!(app
            .get_routes("/user/:id", crate::types::HttpMethods::GET)
            .is_some());
    }

    #[test]
    pub fn test_add_head_route() {
        let mut app = App::new();
        app.head("/user/:id", _test_handler);
        assert!(app
            .get_routes("/user/:id", crate::types::HttpMethods::HEAD)
            .is_some());
    }

    #[test]
    pub fn test_add_post_route() {
        let mut app = App::new();
        app.post("/user/:id", _test_handler);

        assert!(app
            .get_routes("/user/:id", crate::types::HttpMethods::POST)
            .is_some());
    }
    #[test]
    pub fn test_add_delete_route() {
        let mut app = App::new();
        app.delete("/user/:id", _test_handler);
        assert!(app
            .get_routes("/user/:id", crate::types::HttpMethods::DELETE)
            .is_some());
    }

    #[test]
    pub fn test_add_patch_route() {
        let mut app = App::new();
        app.patch("/user/:id", _test_handler);
        assert!(app
            .get_routes("/user/:id", crate::types::HttpMethods::PATCH)
            .is_some());
    }

    #[test]
    pub fn test_add_put_route() {
        let mut app = App::new();
        app.put("/user/:id", _test_handler);
        assert!(app
            .get_routes("/user/:id", crate::types::HttpMethods::PUT)
            .is_some());
    }

    #[test]
    pub fn test_add_all_route() {
        let mut app = App::new();
        app.all("/user/:id", _test_handler);

        assert!(app
            .get_routes("/user/:id", crate::types::HttpMethods::POST)
            .is_some());
        assert!(app
            .get_routes("/user/:id", crate::types::HttpMethods::GET)
            .is_some());

        assert!(app
            .get_routes("/user/:id", crate::types::HttpMethods::PUT)
            .is_some());

        assert!(app
            .get_routes("/user/:id", crate::types::HttpMethods::PATCH)
            .is_some());

        assert!(app
            .get_routes("/user/:id", crate::types::HttpMethods::DELETE)
            .is_some());

        assert!(app
            .get_routes("/user/:id", crate::types::HttpMethods::HEAD)
            .is_some());
    }

    #[tokio::test]
    async fn test_box_future() {
        async fn test_handler() -> HttpResponse {
            HttpResponse::new().ok().text("test")
        }

        let boxed = box_future(test_handler());

        let response = boxed.await;
        assert_eq!(response.status_code, 200);
    }

    #[tokio::test]
    async fn test_listen() {
        let mut app = App::new();
        app.get("/", _test_handler);
        app.post("/", _test_handler);
        app.patch("/", _test_handler);
        app.put("/", _test_handler);
        app.delete("/", _test_handler);
        app.head("/", _test_handler);

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

        app.use_middleware("", |req, _| {
            let req = req.clone();
            Box::pin(async move {
                println!("Middleware 1");
                (req, None)
            })
        });

        assert!(!app.get_middlewares().is_empty());
    }

    #[test]
    fn test_http_methods_display() {
        let get_method = HttpMethods::GET;
        let post_method = HttpMethods::POST;
        let patch_method = HttpMethods::PATCH;
        let delete_method = HttpMethods::DELETE;
        let put_method = HttpMethods::PUT;
        let head_method = HttpMethods::HEAD;

        println!(
            "{} {} {} {} {} {}",
            get_method, post_method, patch_method, delete_method, put_method, head_method
        );
    }

    #[test]
    fn test_next_new_fn() {
        let new_next = Next::new();
        assert_eq!(new_next.middleware.len(), 0);
    }

    #[tokio::test]
    async fn test_listen_function() {
        // Create an App instance and add a simple GET route that returns "Hello World"
        let mut app = App::new();
        app.get("/", |_: HttpRequest, res: HttpResponse| async move {
            res.ok().text("Hello World")
        });

        // Spawn the server on port 3001 in a background task.
        let server_handle = tokio::spawn(async move {
            app.listen(3001, || {
                println!("Server started on 3001");
            })
            .await;
        });

        // Allow the server some time to start.
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Send an HTTP GET request to the "/" route.
        let response = reqwest::get("http://127.0.0.1:3001/")
            .await
            .expect("Failed to send request");
        assert_eq!(response.status(), 200);
        let body = response.text().await.expect("Failed to read response text");
        assert_eq!(body, "Hello World");

        // Stop the server by aborting the task.
        server_handle.abort();
    }
}
