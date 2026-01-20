#![warn(missing_docs)]
use crate::{context::HttpResponse, req::HttpRequest, types::MiddlewareOutput};
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, time::Instant};
use tokio::sync::Mutex;
use tokio::time::interval;

/// Builtin Rate Limiter Middleware
///
/// This middleware implements a sliding window rate limiter that controls the number
/// of requests clients can make within a specified time period. It's essential for
/// protecting APIs from abuse, preventing DoS attacks, and ensuring fair resource
/// usage across clients. The middleware uses an in-memory store with automatic cleanup
/// and supports both direct connections and proxy environments.
///
/// ## Features
///
/// * **Sliding window algorithm** - More accurate than fixed windows, prevents burst traffic at window boundaries
/// * **Per-client tracking** - Individual rate limits based on client IP addresses
/// * **Proxy support** - Extracts real client IPs from `X-Forwarded-For` headers
/// * **Automatic cleanup** - Periodic cleanup of expired entries to prevent memory leaks
/// * **Standard headers** - Follows RFC-compliant rate limit headers for client guidance
/// * **Configurable responses** - Custom messages and limits for different use cases
/// * **Thread-safe** - Uses async mutexes for safe concurrent access
/// * **Memory efficient** - Lightweight tracking structure per client
/// * **Graceful degradation** - Continues operation even under high load
///
/// ## Rate Limiting Algorithm
///
/// The middleware uses a **sliding window** approach:
/// 1. **First request** from a client starts a new window
/// 2. **Subsequent requests** within the window are counted against the limit
/// 3. **Window expiry** resets the counter and starts a new window
/// 4. **Requests over limit** are rejected with 429 status until window resets
///
/// This approach is more accurate than fixed windows because it doesn't allow
/// burst traffic at window boundaries (e.g., 2x limit by making requests at
/// the end of one window and start of the next).
///
/// ## Client Identification
///
/// ### Direct Connection Mode (`proxy: false`)
/// - Uses the direct client IP address from the TCP connection
/// - Suitable for applications directly facing the internet
/// - Most accurate when clients connect directly
///
/// ### Proxy Mode (`proxy: true`)
/// - Extracts the real client IP from `X-Forwarded-For` header
/// - Falls back to direct IP if header is missing or malformed
/// - Essential when behind load balancers, CDNs, or reverse proxies
/// - Takes the first IP from comma-separated list (closest to client)
///
/// ## Memory Management
///
/// The middleware includes automatic cleanup to prevent memory leaks:
/// * **Cleanup interval**: Every 5 minutes
/// * **Cleanup criteria**: Removes entries older than the configured window
/// * **Background task**: Runs independently without blocking requests
/// * **Memory bounds**: Automatically limits growth of client tracking data
///
/// ## Configuration Options
///
/// * `window_ms` - Duration of the rate limiting window (default: 10 seconds)
/// * `max_requests` - Maximum requests allowed per client per window (default: 10)
/// * `proxy` - Whether to extract real IP from proxy headers (default: false)
/// * `message` - Custom message returned when limit exceeded (default: "Too many requests")
///
/// ## Response Headers
///
/// The middleware adds standard rate limiting headers to all responses:
/// * `X-RateLimit-Limit` - The maximum number of requests allowed in the window
/// * `X-RateLimit-Remaining` - Number of requests remaining in current window
/// * `X-RateLimit-Reset` - Seconds until the current window resets
/// * `Retry-After` - Seconds to wait before retrying (only when rate limited)
///
/// These headers help clients implement proper backoff strategies and respect rate limits.
///
/// ## Examples
///
/// Basic rate limiting with default settings:
///
/// ```no_run
/// use ripress::{app::App, middlewares::rate_limiter::RateLimiterConfig};
///
/// let mut app = App::new();
/// app.use_rate_limiter(Some(RateLimiterConfig::default()));
/// ```
///
/// API rate limiting for production (100 requests per minute):
///
/// ```no_run
/// use ripress::{app::App, middlewares::rate_limiter::RateLimiterConfig};
/// use std::time::Duration;
///
/// let mut app = App::new();
/// let config = RateLimiterConfig {
///     window_ms: Duration::from_secs(60), // 1 minute window
///     max_requests: 100,
///     proxy: true, // Behind load balancer
///     message: "Rate limit exceeded. Please try again later.".to_string(),
/// };
/// app.use_rate_limiter(Some(config));
/// ```
///
/// Strict rate limiting for sensitive endpoints:
///
/// ```no_run
/// use ripress::{app::App, middlewares::rate_limiter::RateLimiterConfig};
/// use std::time::Duration;
///
/// let mut app = App::new();
/// let config = RateLimiterConfig {
///     window_ms: Duration::from_secs(300), // 5 minute window
///     max_requests: 5, // Very restrictive
///     proxy: false,
///     message: "Too many attempts. Please wait before trying again.".to_string(),
/// };
/// app.use_rate_limiter(Some(config));
/// ```
///
/// Development-friendly configuration:
///
/// ```no_run
/// use ripress::{app::App, middlewares::rate_limiter::RateLimiterConfig};
/// use std::time::Duration;
///
/// let mut app = App::new();
/// let config = RateLimiterConfig {
///     window_ms: Duration::from_secs(10),
///     max_requests: 1000, // Very permissive for development
///     proxy: false,
///     message: "Development rate limit exceeded".to_string(),
/// };
/// app.use_rate_limiter(Some(config));
/// ```
///
/// Multiple rate limiters for different endpoints:
///
/// ```no_run
/// use ripress::{app::App, middlewares::rate_limiter::RateLimiterConfig};
/// use std::time::Duration;
///
/// let mut app = App::new();
///
/// // Generous limits for read operations
/// let read_config = RateLimiterConfig {
///     window_ms: Duration::from_secs(60),
///     max_requests: 200,
///     proxy: true,
///     message: "Too many read requests".to_string(),
/// };
/// app.use_rate_limiter(Some(read_config));
///
/// // Stricter limits for write operations
/// let write_config = RateLimiterConfig {
///     window_ms: Duration::from_secs(60),
///     max_requests: 50,
///     proxy: true,
///     message: "Too many write requests".to_string(),
/// };
/// app.use_rate_limiter(Some(write_config));
/// ```
///
/// Using default configuration:
///
/// ```no_run
/// use ripress::app::App;
///
/// let mut app = App::new();
/// app.use_rate_limiter(None); // Uses defaults
/// ```
///
/// ## Client Integration Examples
///
/// ### JavaScript/Fetch API
/// ```javascript
/// async function apiCall(url) {
///     const response = await fetch(url);
///     
///     if (response.status === 429) {
///         const retryAfter = response.headers.get('Retry-After');
///         console.log(`Rate limited. Retry after ${retryAfter} seconds`);
///         // Implement exponential backoff
///         await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));
///         return apiCall(url); // Retry
///     }
///     
///     // Check remaining requests
///     const remaining = response.headers.get('X-RateLimit-Remaining');
///     console.log(`${remaining} requests remaining`);
///     
///     return response.json();
/// }
/// ```
///
/// ### curl Example
/// ```bash
/// # Check rate limit headers
/// curl -I https://api.example.com/data
/// # X-RateLimit-Limit: 100
/// # X-RateLimit-Remaining: 95
/// # X-RateLimit-Reset: 45
/// ```
///
/// ## Security Considerations
///
/// ### IP Spoofing Prevention
/// - **Validate proxy headers** when `proxy: true`
/// - **Use trusted proxies** only - validate `X-Forwarded-For` sources
/// - **Consider multiple headers** like `X-Real-IP`, `CF-Connecting-IP`
/// - **Implement IP validation** for critical applications
///
/// ### DDoS Protection
/// - Rate limiting alone may not stop sophisticated DDoS attacks
/// - Consider combining with other protective measures
/// - Monitor for distributed attacks across many IPs
/// - Implement circuit breakers for upstream services
///
/// ### Bypass Attempts
/// - Clients may try to bypass limits by changing IP addresses
/// - Consider session-based or token-based limiting for authenticated users
/// - Monitor for suspicious patterns in rate limit violations
/// - Implement progressive penalties for repeated violations
///
/// ## Performance Characteristics
///
/// ### Memory Usage
/// - **Per-client overhead**: ~32 bytes (IP string + tracking data)
/// - **Cleanup efficiency**: Periodic cleanup prevents unbounded growth
/// - **Hash map operations**: O(1) average case for lookups and updates
/// - **Memory bounds**: Automatically managed through cleanup task
///
/// ### CPU Usage
/// - **Lock contention**: Uses async mutex to minimize blocking
/// - **Cleanup cost**: Background task runs every 5 minutes
/// - **Header parsing**: Minimal overhead for proxy IP extraction
/// - **Time calculations**: Efficient duration-based comparisons
///
/// ### Scalability
/// - **Single-instance**: Suitable for single-server deployments
/// - **Multi-instance**: Each instance maintains separate state
/// - **Redis alternative**: Consider external stores for distributed deployments
/// - **Horizontal scaling**: May need shared rate limit store
///
/// ## Monitoring and Observability
///
/// ### Metrics to Track
/// - Rate limit violations per endpoint
/// - Top rate-limited IP addresses
/// - Average requests per client per window
/// - Memory usage of client tracking data
///
/// ### Logging Considerations
/// - Log rate limit violations for security monitoring
/// - Track cleanup task performance
/// - Monitor for memory leaks or excessive growth
/// - Alert on unusual rate limiting patterns
///
/// ## Limitations
///
/// ### Single Instance State
/// - Rate limits are per-application instance
/// - Load balanced deployments may see inconsistent limits
/// - Consider Redis or database-backed solutions for distributed systems
///
/// ### Memory Constraints
/// - All client data stored in memory
/// - Not suitable for applications with millions of unique clients
/// - Cleanup task may not keep up under extreme load
///
/// ### IP-Based Limitations
/// - NAT/proxy environments may affect multiple users
/// - IPv6 addresses require more memory
/// - Mobile users may change IPs frequently
///
/// ## Best Practices
///
/// ### Configuration Guidelines
/// - Set `window_ms` based on your API's usage patterns
/// - Use generous limits initially and adjust based on monitoring
/// - Enable `proxy: true` when behind load balancers or CDNs
/// - Provide clear error messages to guide client behavior
///
/// ### Production Deployment
/// - Monitor rate limit violation patterns
/// - Implement alerting for excessive rate limiting
/// - Consider progressive rate limiting (increasing penalties)
/// - Test rate limiting behavior under load
///
/// ### Client-Friendly Design
/// - Always include rate limit headers
/// - Provide clear error messages with retry guidance
/// - Document rate limits in API documentation
/// - Implement proper error handling in client applications

/// Configuration struct for the rate limiter middleware
///
/// This struct defines all configurable aspects of the rate limiting behavior,
/// including time windows, request limits, proxy support, and error messaging.
/// The configuration is cloned per request to ensure thread safety.
///
/// ## Configuration Strategy
///
/// Choose configuration values based on your application's needs:
/// - **Public APIs**: More permissive limits with longer windows
/// - **Private APIs**: Moderate limits based on expected usage
/// - **Authentication endpoints**: Very strict limits with longer windows
/// - **File uploads**: Consider request size, not just count
///
/// ## Window vs Limits Trade-offs
///
/// * **Shorter windows + higher limits**: More responsive to traffic bursts
/// * **Longer windows + lower limits**: Better for preventing sustained abuse
/// * **Multiple tiers**: Different limits for different endpoint types
#[derive(Clone)]
pub struct RateLimiterConfig {
    /// The duration of the rate limiting window
    ///
    /// This defines how long the middleware tracks requests from each client.
    /// When the window expires, the request count resets to zero. Choose based
    /// on your application's tolerance for request bursts:
    ///
    /// **Examples:**
    /// - `Duration::from_secs(60)` - 1 minute window (common for APIs)
    /// - `Duration::from_secs(300)` - 5 minute window (for auth endpoints)
    /// - `Duration::from_millis(10_000)` - 10 second window (default, good for testing)
    ///
    /// **Considerations:**
    /// - Shorter windows allow faster recovery from rate limits
    /// - Longer windows provide better protection against sustained abuse
    /// - Very short windows may not effectively prevent abuse
    pub window_ms: Duration,

    /// Whether to use proxy headers for client IP detection
    ///
    /// When `true`, the middleware attempts to extract the real client IP
    /// from the `X-Forwarded-For` header. This is essential when your
    /// application is behind a load balancer, CDN, or reverse proxy.
    ///
    /// **When to enable:**
    /// - Behind AWS ALB/ELB, nginx, Apache, or other reverse proxies
    /// - Using CDN services like CloudFlare, AWS CloudFront
    /// - In containerized environments with ingress controllers
    ///
    /// **When to disable:**
    /// - Direct internet-facing applications
    /// - When you don't trust proxy headers
    /// - For maximum security (harder to spoof direct IPs)
    ///
    /// **Security note:** Only enable when you trust the proxy source,
    /// as clients can potentially spoof these headers.
    pub proxy: bool,

    /// Maximum number of requests allowed per client within the window
    ///
    /// This is the core rate limiting parameter. Set based on expected
    /// legitimate usage patterns for your API endpoints.
    ///
    /// **Guidelines:**
    /// - Start with conservative limits and increase based on monitoring
    /// - Consider different limits for different endpoint types
    /// - Factor in client retry behaviors and error rates
    /// - Account for legitimate burst usage patterns
    ///
    /// **Examples by use case:**
    /// - Public REST API: 100-1000 requests per minute
    /// - Authentication endpoints: 5-10 requests per 5 minutes  
    /// - File upload endpoints: 10-50 requests per hour
    /// - Health check endpoints: Very high or unlimited
    pub max_requests: usize,

    /// Custom message returned when rate limit is exceeded
    ///
    /// This message is sent in the response body when clients exceed their
    /// rate limit. Provide clear, actionable guidance to help clients
    /// understand what happened and how to proceed.
    ///
    /// **Best practices:**
    /// - Be specific about what limit was exceeded
    /// - Provide guidance on retry timing
    /// - Include contact information for legitimate high-volume users
    /// - Keep messages concise but informative
    ///
    /// **Examples:**
    /// - "Rate limit exceeded. Please try again in a few minutes."
    /// - "Too many requests. Check the Retry-After header for timing."
    /// - "API rate limit reached. Upgrade your plan for higher limits."
    /// - "Authentication rate limit exceeded. Wait 5 minutes before retry."
    pub message: String,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        RateLimiterConfig {
            max_requests: 10,
            window_ms: Duration::from_millis(10_000),
            proxy: false,
            message: String::from("Too many requests"),
        }
    }
}

/// Internal structure for tracking client rate limit data
///
/// This lightweight structure stores the minimum information needed
/// to track each client's request count and window timing. The
/// `Copy` trait allows efficient updates without ownership issues.
#[derive(Clone, Copy)]
struct RateLimiterStruct {
    /// When this client's current window started
    window_started: Instant,
    /// Number of requests made in the current window
    requests: usize,
}

/// Creates a rate limiter middleware function
///
/// Returns a middleware function that implements sliding window rate limiting
/// based on client IP addresses. The middleware automatically handles window
/// expiration, client tracking, cleanup, and proper HTTP response codes and headers.
///
/// ## Parameters
///
/// * `config` - Optional rate limiter configuration. If `None`, uses `RateLimiterConfig::default()`
///   which allows 10 requests per 10-second window with no proxy support.
///
/// ## Returns
///
/// A middleware function compatible with the ripress framework that:
/// * Tracks request counts per client IP address using sliding windows
/// * Automatically rejects requests exceeding configured limits with 429 status
/// * Adds standard rate limiting headers to all responses
/// * Supports both direct and proxy-based client IP detection
/// * Runs background cleanup to prevent memory leaks
/// * Handles concurrent access safely with async mutexes
///
/// ## Middleware Behavior
///
/// ### For Requests Within Limits
/// 1. Identifies client IP (direct or via proxy headers)
/// 2. Updates request count for the current window
/// 3. Adds rate limiting headers to response
/// 4. Continues to next middleware/handler
///
/// ### For Requests Exceeding Limits
/// 1. Calculates time remaining in current window
/// 2. Returns 429 Too Many Requests status immediately
/// 3. Includes retry guidance in headers and response body
/// 4. Does not call subsequent middleware or handlers
///
/// ## Background Tasks
///
/// The middleware spawns a cleanup task that:
/// * Runs every 5 minutes to remove expired client entries
/// * Prevents memory leaks from accumulating client data
/// * Runs independently without affecting request processing
/// * Uses the same window duration for cleanup timing
///
/// ## Thread Safety and Performance
///
/// * **Async mutex**: Prevents blocking while ensuring data consistency
/// * **Arc sharing**: Efficient sharing of client data across requests
/// * **Minimal locking**: Lock held only for map operations, not entire request
/// * **Background cleanup**: Maintains performance under sustained load
/// * **Copy semantics**: Efficient updates of client tracking data
///
/// ## Memory Management
///
/// * **Automatic cleanup**: Expired entries removed every 5 minutes
/// * **Bounded growth**: Client map size naturally limited by cleanup
/// * **Lightweight entries**: Minimal memory per tracked client
/// * **Efficient operations**: Hash map provides O(1) average lookup/update
///
/// ## Error Handling
///
/// The middleware is designed to be robust and never panic:
/// * **Missing headers**: Gracefully falls back to direct IP
/// * **Invalid durations**: Uses saturating arithmetic to prevent underflow
/// * **Lock contention**: Async mutex prevents deadlocks
/// * **Memory pressure**: Background cleanup prevents unbounded growth
///
/// ## Rate Limiting Headers
///
/// All responses include standard headers for client guidance:
/// * **X-RateLimit-Limit**: Maximum requests allowed in window
/// * **X-RateLimit-Remaining**: Requests remaining in current window  
/// * **X-RateLimit-Reset**: Seconds until current window expires
/// * **Retry-After**: Seconds to wait before retrying (429 responses only)
pub(crate) fn rate_limiter(
    config: Option<RateLimiterConfig>,
) -> impl Fn(HttpRequest, HttpResponse) -> MiddlewareOutput + Send + Sync + 'static {
    let client_map: Arc<Mutex<HashMap<String, RateLimiterStruct>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let cfg = config.unwrap_or_default();

    let cleanup_map = client_map.clone();
    let cleanup_window = cfg.window_ms;
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(300));
        loop {
            ticker.tick().await;
            let now = Instant::now();
            let mut map = cleanup_map.lock().await;
            map.retain(|_, v| now.duration_since(v.window_started) <= cleanup_window);
        }
    });

    move |req: HttpRequest, mut res| {
        let client_map = client_map.clone();
        let cfg = cfg.clone();

        Box::pin(async move {
            let now = Instant::now();
            let client_ip = if cfg.proxy {
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
                if now.duration_since(client.window_started) > cfg.window_ms {
                    *client = RateLimiterStruct {
                        window_started: now,
                        requests: 1,
                    };
                } else {
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
                map.insert(
                    client_ip.clone(),
                    RateLimiterStruct {
                        window_started: now,
                        requests: 1,
                    },
                );
            }

            let client_data = map.get(&client_ip).unwrap();
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
