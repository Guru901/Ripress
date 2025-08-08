use crate::{context::HttpResponse, req::HttpRequest, types::FutMiddleware};

/// Configuration for the Cors Middleware
///
/// ## Fields
///
/// * `allowed_origin` - The allowed origin for the request
/// * `allowed_methods` - The allowed methods for the request
/// * `allow_credentials` - Whether to allow credentials

#[derive(Clone)]
pub struct CorsConfig {
    pub allowed_origin: &'static str,
    pub allowed_methods: &'static str,
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        CorsConfig {
            allowed_origin: "*",
            allowed_methods: "GET, POST, PUT, DELETE, OPTIONS",
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
///
///```
///```
/// use ripress::{app::App, middlewares::cors::{cors, CorsConfig}};
/// let mut app = App::new();
/// app.use_middleware("", cors(Some(CorsConfig {
///     allowed_origin: "https://example.com".to_string(),
///     allowed_methods: "GET, POST, PUT, DELETE, OPTIONS".to_string(),
///     allow_credentials: true,
/// })));
/// ```

pub fn cors(
    config: Option<CorsConfig>,
) -> impl Fn(&mut HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + Clone + 'static {
    move |req, mut res| {
        let config = config.clone().unwrap_or_default();
        let req: HttpRequest = req.clone();

        Box::pin(async move {
            res = res
                .set_header("Access-Control-Allow-Origin", config.allowed_origin)
                .set_header("Access-Control-Allow-Methods", config.allowed_methods)
                .set_header(
                    "Access-Control-Allow-Headers",
                    "Content-Type, Authorization",
                );

            if config.allow_credentials {
                res = res.set_header("Access-Control-Allow-Credentials", "true");
            }
            (req, Some(res))
        })
    }
}
