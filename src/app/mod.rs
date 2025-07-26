#![warn(missing_docs)]

use crate::req::HttpRequest;
use crate::res::HttpResponse;
use crate::types::{Fut, HttpMethods, Middleware, Next, RouterFns, Routes};
use std::collections::HashMap;

pub(crate) fn box_future<F>(future: F) -> Fut
where
    F: Future<Output = HttpResponse> + Send + 'static,
{
    Box::pin(future)
}

/// The App struct is the core of Ripress, providing a simple interface for creating HTTP servers and handling requests. It follows an Express-like pattern for route handling.
pub struct App {
    routes: Routes,
    middlewares: Vec<Box<dyn Middleware>>,
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
        let path = path.into().unwrap_or("/").to_string();

        struct Wrapper<F> {
            func: F,
            path: String,
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
                    if req.path.starts_with(self.path.as_str()) {
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
                    path: self.path.clone(),
                })
            }
        }

        self.middlewares.push(Box::new(Wrapper {
            func: middleware,
            path: path,
        }));

        self
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

    pub async fn listen<F: FnOnce()>(&self, port: u16, cb: F) -> std::io::Result<()> {
        cb();
        let routes = self.routes.clone();
        let middlewares = self.middlewares.clone();

        actix_web::HttpServer::new(move || {
            let routes = routes.clone();
            let middlewares = middlewares.clone();

            routes
                .iter()
                .fold(actix_web::App::new(), move |mut app, (path, handlers)| {
                    for (method, handler) in handlers {
                        let route_method = match method {
                            HttpMethods::GET => actix_web::web::get(),
                            HttpMethods::POST => actix_web::web::post(),
                            HttpMethods::PUT => actix_web::web::put(),
                            HttpMethods::HEAD => actix_web::web::head(),
                            HttpMethods::DELETE => actix_web::web::delete(),
                            HttpMethods::PATCH => actix_web::web::patch(),
                        };

                        let handler = handler.clone();
                        let path = path.clone();
                        let middlewares = middlewares.clone();

                        app = app.route(
                            &path,
                            route_method.to(
                                move |req: actix_web::HttpRequest, payload: actix_web::web::Payload| {
                                    let handler = handler.clone();
                                    let middlewares = middlewares.clone();

                                    async move {
                                        let our_req = HttpRequest::from_actix_request(req, payload)
                                            .await
                                            .unwrap();
                                        let our_res = HttpResponse::new();

                                        if !middlewares.is_empty() {
                                            // Create a Next with our middlewares and handler
                                            let next = Next {
                                                middleware: middlewares.clone(),
                                                handler: handler.clone(),
                                            };

                                            // Run the middleware chain
                                            let response = next.run(our_req, our_res).await;
                                            response.to_responder()
                                        } else {
                                            // No middlewares, call the handler directly
                                            let response = handler(our_req, our_res).await;
                                            response.to_responder()
                                        }
                                    }
                                },
                            ),
                        );
                    }
                    app
                })
        })
        .bind(format!("127.0.0.1:{}", port))?
        .run()
        .await
    }
}
