#![warn(missing_docs)]
use crate::{context::HttpResponse, req::HttpRequest, types::FutMiddleware};
use std::collections::HashMap;

/// Configuration for the Logger Middleware
///
/// ## Fields
///
/// * `method` -  Whether to log the method
/// * `path` - Whether to log the path
/// * `status` - Whether to log the status code
/// * `user_agent` - Whether to log the user agent
/// * `ip` - Whether to log the IP address
/// * `headers` - Which headers to log
/// * `body_size` - Whether to log the body size
/// * `query_params` - Whether to log the query parameters
/// * `exclude_paths` - Paths to exclude from logging
/// (Duration logging is currently not supported in this middleware.)

#[derive(Clone)]
pub struct LoggerConfig {
    /// Whether to log the method
    pub method: bool,
    /// Whether to log the path
    pub path: bool,
    /// Whether to log the status code
    pub status: bool,
    /// Whether to log the user agent
    pub user_agent: bool,
    /// Whether to log the IP address
    pub ip: bool,
    /// Specific headers to log
    pub headers: Vec<String>, // Specific headers to log
    /// Whether to log the body size
    pub body_size: bool,
    /// Whether to log the query parameters
    pub query_params: bool,
    /// Don't log health checks, etc.
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

pub(crate) fn logger(
    config: Option<LoggerConfig>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static {
    let cfg = std::sync::Arc::new(config.unwrap_or_default());
    move |req, res| {
        let config = std::sync::Arc::clone(&cfg);

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
                    let key = header.to_ascii_lowercase();
                    let value = req
                        .headers
                        .get(&key)
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "<missing>".to_string());
                    headers.insert(key, value);
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
                if res.is_stream {
                    msg.push_str("body_size: stream\n");
                } else {
                    msg.push_str(&format!("body_size: {}\n", res.body.len()));
                }
            }

            let msg = msg.trim_end_matches([',', ' ', '\t', '\n']);

            println!("{}", msg);

            (req, None)
        })
    }
}
