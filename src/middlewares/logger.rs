use crate::{context::HttpResponse, req::HttpRequest, types::FutMiddleware};

/// Configuration for the Logger Middleware
///
/// ## Fields
///
/// * `method` -  Whether to log the method
/// * `path` - Whether to log the path
/// * `duration` - Whether to log the duration

#[derive(Clone)]
pub struct LoggerConfig {
    pub method: bool,
    pub path: bool,
    pub duration: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        LoggerConfig {
            duration: true,
            method: true,
            path: true,
        }
    }
}

/// Builtin Logger Middleware
///
/// ## Arguments
///
/// * `config` - Configuration for the middleware
///
/// ## Examples
///
/// ```
/// use ripress::{app::App, middlewares::logger::logger};
/// let mut app = App::new();
/// app.use_middleware("", logger(None));
///
///```
///```
/// use ripress::{app::App, middlewares::logger::{logger, LoggerConfig}};
/// let mut app = App::new();
/// app.use_middleware("", logger(Some(LoggerConfig {
///     duration: true,
///     method: true,
///     path: true,
/// })));
/// ```
pub fn logger(
    config: Option<LoggerConfig>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + Clone + 'static {
    move |req, _| {
        let config = config.clone().unwrap_or_default();
        let req = req.clone();

        let start_time = std::time::Instant::now();
        let path = req.path.clone();
        let method = req.method.clone();

        Box::pin(async move {
            let duration = start_time.elapsed();

            if config.path {
                print!("path: {}, ", path);
            }

            if config.duration {
                print!("Time taken: {}ms, ", duration.as_millis());
            }

            if config.method {
                print!("method: {}", method);
            }

            println!();

            (req, None)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HttpMethods;

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
            duration: false,
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
            duration: false,
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
