#![warn(missing_docs)]

use crate::helpers::exec_middleware;
use crate::req::HttpRequest;
use crate::res::HttpResponse;
use crate::types::{Fut, FutMiddleware, HttpMethods, RouterFns, Routes};
use hyper::header;
use hyper::http::StatusCode;
use hyper::{Body, Request, Response, Server};
use hyper_staticfile::Static;
use routerify::ext::RequestExt;
use routerify::{Router, RouterService};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

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

#[derive(Clone)]
pub struct Middleware {
    pub func: Arc<dyn Fn(&mut HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static>,
    pub path: String,
}

#[derive(Debug)]
pub enum ApiError {
    Generic(HttpResponse),
}

unsafe impl Sync for ApiError {}

impl std::error::Error for ApiError {}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApiError::Generic(msg) => {
                write!(f, "Middleware Error: {:?}", msg)
            }
        }
    }
}

impl From<HttpResponse> for ApiError {
    fn from(res: HttpResponse) -> Self {
        ApiError::Generic(res)
    }
}

impl From<ApiError> for Box<dyn std::error::Error + Send> {
    fn from(error: ApiError) -> Self {
        Box::new(error)
    }
}

/// The App struct is the core of Ripress, providing a simple interface for creating HTTP servers and handling requests. It follows an Express-like pattern for route handling.
pub struct App {
    routes: Routes,
    middlewares: Vec<Middleware>,
    pub(crate) static_files: HashMap<&'static str, &'static str>,
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
    /// app.use_middleware("path", |req, res| {
    ///     let mut req = req.clone();
    ///     Box::pin(async move {
    ///         (req, None)
    ///     })
    /// });
    ///
    /// ```

    pub fn use_middleware<F, Fut, P>(&mut self, path: P, middleware: F) -> &mut Self
    where
        P: Into<Option<&'static str>>,
        F: Fn(&mut HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static,
    {
        let path = path.into().unwrap_or("/").to_string();

        self.middlewares.push(Middleware {
            func: Arc::new(move |req, res| -> crate::types::FutMiddleware {
                box_future_middleware(middleware(req, res))
            }),
            path: path,
        });

        self
    }

    /// Add a static file server to the application.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `file` - The path to the file.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse} };
    ///
    /// let mut app = App::new();
    /// app.static_files("/public", "./public");
    ///
    /// ```

    pub fn static_files(&mut self, path: &'static str, file: &'static str) {
        self.static_files.insert("serve_from", file);
        self.static_files.insert("mount_path", path);
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

        async fn error_handler(err: routerify::RouteError) -> Response<Body> {
            let api_err = err.downcast::<ApiError>().unwrap_or_else(|_| {
                return Box::new(ApiError::Generic(
                    HttpResponse::new()
                        .internal_server_error()
                        .text("Unhandled error"),
                ));
            });

            match api_err.as_ref() {
                ApiError::Generic(res) => {
                    <HttpResponse as Clone>::clone(res).to_responder().unwrap()
                }
            }
        }

        for middleware in self.middlewares.iter() {
            let middleware = middleware.clone();
            router = router.middleware(routerify::Middleware::pre(move |req| {
                exec_middleware(req, middleware.clone())
            }));
        }

        if let (Some(mount_path), Some(serve_from)) = (
            self.static_files.get("mount_path"),
            self.static_files.get("serve_from"),
        ) {
            let serve_from = serve_from.to_string();
            let mount_root = mount_path.to_string();
            let mount_with_wildcard = format!("{}/*", mount_path);

            let serve_from_clone = serve_from.clone();
            let mount_root_clone = mount_root.clone();
            router = router.get(mount_with_wildcard, move |req| {
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
                                match response.to_responder() {
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
                                match response.to_responder() {
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
                                match response.to_responder() {
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
                                match response.to_responder() {
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
                                match response.to_responder() {
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
                                match response.to_responder() {
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
                                match response.to_responder() {
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
        router = router.err_handler(error_handler);

        let router = router.build().unwrap();

        cb();

        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        let service = RouterService::new(router).unwrap();

        let server = Server::bind(&addr).serve(service);

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }

    async fn serve_static_with_headers(
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

        let trimmed_path = if original_path.starts_with(mount_root.as_str()) {
            &original_path[mount_root.len()..]
        } else {
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

        parts.uri = new_path_and_query.parse().unwrap();
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
}
