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
    };

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
