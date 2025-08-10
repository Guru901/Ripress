use crate::{
    context::HttpResponse,
    req::HttpRequest,
    types::{FutMiddleware, HttpMethods},
};

/// Configuration for the Cors Middleware
///
/// ## Fields
///
/// * `allowed_origin` - The allowed origin for the request
/// * `allowed_methods` - The allowed methods for the request
/// * `allowed_headers` - The allowed headers for the request
/// * `allow_credentials` - Whether to allow credentials
#[derive(Clone)]
pub struct CorsConfig {
    pub allowed_origin: &'static str,
    pub allowed_methods: &'static str,
    pub allowed_headers: &'static str,
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        CorsConfig {
            allowed_origin: "*",
            allowed_methods: "GET, POST, PUT, DELETE, OPTIONS",
            allowed_headers: "Content-Type, Authorization",
            allow_credentials: false,
        }
    }
}

/// Builtin Cors Middleware
///
/// ## Arguments
///
/// * `config` - Configuration for the middleware
///
/// ## Examples
///
/// ```
/// use ripress::{app::App, middlewares::cors::cors};
/// let mut app = App::new();
/// app.use_middleware("", cors(None));
/// ```
///
/// ```
/// use ripress::{app::App, middlewares::cors::{cors, CorsConfig}};
/// let mut app = App::new();
/// app.use_middleware("", cors(Some(CorsConfig {
///     allowed_origin: "https://example.com",
///     allowed_methods: "GET, POST, PUT, DELETE, OPTIONS",
///     allowed_headers: "Content-Type, Authorization",
///     allow_credentials: true,
/// })));
/// ```
pub fn cors(
    config: Option<CorsConfig>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + Clone + 'static {
    move |req, mut res| {
        let config = config.clone().unwrap_or_default();
        let req_clone = req.clone();

        Box::pin(async move {
            // Always add CORS headers
            res = res
                .set_header("Access-Control-Allow-Origin", config.allowed_origin)
                .set_header("Access-Control-Allow-Methods", config.allowed_methods)
                .set_header("Access-Control-Allow-Headers", config.allowed_headers);

            if config.allow_credentials {
                res = res.set_header("Access-Control-Allow-Credentials", "true");
            }

            // Handle preflight OPTIONS requests - terminate here with 200
            if req_clone.method == HttpMethods::OPTIONS {
                return (req_clone, Some(res.ok()));
            }

            // For all other requests, add CORS headers but continue to next handler
            (req_clone, None) // Continue to next middleware/handler
        })
    }
}

/// Alternative version that always continues (if you want CORS headers on all responses)
pub fn cors_passthrough(
    config: Option<CorsConfig>,
) -> impl Fn(&mut HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + Clone + 'static {
    move |req, mut res| {
        let config = config.clone().unwrap_or_default();
        let req_clone = req.clone();

        Box::pin(async move {
            // This version always continues to the next handler
            // CORS headers would need to be added by the actual route handlers
            // or by a response middleware that runs after route handlers

            // Store CORS config in request context for later use
            // (This would require extending HttpRequest to store metadata)

            (req_clone, None) // Always continue
        })
    }
}
