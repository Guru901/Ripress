use crate::context::{HttpRequest, HttpResponse};

async fn _test_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    return res.ok();
}

#[cfg(test)]
mod tests {

    use crate::{
        app::{App, box_future},
        context::HttpResponse,
        res::response_status::StatusCode,
        tests::app_test::_test_handler,
        types::RouterFns,
    };

    #[test]
    fn test_add_get_route() {
        let mut app = App::new();
        app.get("/user/{id}", _test_handler);
        assert!(
            app.get_routes("/user/{id}", crate::types::HttpMethods::GET)
                .is_some()
        );
    }

    #[test]
    fn test_add_head_route() {
        let mut app = App::new();
        app.head("/user/{id}", _test_handler);
        assert!(
            app.get_routes("/user/{id}", crate::types::HttpMethods::HEAD)
                .is_some()
        );
    }

    #[test]
    fn test_add_post_route() {
        let mut app = App::new();
        app.post("/user/{id}", _test_handler);

        assert!(
            app.get_routes("/user/{id}", crate::types::HttpMethods::POST)
                .is_some()
        );
    }
    #[test]
    fn test_add_delete_route() {
        let mut app = App::new();
        app.delete("/user/{id}", _test_handler);
        assert!(
            app.get_routes("/user/{id}", crate::types::HttpMethods::DELETE)
                .is_some()
        );
    }

    #[test]
    fn test_add_patch_route() {
        let mut app = App::new();
        app.patch("/user/{id}", _test_handler);
        assert!(
            app.get_routes("/user/{id}", crate::types::HttpMethods::PATCH)
                .is_some()
        );
    }

    #[test]
    fn test_add_put_route() {
        let mut app = App::new();
        app.put("/user/{id}", _test_handler);
        assert!(
            app.get_routes("/user/{id}", crate::types::HttpMethods::PUT)
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_box_future() {
        async fn test_handler() -> HttpResponse {
            HttpResponse::new().ok().text("test")
        }

        let boxed = box_future(test_handler());

        let response = boxed.await;
        assert_eq!(response.status_code, StatusCode::Ok);
    }

    #[test]
    fn test_static_files_configuration() {
        let mut app = App::new();

        app.static_files("/public", "./public");

        let mount_path = app
            .static_files
            .get("mount_path")
            .expect("mount_path should be set");
        let serve_from = app
            .static_files
            .get("serve_from")
            .expect("serve_from should be set");

        assert_eq!(mount_path, &"/public");
        assert_eq!(serve_from, &"./public");
    }
}
