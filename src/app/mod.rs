#![warn(missing_docs)]

use crate::req::HttpRequest;
use crate::res::HttpResponse;
use crate::types::{Fut, HttpMethods, Middleware, Next, RouterFns, Routes};
use hyper::{Body, Response, Server};
use routerify::ext::RequestExt;
use routerify::{Router, RouterService};
use std::collections::HashMap;
use std::net::SocketAddr;

pub(crate) fn box_future<F>(future: F) -> Fut
where
    F: Future<Output = HttpResponse> + Send + 'static,
{
    Box::pin(future)
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
    middlewares: Vec<Box<dyn Middleware>>,
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
    /// app.use_middleware("path", |req, res, next| {
    ///     println!("here");
    ///     Box::pin(async move { next.run(req, res).await })
    /// });
    ///
    /// ```

    pub fn use_middleware<F, Fut, P>(&mut self, path: P, middleware: F) -> &mut Self
    where
        P: Into<Option<&'static str>>,
        F: Fn(HttpRequest, HttpResponse, Next) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = HttpResponse> + Send + 'static,
    {
        let path = path.into().unwrap_or("/");

        struct Wrapper<F> {
            func: F,
            path: &'static str,
        }

        impl<F, Fut> Middleware for Wrapper<F>
        where
            F: Fn(HttpRequest, HttpResponse, Next) -> Fut + Send + Sync + Clone + 'static,
            Fut: std::future::Future<Output = HttpResponse> + Send + 'static,
        {
            fn handle(
                &self,
                req: HttpRequest,
                res: HttpResponse,
                next: Next,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = HttpResponse> + Send + 'static>>
            {
                if self.path == "/" {
                    let fut = (self.func)(req, res, next);
                    Box::pin(fut)
                } else {
                    if req.path.starts_with(self.path) {
                        let fut = (self.func)(req, res, next);
                        Box::pin(fut)
                    } else {
                        Box::pin(async move { next.run(req, res).await })
                    }
                }
            }

            fn clone_box(&self) -> Box<dyn Middleware> {
                Box::new(Wrapper {
                    func: self.func.clone(),
                    path: self.path,
                })
            }
        }

        self.middlewares.push(Box::new(Wrapper {
            func: middleware,
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
    ///     app.listen(3000, || {println!("server running on port 3000")}).await.unwrap();
    /// }
    ///
    /// ```

    // pub async fn listen<F: FnOnce()>(&self, port: u16, cb: F) -> std::io::Result<()> {
    //     cb();
    //     let routes = self.routes.clone();
    //     let middlewares = self.middlewares.clone();
    //     let static_files = self.static_files.clone();

    //     actix_web::HttpServer::new(move || {
    //         let routes = routes.clone();
    //         let middlewares = middlewares.clone();
    //         let static_files = static_files.clone();

    //         let mut app =
    //             routes
    //                 .iter()
    //                 .fold(actix_web::App::new(), move |mut app, (path, handlers)| {
    //                     for (method, handler) in handlers {
    //                         let route_method = match method {
    //                             HttpMethods::GET => actix_web::web::get(),
    //                             HttpMethods::POST => actix_web::web::post(),
    //                             HttpMethods::PUT => actix_web::web::put(),
    //                             HttpMethods::HEAD => actix_web::web::head(),
    //                             HttpMethods::DELETE => actix_web::web::delete(),
    //                             HttpMethods::PATCH => actix_web::web::patch(),
    //                         };

    //                         let handler = handler.clone();
    //                         let path = path.clone();
    //                         let middlewares = middlewares.clone();

    //                         app = app.route(
    //                     &path,
    //                     route_method.to(
    //                         move |req: actix_web::HttpRequest, payload: actix_web::web::Payload| {
    //                             let handler = handler.clone();
    //                             let middlewares = middlewares.clone();

    //                             async move {
    //                                 let our_req = HttpRequest::from_actix_request(req, payload)
    //                                     .await
    //                                     .unwrap();
    //                                 let our_res = HttpResponse::new();

    //                                 if !middlewares.is_empty() {
    //                                     // Create a Next with our middlewares and handler
    //                                     let next = Next {
    //                                         middleware: middlewares.clone(),
    //                                         handler: handler.clone(),
    //                                     };

    //                                     // Run the middleware chain
    //                                     let response = next.run(our_req, our_res).await;
    //                                     response.to_responder()
    //                                 } else {
    //                                     // No middlewares, call the handler directly
    //                                     let response = handler(our_req, our_res).await;
    //                                     response.to_responder()
    //                                 }
    //                             }
    //                         },
    //                     ),
    //                 );
    //                     }
    //                     app
    //                 });

    //         // Add static files service if configured
    //         if let (Some(mount_path), Some(serve_from)) = (
    //             static_files.get("mount_path"),
    //             static_files.get("serve_from"),
    //         ) {
    //             app = app
    //                 .service(actix_files::Files::new(mount_path, serve_from).show_files_listing());
    //         }

    //         app
    //     })
    //     .bind(format!("127.0.0.1:{}", port))?
    //     .run()
    //     .await
    // }

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
}
