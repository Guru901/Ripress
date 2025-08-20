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
            allowed_methods: "GET, POST, PUT, DELETE, OPTIONS, HEAD",
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
///     allowed_methods: "GET, POST, PUT, DELETE, OPTIONS, HEAD",
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
            let origin = req.headers.get("Origin");
            let allowed_methods = req.headers.get("Access-Control-Request-Method");
            let requested_headers = req.headers.get("Access-Control-Request-Headers");

            if let (Some(origin), Some(allowed_methods)) = (origin, allowed_methods) {
                res = res
                    .set_header("Access-Control-Allow-Origin", origin)
                    .set_header("Access-Control-Allow-Methods", allowed_methods)
                    .set_header(
                        "Access-Control-Allow-Headers",
                        requested_headers.unwrap_or(config.allowed_headers),
                    )
                    .set_header(
                        "Vary",
                        "Origin, Access-Control-Request-Method, Access-Control-Request-Headers",
                    );
            } else {
                res = res
                    .set_header("Access-Control-Allow-Origin", config.allowed_origin)
                    .set_header("Access-Control-Allow-Methods", config.allowed_methods)
                    .set_header("Access-Control-Allow-Headers", config.allowed_headers)
            }
            // Note: when not reflecting, Vary is not strictly required; keep defaults minimal.
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
