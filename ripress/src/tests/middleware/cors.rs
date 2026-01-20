#[cfg(test)]
mod test {
    use crate::{
        middlewares::cors::{CorsConfig, cors},
        req::HttpRequest,
        res::HttpResponse,
        types::HttpMethods,
    };

    fn run_cors_middleware(
        method: HttpMethods,
        config: Option<CorsConfig>,
    ) -> (HttpRequest, Option<HttpResponse>) {
        let mut req = HttpRequest::new();
        req.method = method;
        let res = HttpResponse::new();
        let mw = cors(config);
        futures::executor::block_on(mw(req, res))
    }

    #[test]
    fn test_cors_headers_default_config() {
        let (_, maybe_res) = run_cors_middleware(HttpMethods::OPTIONS, None);
        assert!(maybe_res.is_some());
        let res = maybe_res.unwrap();

        assert_eq!(res.headers.get("Access-Control-Allow-Origin"), Some("*"));
        assert_eq!(
            res.headers.get("Access-Control-Allow-Methods"),
            Some("GET, POST, PUT, DELETE, OPTIONS, HEAD")
        );
        assert_eq!(
            res.headers.get("Access-Control-Allow-Headers"),
            Some("Content-Type, Authorization")
        );
        assert_eq!(res.headers.get("Access-Control-Allow-Credentials"), None);
    }

    #[test]
    fn test_cors_headers_custom_config_with_credentials() {
        let config = CorsConfig {
            allowed_origin: "https://example.com",
            allowed_methods: "GET, POST",
            allowed_headers: "X-Custom-Header",
            allow_credentials: true,
        };
        let (_, maybe_res) = run_cors_middleware(HttpMethods::OPTIONS, Some(config.clone()));
        assert!(maybe_res.is_some());
        let res = maybe_res.unwrap();

        assert_eq!(
            res.headers.get("Access-Control-Allow-Origin"),
            Some("https://example.com")
        );
        assert_eq!(
            res.headers.get("Access-Control-Allow-Methods"),
            Some("GET, POST")
        );
        assert_eq!(
            res.headers.get("Access-Control-Allow-Headers"),
            Some("X-Custom-Header")
        );
        assert_eq!(
            res.headers.get("Access-Control-Allow-Credentials"),
            Some("true")
        );
    }

    #[test]
    fn test_cors_options_preflight_returns_response() {
        let (_, maybe_res) = run_cors_middleware(HttpMethods::OPTIONS, None);
        assert!(maybe_res.is_some());
        let res = maybe_res.unwrap();
        assert_eq!(res.status_code.as_u16(), 200);
        assert_eq!(res.headers.get("Access-Control-Allow-Origin"), Some("*"));
        assert_eq!(
            res.headers.get("Access-Control-Allow-Methods"),
            Some("GET, POST, PUT, DELETE, OPTIONS, HEAD")
        );
        assert_eq!(
            res.headers.get("Access-Control-Allow-Headers"),
            Some("Content-Type, Authorization")
        );
    }

    #[test]
    fn test_cors_options_preflight_with_credentials() {
        let config = CorsConfig {
            allowed_origin: "https://foo.com",
            allowed_methods: "OPTIONS",
            allowed_headers: "X-Token",
            allow_credentials: true,
        };
        let (_, maybe_res) = run_cors_middleware(HttpMethods::OPTIONS, Some(config.clone()));
        assert!(maybe_res.is_some());
        let res = maybe_res.unwrap();
        assert_eq!(res.status_code.as_u16(), 200);
        assert_eq!(
            res.headers.get("Access-Control-Allow-Origin"),
            Some("https://foo.com")
        );
        assert_eq!(
            res.headers.get("Access-Control-Allow-Methods"),
            Some("OPTIONS")
        );
        assert_eq!(
            res.headers.get("Access-Control-Allow-Headers"),
            Some("X-Token")
        );
        assert_eq!(
            res.headers.get("Access-Control-Allow-Credentials"),
            Some("true")
        );
    }
}
