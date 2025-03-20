use crate::{
    context::HttpResponse,
    request::HttpRequest,
    types::{Fut, Next},
};

#[derive(Clone)]
pub struct CorsConfig {
    pub allowed_origin: String,
    pub allowed_methods: String,
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        CorsConfig {
            allowed_origin: "*".to_string(),
            allowed_methods: "GET, POST, PUT, DELETE, OPTIONS".to_string(),
            allow_credentials: false,
        }
    }
}

pub fn cors(
    config: Option<CorsConfig>,
) -> impl Fn(HttpRequest, HttpResponse, Next) -> Fut + Send + Sync + Clone + 'static {
    move |req, mut res, next| {
        let config = config.clone().unwrap_or_default();

        Box::pin(async move {
            res = res
                .set_header("Access-Control-Allow-Origin", &config.allowed_origin)
                .set_header("Access-Control-Allow-Methods", &config.allowed_methods)
                .set_header(
                    "Access-Control-Allow-Headers",
                    "Content-Type, Authorization",
                );

            if config.allow_credentials {
                res = res.set_header("Access-Control-Allow-Credentials", "true");
            }

            // if req.method() == "OPTIONS" {
            //     return res.ok().text(""); // Preflight response
            // }

            next.run(req, res).await
        })
    }
}
