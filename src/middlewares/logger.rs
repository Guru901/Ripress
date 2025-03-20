use crate::{
    context::HttpResponse,
    request::HttpRequest,
    types::{Fut, Next},
};

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

pub fn logger(
    config: Option<LoggerConfig>,
) -> impl Fn(HttpRequest, HttpResponse, Next) -> Fut + Send + Sync + Clone + 'static {
    move |req, res, next| {
        let config = config.clone().unwrap_or_default();

        let start_time = std::time::Instant::now();
        let path = req.get_path().to_string();

        Box::pin(async move {
            let method = req.get_method();

            let res = next.run(req.clone(), res).await;
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

            println!("");

            res
        })
    }
}
