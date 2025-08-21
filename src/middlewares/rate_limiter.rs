#![warn(missing_docs)]
use crate::{context::HttpResponse, req::HttpRequest, types::FutMiddleware};
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, time::Instant};
use tokio::sync::Mutex;
use tokio::time::interval;

#[derive(Clone)]
pub struct RateLimiterConfig {
    pub window_ms: Duration,
    pub proxy: bool,
    pub max_requests: usize,
    pub message: String,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        RateLimiterConfig {
            max_requests: 10,
            window_ms: Duration::from_millis(10_000), // 10 seconds
            proxy: false,
            message: String::from("Too many requests"),
        }
    }
}

#[derive(Clone, Copy)]
struct RateLimiterStruct {
    window_started: Instant,
    requests: usize,
}

pub(crate) fn rate_limiter(
    config: Option<RateLimiterConfig>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static {
    let client_map: Arc<Mutex<HashMap<String, RateLimiterStruct>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let cfg = config.unwrap_or_default();

    // Start cleanup task once, outside the middleware closure
    let cleanup_map = client_map.clone();
    let cleanup_window = cfg.window_ms;
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(300)); // 5 minutes
        loop {
            ticker.tick().await;
            let now = Instant::now();
            let mut map = cleanup_map.lock().await;
            map.retain(|_, v| now.duration_since(v.window_started) <= cleanup_window);
        }
    });

    move |req, mut res| {
        let client_map = client_map.clone();
        let cfg = cfg.clone();

        Box::pin(async move {
            let now = Instant::now();
            let client_ip = if cfg.proxy {
                // Extract real IP from X-Forwarded-For or similar headers when behind proxy
                req.headers
                    .get("X-Forwarded-For")
                    .and_then(|h| h.split(',').next())
                    .map(|ip| ip.trim().to_string())
                    .unwrap_or_else(|| req.ip.to_string())
            } else {
                req.ip.to_string()
            };

            let mut map = client_map.lock().await;

            if let Some(client) = map.get_mut(&client_ip) {
                // Check if window has expired
                if now.duration_since(client.window_started) > cfg.window_ms {
                    // Reset the window
                    *client = RateLimiterStruct {
                        window_started: now,
                        requests: 1,
                    };
                } else {
                    // Within the current window
                    if client.requests >= cfg.max_requests {
                        let remaining_time = cfg
                            .window_ms
                            .saturating_sub(now.duration_since(client.window_started))
                            .as_secs();

                        let limit = cfg.max_requests.to_string();
                        let retry = remaining_time.to_string();
                        res = res
                            .status(429)
                            .text(cfg.message.clone())
                            .set_header("X-RateLimit-Limit", &limit)
                            .set_header("X-RateLimit-Remaining", "0")
                            .set_header("X-RateLimit-Reset", &retry)
                            .set_header("Retry-After", &retry);
                        return (req, Some(res));
                    } else {
                        client.requests += 1;
                    }
                }
            } else {
                // New client
                map.insert(
                    client_ip.clone(),
                    RateLimiterStruct {
                        window_started: now,
                        requests: 1,
                    },
                );
            }

            let client_data = map.get(&client_ip).unwrap(); // Safe because we just inserted/updated
            let remaining_requests = cfg.max_requests.saturating_sub(client_data.requests);
            let window_remaining = cfg
                .window_ms
                .saturating_sub(now.duration_since(client_data.window_started))
                .as_secs();

            res.headers
                .insert("X-RateLimit-Limit", cfg.max_requests.to_string());
            res.headers
                .insert("X-RateLimit-Remaining", remaining_requests.to_string());
            res.headers
                .insert("X-RateLimit-Reset", window_remaining.to_string());

            (req, None)
        })
    }
}
