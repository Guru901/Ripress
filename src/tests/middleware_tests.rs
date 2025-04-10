#[cfg(test)]
mod tests {

    use crate::{
        context::{HttpRequest, HttpResponse},
        middlewares::{
            cors::{cors, CorsConfig},
            logger::{logger, LoggerConfig},
        },
    };

    #[tokio::test]
    async fn test_cors_default_config() {
        // Use default config by passing None.
        let cors_mw = cors(None);
        let mut req = HttpRequest::new();
        let res = HttpResponse::new();

        let response = cors_mw(&mut req, res).await.1.unwrap();

        // Check the headers set by the middleware.
        assert_eq!(
            response.get_header("Access-Control-Allow-Origin").unwrap(),
            "*"
        );
        assert_eq!(
            response.get_header("Access-Control-Allow-Methods").unwrap(),
            "GET, POST, PUT, DELETE, OPTIONS"
        );
        assert_eq!(
            response.get_header("Access-Control-Allow-Headers").unwrap(),
            "Content-Type, Authorization"
        );
        // Default config does not allow credentials.
        assert!(response
            .get_header("Access-Control-Allow-Credentials")
            .is_err());
    }

    #[tokio::test]
    async fn test_cors_custom_config() {
        let config = CorsConfig {
            allowed_origin: "https://example.com".to_string(),
            allowed_methods: "GET, POST".to_string(),
            allow_credentials: true,
        };
        let cors_mw = cors(Some(config.clone()));
        let mut req = HttpRequest::new();
        let res = HttpResponse::new();

        let response = cors_mw(&mut req, res).await.1.unwrap();

        assert_eq!(
            response.get_header("Access-Control-Allow-Origin").unwrap(),
            config.allowed_origin
        );
        assert_eq!(
            response.get_header("Access-Control-Allow-Methods").unwrap(),
            config.allowed_methods
        );
        assert_eq!(
            response.get_header("Access-Control-Allow-Headers").unwrap(),
            "Content-Type, Authorization"
        );
        // For custom config allow_credentials is set to true.
        assert_eq!(
            response
                .get_header("Access-Control-Allow-Credentials")
                .unwrap(),
            "true"
        );
    }

    #[tokio::test]
    async fn test_logger_default_config() {
        // Use default config by passing None.
        let logger_mw = logger(None);
        let mut req = HttpRequest::new();
        let res = HttpResponse::new();

        let _ = logger_mw(&mut req, res).await;
    }

    #[tokio::test]
    async fn test_logger_custom_config() {
        let config = LoggerConfig {
            duration: true,
            method: true,
            path: true,
        };
        let logger_mw = logger(Some(config.clone()));
        let mut req = HttpRequest::new();
        let res = HttpResponse::new();

        let _ = logger_mw(&mut req, res).await;
    }
}
