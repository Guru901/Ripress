use crate::context::{HttpRequest, HttpResponse};

async fn _test_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    return res.ok();
}

#[cfg(test)]
mod tests {
    use super::_test_handler;
    use crate::{app::App, router::Router, types::RouterFns};

    #[test]
    fn test_add_options_route() {
        let mut router = Router::new("/");
        router.options("/user/{id}", _test_handler);
        assert!(
            router
                .get_routes("/user/{id}", crate::types::HttpMethods::OPTIONS)
                .is_some()
        );
    }
    #[test]
    fn test_add_put_route() {
        let mut router = Router::new("/");
        router.put("/user/{id}", _test_handler);
        assert!(
            router
                .get_routes("/user/{id}", crate::types::HttpMethods::PUT)
                .is_some()
        );
    }
    #[test]
    fn test_add_patch_route() {
        let mut router = Router::new("/");
        router.patch("/user/{id}", _test_handler);
        assert!(
            router
                .get_routes("/user/{id}", crate::types::HttpMethods::PATCH)
                .is_some()
        );
    }
    #[test]
    fn test_add_delete_route() {
        let mut router = Router::new("/");
        router.delete("/user/{id}", _test_handler);
        assert!(
            router
                .get_routes("/user/{id}", crate::types::HttpMethods::DELETE)
                .is_some()
        );
    }
    #[test]
    fn test_add_post_route() {
        let mut router = Router::new("/");
        router.post("/user/{id}", _test_handler);
        assert!(
            router
                .get_routes("/user/{id}", crate::types::HttpMethods::POST)
                .is_some()
        );
    }
    #[test]
    fn test_add_head_route() {
        let mut router = Router::new("/");
        router.head("/user/{id}", _test_handler);
        assert!(
            router
                .get_routes("/user/{id}", crate::types::HttpMethods::HEAD)
                .is_some()
        );
    }

    #[test]

    fn test_register() {
        let mut app = App::new();
        let mut router = Router::new("/");
        router.get("/user/{id}", _test_handler);
        #[allow(deprecated)]
        router.register(&mut app);

        assert!(
            app.get_routes("//user/{id}", crate::types::HttpMethods::GET)
                .is_some()
        );
    }
}
