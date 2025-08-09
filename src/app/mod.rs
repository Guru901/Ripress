#![warn(missing_docs)]

use crate::helpers::exec_middleware;
use crate::req::HttpRequest;
use crate::res::HttpResponse;
use crate::types::{Fut, FutMiddleware, HttpMethods, RouterFns, Routes};
use hyper::{Body, Error, Request, Response, Server};
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
    middlewares: Vec<Box<Middleware>>,
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

        self.middlewares.push(Box::new(Middleware {
            func: Arc::new(move |req, res| -> crate::types::FutMiddleware {
                box_future_middleware(middleware(req, res))
            }),
            path: path,
        }));

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
            let api_err = err.downcast::<ApiError>().unwrap();

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
            router = router.get(*mount_path, move |req| {
                let serve_from = serve_from.clone();
                async move {
                    match Self::serve_static_with_headers(req, serve_from).await {
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
                                let mut our_req =
                                    HttpRequest::from_hyper_request(&mut req).await.unwrap();
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });

                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                Ok(response.to_responder().unwrap())
                            }
                        });
                    }
                    HttpMethods::POST => {
                        router = router.post(path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req =
                                    HttpRequest::from_hyper_request(&mut req).await.unwrap();
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });

                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                Ok(response.to_responder().unwrap())
                            }
                        });
                    }
                    HttpMethods::PUT => {
                        router = router.put(path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req =
                                    HttpRequest::from_hyper_request(&mut req).await.unwrap();
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                Ok(response.to_responder().unwrap())
                            }
                        });
                    }
                    HttpMethods::DELETE => {
                        router = router.delete(path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req =
                                    HttpRequest::from_hyper_request(&mut req).await.unwrap();
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                Ok(response.to_responder().unwrap())
                            }
                        });
                    }
                    HttpMethods::PATCH => {
                        router = router.patch(path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req =
                                    HttpRequest::from_hyper_request(&mut req).await.unwrap();
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                Ok(response.to_responder().unwrap())
                            }
                        });
                    }
                    HttpMethods::HEAD => {
                        router = router.head(path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req =
                                    HttpRequest::from_hyper_request(&mut req).await.unwrap();
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                Ok(response.to_responder().unwrap())
                            }
                        });
                    }
                    HttpMethods::OPTIONS => {
                        router = router.options(path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req =
                                    HttpRequest::from_hyper_request(&mut req).await.unwrap();
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(our_req, our_res).await;
                                Ok(response.to_responder().unwrap())
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
        path: String,
    ) -> Result<Response<Body>, std::io::Error> {
        let static_service = Static::new(Path::new(path.as_str()));

        match static_service.serve(req).await {
            Ok(mut response) => {
                // Add custom headers
                response
                    .headers_mut()
                    .insert("Cache-Control", "public, max-age=86400".parse().unwrap());
                response
                    .headers_mut()
                    .insert("X-Served-By", "hyper-staticfile".parse().unwrap());
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }
}
