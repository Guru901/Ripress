#[cfg(test)]
mod test {
    use crate::{
        middlewares::logger::{LoggerConfig, logger},
        req::HttpRequest,
        res::HttpResponse,
        types::HttpMethods,
    };

    #[tokio::test]
    async fn test_logger_default_config() {
        let logger_mw = logger(None);
        let mut req = HttpRequest::new();
        req.path = "/test".to_string();
        req.method = HttpMethods::POST;
        let res = HttpResponse::new();

        // Test that the middleware runs without panicking
        // and returns the expected values
        let (returned_req, maybe_res) = logger_mw(req.clone(), res.clone()).await;

        assert_eq!(returned_req.path, "/test");
        assert_eq!(returned_req.method, HttpMethods::POST);
        assert!(maybe_res.is_none());
    }

    #[tokio::test]
    async fn test_logger_custom_config() {
        let logger_mw = logger(Some(LoggerConfig {
            method: true,
            path: false,
            ..Default::default()
        }));

        let mut req = HttpRequest::new();
        req.path = "/foo".to_string();
        req.method = HttpMethods::PUT;
        let res = HttpResponse::new();

        let (returned_req, maybe_res) = logger_mw(req.clone(), res.clone()).await;

        assert_eq!(returned_req.path, "/foo");
        assert_eq!(returned_req.method, HttpMethods::PUT);
        assert!(maybe_res.is_none());
    }

    #[tokio::test]
    async fn test_logger_preserves_request_data() {
        let logger_mw = logger(None);
        let mut req = HttpRequest::new();
        req.path = "/api/users".to_string();
        req.method = HttpMethods::GET;
        let res = HttpResponse::new();

        let (returned_req, _) = logger_mw(req.clone(), res.clone()).await;

        // Verify the middleware preserves all request data
        assert_eq!(returned_req.path, req.path);
        assert_eq!(returned_req.method, req.method);
    }

    #[tokio::test]
    async fn test_logger_with_all_disabled() {
        let logger_mw = logger(Some(LoggerConfig {
            method: false,
            path: false,
            ..Default::default()
        }));

        let mut req = HttpRequest::new();
        req.path = "/disabled".to_string();
        req.method = HttpMethods::DELETE;
        let res = HttpResponse::new();

        let (returned_req, maybe_res) = logger_mw(req.clone(), res.clone()).await;

        assert_eq!(returned_req.path, "/disabled");
        assert_eq!(returned_req.method, HttpMethods::DELETE);
        assert!(maybe_res.is_none());
    }
}
