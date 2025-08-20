use crate::{context::HttpResponse, req::HttpRequest, types::FutMiddleware};
use std::collections::HashMap;

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
    pub status: bool,
    pub user_agent: bool,
    pub ip: bool,
    pub headers: Vec<String>, // Specific headers to log
    pub body_size: bool,
    pub query_params: bool,
    pub exclude_paths: Vec<String>, // Don't log health checks, etc.
}

impl Default for LoggerConfig {
    fn default() -> Self {
        LoggerConfig {
            duration: true,
            method: true,
            path: true,
            status: true,
            user_agent: true,
            ip: true,
            headers: vec![],
            body_size: true,
            query_params: true,
            exclude_paths: vec![],
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
/// use ripress::app::App;
/// let mut app = App::new();
/// app.use_logger(None);
///
///```
///```
/// use ripress::{app::App, middlewares::logger::LoggerConfig};
/// let mut app = App::new();
/// app.use_logger(Some(LoggerConfig {
///     duration: true,
///     method: true,
///     path: true,
///     ..Default::default()
/// }));
/// ```
pub(crate) fn logger(
    config: Option<LoggerConfig>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + Clone + 'static {
    move |req, res| {
        let config = config.clone().unwrap_or_default();

        if config.exclude_paths.contains(&req.path) {
            return Box::pin(async move { (req, None) });
        }

        Box::pin(async move {
            let start_time = std::time::Instant::now();
            let path = req.path.clone();
            let method = req.method.clone();
            let status = res.status_code.clone();
            let user_agent = req.headers.user_agent().unwrap_or("Unknown").to_string();
            let ip = req.ip;
            let mut headers = HashMap::new();

            if !config.headers.is_empty() {
                for header in &config.headers {
                    headers.insert(
                        header.clone(),
                        req.headers.get(header).unwrap_or("None").to_string(),
                    );
                }
            }

            let query_params = req.query.clone();
            let body_size = res.body.len();

            let duration = start_time.elapsed();

            let mut msg = String::new();

            if config.path {
                msg.push_str(&format!("path: {}, ", path));
            }
            if config.status {
                msg.push_str(&format!("status: {}, ", status));
            }
            if config.user_agent {
                msg.push_str(&format!("user_agent: {}, ", user_agent));
            }
            if config.ip {
                msg.push_str(&format!("ip: {}, ", ip));
            }
            for (key, value) in headers {
                msg.push_str(&format!("{}: {}, ", key, value));
            }
            if config.query_params {
                msg.push_str(&format!("query_params: {:?}, ", query_params));
            }
            if config.body_size {
                msg.push_str(&format!("body_size: {}, ", body_size));
            }
            if config.duration {
                msg.push_str(&format!("duration_ms: {}, ", duration.as_millis()));
            }
            if config.method {
                msg.push_str(&format!("method: {}", method));
            }
            let msg = msg.trim_end_matches([',', ' ', '\t', '\n']);

            tracing::info!("{}", msg);

            (req, None)
        })
    }
}
