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

        // Treat entries as prefixes; excludes "/health" will also exclude "/health/live".
        if config
            .exclude_paths
            .iter()
            .any(|prefix| req.path.starts_with(prefix))
        {
            return Box::pin(async move { (req, None) });
        }

        Box::pin(async move {
            let path = req.path.clone();
            let method = req.method.clone();
            let user_agent = req.headers.user_agent().unwrap_or("Unknown").to_string();
            let ip = req.ip;
            let status_code = res.status_code;
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

            let mut msg = String::new();

            if config.path {
                msg.push_str(&format!("path: {}, \n", path));
            }
            if config.user_agent {
                msg.push_str(&format!("user_agent: {}, \n", user_agent));
            }
            if config.ip {
                msg.push_str(&format!("ip: {}, \n", ip));
            }
            for (key, value) in headers {
                msg.push_str(&format!("{}: {}, \n", key, value));
            }
            if config.status {
                msg.push_str(&format!("status_code: {}\n", status_code));
            }
            if config.query_params {
                msg.push_str(&format!("query_params: {:?}, \n", query_params));
            }
            if config.method {
                msg.push_str(&format!("method: {}, \n", method));
            }
            if config.body_size {
                msg.push_str(&format!("body_size: {}\n", res.body.len()));
            }

            let msg = msg.trim_end_matches([',', ' ', '\t', '\n']);

            println!("{}", msg);

            (req, None)
        })
    }
}
