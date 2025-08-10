use crate::types::HandlerMiddleware;
use std::sync::Arc;

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
pub fn logger(config: Option<LoggerConfig>) -> HandlerMiddleware {
    Arc::new(move |req, _| {
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
    })
}
