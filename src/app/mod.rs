#![warn(missing_docs)]

use crate::app::api_error::ApiError;
#[cfg(feature = "with-wynd")]
use crate::helpers::exec_wynd_middleware;
use crate::helpers::{exec_post_middleware, exec_pre_middleware};
use crate::middlewares::body_limit::body_limit;
use crate::middlewares::compression::{CompressionConfig, compression};
use crate::middlewares::cors::{CorsConfig, cors};
use crate::middlewares::logger::{LoggerConfig, logger};
use crate::middlewares::rate_limiter::{RateLimiterConfig, rate_limiter};
use crate::middlewares::shield::{ShieldConfig, shield};
use crate::req::HttpRequest;
use crate::res::HttpResponse;
#[cfg(feature = "with-wynd")]
use crate::types::WyndMiddlewareHandler;
use crate::types::{Fut, FutMiddleware, HandlerMiddleware, HttpMethods, RouterFns, Routes};
use hyper::header;
use hyper::http::StatusCode;
use hyper::{Body, Request, Response, Server};
use hyper_staticfile::Static;
use routerify::ext::RequestExt;
use routerify::{Router, RouterService};
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::path::Path;
#[cfg(feature = "with-wynd")]
use std::pin::Pin;
use std::sync::Arc;

pub(crate) mod api_error;

pub(crate) fn box_future<F>(future: F) -> Fut
where
    F: Future<Output = HttpResponse> + Send + 'static,
{
    Box::pin(future)
}

pub(crate) fn box_future_middleware<F>(future: F) -> FutMiddleware
where
    F: Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static,
{
    Box::pin(future)
}

/// Represents a middleware in the Ripress application.
///
/// A `Middleware` consists of a function and an associated path. The function is an
/// asynchronous closure or function that takes an [`HttpRequest`] and [`HttpResponse`],
/// and returns a future resolving to a tuple of the potentially modified request and an
/// optional response. If the middleware returns `Some(response)`, the response is sent
/// immediately and further processing is halted. If it returns `None`, the request
/// continues through the middleware chain and to the route handler.
///
/// The `path` field specifies the route prefix or pattern for which this middleware
/// should be applied. Middlewares are matched in the order they are added to the app.

#[derive(Clone)]
pub struct Middleware {
    /// The middleware function.
    ///
    /// This is an `Arc`-wrapped closure or function pointer that takes an [`HttpRequest`]
    /// and [`HttpResponse`], and returns a boxed future resolving to a tuple of the
    /// (possibly modified) request and an optional response.
    pub func: Arc<dyn Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static>,

    /// The path or route prefix this middleware applies to.
    ///
    /// If the incoming request path starts with this string, the middleware will be invoked.
    pub path: String,

    pub(crate) name: String,
}

#[cfg(feature = "with-wynd")]
#[derive(Clone)]
pub(crate) struct WyndMiddleware {
    pub func: WyndMiddlewareHandler,
    pub path: String,
}

/// The App struct is the core of Ripress, providing a simple interface for creating HTTP servers and handling requests. It follows an Express-like pattern for route handling.
pub struct App {
    routes: Routes,
    pub(crate) middlewares: Vec<Middleware>,
    pub(crate) static_files: HashMap<&'static str, &'static str>,

    #[cfg(feature = "with-wynd")]
    pub(crate) wynd_middleware: Option<WyndMiddleware>,
}

impl RouterFns for App {
    fn routes(&mut self) -> &mut Routes {
        &mut self.routes
    }
}

impl App {
    /// Creates a new App instance.
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    ///
    /// let mut app = App::new();
    /// ```
    pub fn new() -> Self {
        App {
            routes: HashMap::new(),
            middlewares: Vec::new(),
            static_files: HashMap::new(),
            #[cfg(feature = "with-wynd")]
            wynd_middleware: None,
        }
    }

    /// Add a middleware to the application.
    ///
    /// ## Arguments
    ///
    /// * `middleware` - The middleware to add.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    /// let mut app = App::new();
    ///
    /// app.use_middleware("path", |req, _res| async move {
    ///     let mut req = req.clone();
    ///     (req, None)
    /// });
    ///
    /// ```

    pub fn use_middleware<F, Fut, P>(&mut self, path: P, middleware: F) -> &mut Self
    where
        P: Into<Option<&'static str>>,
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static,
    {
        let path = path.into().unwrap_or("/").to_string();
        self.middlewares.push(Middleware {
            func: Self::middleware_from_closure(middleware),
            path: path,
            name: "custom".to_string(),
        });
        self
    }

    /// Adds a logger middleware to the application.
    ///
    /// ## Arguments
    ///
    /// * `Option<LoggerConfig>` - The configuration for the logger.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    /// use ripress::middlewares::logger::LoggerConfig;
    ///
    /// let mut app = App::new();
    ///
    /// app.use_logger(None);
    ///
    /// app.use_logger(Some(LoggerConfig {
    ///     method: true,
    ///     path: true,
    ///     ..Default::default()
    /// }));
    ///
    /// ```

    pub fn use_logger(&mut self, config: Option<LoggerConfig>) -> &mut Self {
        self.middlewares.push(Middleware {
            func: Self::middleware_from_closure(logger(config)),
            path: "/".to_string(),
            name: "logger".to_string(),
        });
        self
    }

    /// Adds a cors middleware to the application.
    ///
    /// ## Arguments
    ///
    /// * `Option<CorsConfig>` - The configuration for the cors middleware.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    /// use ripress::middlewares::cors::CorsConfig;
    ///
    /// let mut app = App::new();
    ///
    /// app.use_cors(None);
    ///
    /// app.use_cors(Some(CorsConfig {
    ///     allowed_origin: "https://example.com",
    ///     allowed_methods: "GET, POST, PUT, DELETE, OPTIONS, HEAD",
    ///     ..Default::default()
    /// }));
    ///
    /// ```

    pub fn use_cors(&mut self, config: Option<CorsConfig>) -> &mut Self {
        self.middlewares.push(Middleware {
            func: Self::middleware_from_closure(cors(config)),
            path: "/".to_string(),
            name: "cors".to_string(),
        });
        self
    }

    /// Adds a body size limit middleware to the application.
    ///
    /// ## Arguments
    ///
    /// * `config` - An optional maximum size in bytes for the request body. If `None` is provided,
    ///   the default limit is 1 MB (1,048,576 bytes).
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    ///
    /// let mut app = App::new();
    ///
    /// // Use the default 1 MB limit
    /// app.use_body_limit(None);
    ///
    /// // Set a custom limit (e.g., 2 MB)
    /// app.use_body_limit(Some(2 * 1024 * 1024));
    /// ```
    pub fn use_body_limit(&mut self, config: Option<usize>) -> &mut Self {
        self.middlewares.push(Middleware {
            func: Self::middleware_from_closure(body_limit(config)),
            path: "/".to_string(),
            name: "body_limit".to_string(),
        });
        self
    }

    #[cfg(feature = "with-wynd")]
    /// Adds wynd middleware to the application to use websockets.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::{app::App, types::RouterFns};
    /// use wynd::wynd::Wynd;

    /// #[tokio::main]
    /// async fn main() {
    ///     let mut app = App::new();
    ///     let mut wynd = Wynd::new();

    ///     wynd.on_connection(|conn| async move {
    ///         conn.on_text(|event, handle| async move {
    ///             println!("{}", event.data);
    ///         });
    ///     });

    ///     app.get("/", |_, res| async move { res.ok().text("Hello World!") });

    ///     app.use_wynd("/ws", wynd.handler());
    /// }
    /// ```
    pub fn use_wynd<F, Fut>(&mut self, path: &'static str, handler: F) -> &mut Self
    where
        F: Fn(hyper::Request<hyper::Body>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = hyper::Result<hyper::Response<hyper::Body>>> + Send + 'static,
    {
        self.wynd_middleware = Some(WyndMiddleware {
            func: Self::wynd_middleware_from_closure(handler),
            path: path.to_string(),
        });
        self
    }

    /// Adds a rate limiter middleware to the application.
    ///
    /// ## Arguments
    ///
    /// * `Option<RateLimiterConfig>` - The configuration for the rate limiter middleware.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use ripress::app::App;
    /// use ripress::middlewares::rate_limiter::RateLimiterConfig;
    ///
    /// let mut app = App::new();
    ///
    /// app.use_rate_limiter(None);
    ///
    /// app.use_rate_limiter(Some(RateLimiterConfig {
    ///     max_requests: 10,
    ///     ..Default::default()
    /// }));
    ///
    /// ```

    pub fn use_rate_limiter(&mut self, config: Option<RateLimiterConfig>) -> &mut Self {
        self.middlewares.push(Middleware {
            func: Self::middleware_from_closure(rate_limiter(config)),
            path: "/".to_string(),
            name: "rate_limiter".to_string(),
        });
        self
    }

    /// Adds a security middleware (shield) to the application.
    ///
    /// The shield middleware helps protect your application from common web vulnerabilities
    /// by setting various HTTP headers and applying security best practices.
    ///
    /// ## Arguments
    ///
    /// * `config` - An optional [`ShieldConfig`] to customize the shield middleware's behavior.
    ///   If `None` is provided, default security settings will be applied.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    /// use ripress::middlewares::shield::{ShieldConfig, Hsts};
    ///
    /// let mut app = App::new();
    ///
    /// // Use default shield settings
    /// app.use_shield(None);
    ///
    /// // Use custom shield configuration
    /// app.use_shield(Some(ShieldConfig {
    ///     hsts: Hsts {
    ///         enabled: true,
    ///         ..Default::default()
    ///     },
    ///     ..Default::default()
    /// }));
    /// ```
    pub fn use_shield(&mut self, config: Option<ShieldConfig>) -> &mut Self {
        self.middlewares.push(Middleware {
            func: Self::middleware_from_closure(shield(config)),
            path: "/".to_string(),
            name: "shield".to_string(),
        });
        self
    }

    /// Adds a compression middleware to the application.
    ///
    /// ## Arguments
    ///
    /// Adds a compression middleware to the application.
    ///
    /// ## Arguments
    ///
    /// * `Option<CompressionConfig>` - The configuration for the compression middleware.
    ///
    /// ## Example

    pub fn use_compression(&mut self, config: Option<CompressionConfig>) -> &mut Self {
        self.middlewares.push(Middleware {
            func: Self::middleware_from_closure(compression(config)),
            path: "/".to_string(),
            name: "compression".to_string(),
        });
        self
    }

    fn middleware_from_closure<F, Fut>(f: F) -> HandlerMiddleware
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static,
    {
        Arc::new(move |req, res| box_future_middleware(f(req, res)))
    }

    #[cfg(feature = "with-wynd")]
    fn wynd_middleware_from_closure<F, Fut>(f: F) -> WyndMiddlewareHandler
    where
        F: Fn(hyper::Request<hyper::Body>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = hyper::Result<hyper::Response<hyper::Body>>>
            + Send
            + 'static,
    {
        Arc::new(move |req| Box::pin(f(req)))
    }

    /// Add a static file server to the application.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to mount the static files at (e.g., "/public", "/static", or "/" for root)
    /// * `file` - The path to the directory containing static files (e.g., "./public", "./assets")
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::app::App;
    ///
    /// let mut app = App::new();
    ///
    /// // Mount static files at a specific path
    /// app.static_files("/public", "./public");
    /// app.static_files("/assets", "./dist/assets");
    ///
    /// // Mount static files at root (useful for SPAs)
    /// app.static_files("/", "./dist");
    /// ```
    ///
    /// ## Important Notes
    ///
    /// - Serving from filesystem root "/" is blocked for security reasons
    /// - When mounting at "/", static files will be served as fallback (API routes take precedence)
    /// - Always use specific directories like "./public" for the file path
    pub fn static_files(
        &mut self,
        path: &'static str,
        file: &'static str,
    ) -> Result<(), &'static str> {
        // Validate inputs
        if file == "/" {
            return Err("Serving from filesystem root '/' is not allowed for security reasons");
        }
        if path.is_empty() {
            return Err("Mount path cannot be empty");
        }
        if file.is_empty() {
            return Err("File path cannot be empty");
        }
        // Require paths to start with '/'
        if !path.starts_with('/') {
            return Err("Mount path must start with '/'");
        }
        self.static_files.insert(path, file);
        Ok(())
    }

    /// Starts the server and listens on the specified address.
    ///
    /// ## Arguments
    ///
    /// * `addr` - The address to listen on e.g. "127.0.0.1:3000".
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse} };
    /// use tokio;
    ///
    /// ##[tokio::main]
    /// async fn main() {
    ///     let mut app = App::new();
    ///     app.listen(3000, || {println!("server running on port 3000")}).await;
    /// }
    ///
    /// ```

    pub async fn listen<F: FnOnce()>(&self, port: u16, cb: F) {
        let mut router = Router::<Body, ApiError>::builder();

        #[cfg(feature = "with-wynd")]
        if let Some(middleware) = &self.wynd_middleware {
            router = router.middleware(routerify::Middleware::pre({
                let middleware = middleware.clone();
                move |req| exec_wynd_middleware(req, middleware.clone())
            }));
        }

        // Apply middlewares first
        for middleware in self.middlewares.iter() {
            let middleware = middleware.clone();

            if middleware.name == "logger" || middleware.name == "compression" {
                router = router.middleware(routerify::Middleware::post_with_info({
                    let middleware = middleware.clone();
                    move |res, info| exec_post_middleware(res, middleware.clone(), info)
                }));
            } else {
                router = router.middleware(routerify::Middleware::pre({
                    let middleware = middleware.clone();
                    move |req| exec_pre_middleware(req, middleware.clone())
                }));
            }
        }

        // Register API routes FIRST (before static files)
        // This ensures API routes take precedence over static file serving
        for (path, methods) in &self.routes {
            for (method, handler) in methods {
                let handler = handler.clone();
                match method {
                    HttpMethods::GET => {
                        router = router.get(path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req = match HttpRequest::from_hyper_request(&mut req)
                                    .await
                                {
                                    Ok(r) => r,
                                    Err(e) => {
                                        return Err(ApiError::Generic(
                                            HttpResponse::new().bad_request().text(e.to_string()),
                                        ));
                                    }
                                };
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });

                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                match response.to_hyper_response() {
                                    Ok(r) => Ok(r),
                                    Err(_e) => Err(ApiError::Generic(
                                        HttpResponse::new()
                                            .bad_request()
                                            .text("Failed to create response"),
                                    )),
                                }
                            }
                        });
                    }
                    HttpMethods::POST => {
                        router = router.post(path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req = match HttpRequest::from_hyper_request(&mut req)
                                    .await
                                {
                                    Ok(r) => r,
                                    Err(e) => {
                                        return Err(ApiError::Generic(
                                            HttpResponse::new().bad_request().text(e.to_string()),
                                        ));
                                    }
                                };
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });

                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                match response.to_hyper_response() {
                                    Ok(r) => Ok(r),
                                    Err(_e) => Err(ApiError::Generic(
                                        HttpResponse::new()
                                            .bad_request()
                                            .text("Failed to create response"),
                                    )),
                                }
                            }
                        });
                    }
                    HttpMethods::PUT => {
                        router = router.put(path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req = match HttpRequest::from_hyper_request(&mut req)
                                    .await
                                {
                                    Ok(r) => r,
                                    Err(e) => {
                                        return Err(ApiError::Generic(
                                            HttpResponse::new().bad_request().text(e.to_string()),
                                        ));
                                    }
                                };
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                match response.to_hyper_response() {
                                    Ok(r) => Ok(r),
                                    Err(_e) => Err(ApiError::Generic(
                                        HttpResponse::new()
                                            .bad_request()
                                            .text("Failed to create response"),
                                    )),
                                }
                            }
                        });
                    }
                    HttpMethods::DELETE => {
                        router = router.delete(path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req = match HttpRequest::from_hyper_request(&mut req)
                                    .await
                                {
                                    Ok(r) => r,
                                    Err(e) => {
                                        return Err(ApiError::Generic(
                                            HttpResponse::new().bad_request().text(e.to_string()),
                                        ));
                                    }
                                };
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                match response.to_hyper_response() {
                                    Ok(r) => Ok(r),
                                    Err(_e) => Err(ApiError::Generic(
                                        HttpResponse::new()
                                            .bad_request()
                                            .text("Failed to create response"),
                                    )),
                                }
                            }
                        });
                    }
                    HttpMethods::PATCH => {
                        router = router.patch(path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req = match HttpRequest::from_hyper_request(&mut req)
                                    .await
                                {
                                    Ok(r) => r,
                                    Err(e) => {
                                        return Err(ApiError::Generic(
                                            HttpResponse::new().bad_request().text(e.to_string()),
                                        ));
                                    }
                                };
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                match response.to_hyper_response() {
                                    Ok(r) => Ok(r),
                                    Err(_e) => Err(ApiError::Generic(
                                        HttpResponse::new()
                                            .bad_request()
                                            .text("Failed to create response"),
                                    )),
                                }
                            }
                        });
                    }
                    HttpMethods::HEAD => {
                        router = router.head(path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req = match HttpRequest::from_hyper_request(&mut req)
                                    .await
                                {
                                    Ok(r) => r,
                                    Err(e) => {
                                        return Err(ApiError::Generic(
                                            HttpResponse::new().bad_request().text(e.to_string()),
                                        ));
                                    }
                                };
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                match response.to_hyper_response() {
                                    Ok(r) => Ok(r),
                                    Err(_e) => Err(ApiError::Generic(
                                        HttpResponse::new()
                                            .bad_request()
                                            .text("Failed to create response"),
                                    )),
                                }
                            }
                        });
                    }
                    HttpMethods::OPTIONS => {
                        router = router.options(path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req = match HttpRequest::from_hyper_request(&mut req)
                                    .await
                                {
                                    Ok(r) => r,
                                    Err(e) => {
                                        return Err(ApiError::Generic(
                                            HttpResponse::new().bad_request().text(e.to_string()),
                                        ));
                                    }
                                };
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                match response.to_hyper_response() {
                                    Ok(r) => Ok(r),
                                    Err(_e) => Err(ApiError::Generic(
                                        HttpResponse::new()
                                            .bad_request()
                                            .text("Failed to create response"),
                                    )),
                                }
                            }
                        });
                    }
                }
            }
        }

        for (mount_path, serve_from) in self.static_files.iter() {
            let serve_from = (*serve_from).to_string();
            let mount_root = (*mount_path).to_string();

            let route_pattern_owned = if mount_root == "/" {
                "/*".to_string()
            } else {
                format!("{}/{}", mount_root, "*")
            };

            let route_pattern: &'static str = Box::leak(route_pattern_owned.into_boxed_str());

            let serve_from_clone = serve_from.clone();
            let mount_root_clone = mount_root.clone();

            router = router.get(route_pattern, move |req| {
                let serve_from = serve_from_clone.clone();
                let mount_root = mount_root_clone.clone();
                async move {
                    match Self::serve_static_with_headers(req, mount_root, serve_from).await {
                        Ok(res) => Ok(res),
                        Err(e) => Err(ApiError::Generic(
                            HttpResponse::new()
                                .internal_server_error()
                                .text(e.to_string()),
                        )),
                    }
                }
            });
        }

        router = router.err_handler(Self::error_handler);
        let router = router.build().unwrap();

        cb();

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let service = RouterService::new(router).unwrap();
        let server = Server::bind(&addr).serve(service);

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }

    pub(crate) async fn error_handler(err: routerify::RouteError) -> Response<Body> {
        let api_err = err.downcast::<ApiError>().unwrap_or_else(|_| {
            return Box::new(ApiError::Generic(
                HttpResponse::new()
                    .internal_server_error()
                    .text("Unhandled error"),
            ));
        });

        match api_err.as_ref() {
            ApiError::Generic(res) => <HttpResponse as Clone>::clone(res)
                .to_hyper_response()
                .map_err(ApiError::from)
                .unwrap(),
        }
    }

    pub(crate) async fn serve_static_with_headers(
        req: Request<Body>,
        mount_root: String,
        fs_root: String,
    ) -> Result<Response<Body>, std::io::Error> {
        // Rewrite the request URI by stripping the mount_root prefix so that
        // "/static/index.html" maps to "fs_root/index.html" rather than
        // "fs_root/static/index.html".
        let (mut parts, body) = req.into_parts();
        let original_uri = parts.uri.clone();
        let original_path = original_uri.path();
        let if_none_match = parts
            .headers
            .get(header::IF_NONE_MATCH)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let trimmed_path = if mount_root == "/" {
            // If mounting at root, serve the path as-is
            original_path
        } else if original_path.starts_with(&mount_root) {
            // Strip the mount root prefix, but ensure we don't create an empty path
            let remaining = &original_path[mount_root.len()..];
            if remaining.is_empty() { "/" } else { remaining }
        } else {
            // Path doesn't match mount root - this shouldn't happen in normal routing
            original_path
        };

        let normalized_path = if trimmed_path.is_empty() {
            "/"
        } else {
            trimmed_path
        };

        let new_path_and_query = if let Some(query) = original_uri.query() {
            format!("{}?{}", normalized_path, query)
        } else {
            normalized_path.to_string()
        };

        parts.uri = match new_path_and_query.parse() {
            Ok(uri) => uri,
            Err(e) => {
                eprintln!(
                    "Error parsing URI: {} (original: {}, mount_root: {}, trimmed: {}, normalized: {})",
                    e, original_path, mount_root, trimmed_path, normalized_path
                );
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid URI after rewriting: {}", e),
                ));
            }
        };

        let rewritten_req = Request::from_parts(parts, body);

        let static_service = Static::new(Path::new(fs_root.as_str()));

        match static_service.serve(rewritten_req).await {
            Ok(mut response) => {
                response
                    .headers_mut()
                    .insert("Cache-Control", "public, max-age=86400".parse().unwrap());
                response
                    .headers_mut()
                    .insert("X-Served-By", "hyper-staticfile".parse().unwrap());
                // Handle conditional request with If-None-Match since hyper-staticfile 0.9
                // does not evaluate it. If ETag matches, return 304 with empty body.
                if let Some(if_none_match_value) = if_none_match {
                    if let Some(etag) = response.headers().get(header::ETAG) {
                        if let Ok(etag_value) = etag.to_str() {
                            if if_none_match_value == etag_value {
                                let mut builder =
                                    Response::builder().status(StatusCode::NOT_MODIFIED);
                                if let Some(h) = builder.headers_mut() {
                                    // carry forward ETag, Cache-Control, Last-Modified, etc.
                                    for (k, v) in response.headers().iter() {
                                        h.insert(k.clone(), v.clone());
                                    }
                                    h.remove(header::CONTENT_LENGTH);
                                }
                                return Ok(builder.body(Body::empty()).unwrap());
                            }
                        }
                    }
                }
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }

    pub(crate) fn _build_router(&self) -> routerify::Router<Body, ApiError> {
        routerify::Router::builder()
            .err_handler(Self::error_handler)
            .build()
            .unwrap()
    }
}
