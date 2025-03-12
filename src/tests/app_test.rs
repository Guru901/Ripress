use crate::{context::HttpResponse, request::HttpRequest};

async fn _test_handler(_req: HttpRequest, res: HttpResponse) -> HttpResponse {
    return res.ok();
}

#[cfg(test)]
mod tests {
    use crate::{app::App, tests::app_test::_test_handler};

    #[test]
    pub fn test_add_route_1() {
        let mut app = App::new();
        app.get("/user/{id}", _test_handler);
        assert!(app.get_routes().get("/user/{id}").is_some());
    }

    #[test]
    pub fn test_add_route_2() {
        let mut app = App::new();
        app.post("/user/{id}", _test_handler);
        assert!(app.get_routes().get("/user/{id}").is_some());
    }

    #[test]
    pub fn test_add_route_3() {
        let mut app = App::new();
        app.put("/user/{id}", _test_handler);
        assert!(app.get_routes().get("/user/{id}").is_some());
    }
    #[test]
    pub fn test_add_route_4() {
        let mut app = App::new();
        app.delete("/user/{id}", _test_handler);
        assert!(app.get_routes().get("/user/{id}").is_some());
    }
}
