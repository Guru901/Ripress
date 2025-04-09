use crate::helpers::exec_middleware;
use crate::request::HttpRequest;
use crate::response::HttpResponse;
use crate::types::{ApiError, Fut, FutMiddleware, Handler, HttpMethods, Routes};
use hyper::{Body, Response, Server};
use routerify::ext::RequestExt;
use routerify::{Router, RouterService};
use std::net::SocketAddr;
use std::{collections::HashMap, future::Future, sync::Arc};

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

pub struct App {
    routes: Routes,
    middlewares: Vec<Box<Middleware>>,
    pub(crate) static_files: HashMap<String, String>,
}

impl App {
    pub fn new() -> App {
        App {
            routes: HashMap::new(),
            middlewares: Vec::new(),
            static_files: HashMap::new(),
        }
    }

    pub fn clone_app(&self) -> App {
        App {
            routes: self.routes.clone(),
            middlewares: self.middlewares.clone(),
            static_files: self.static_files.clone(),
        }
    }

    /// Add a GET route to the application.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse} };
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.get("/hello", handler);
    /// ```

    pub fn get<F, Fut>(&mut self, path: &'static str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req: &mut HttpRequest, res: HttpResponse| {
            box_future(handler(req.clone(), res))
        });
        self.add_route(HttpMethods::GET, path, wrapped_handler);
    }

    /// Add a POST route to the application.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse} };
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.post("/hello", handler);
    /// ```

    pub fn post<F, Fut>(&mut self, path: &'static str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req: &mut HttpRequest, res: HttpResponse| {
            box_future(handler(req.clone(), res))
        });
        self.add_route(HttpMethods::POST, path, wrapped_handler);
    }

    /// Add a PUT route to the application.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse} };
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.put("/hello", handler);
    /// ```

    pub fn put<F, Fut>(&mut self, path: &'static str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req: &mut HttpRequest, res: HttpResponse| {
            box_future(handler(req.clone(), res))
        });
        self.add_route(HttpMethods::PUT, path, wrapped_handler);
    }

    /// Add a DELETE route to the application.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse} };
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.delete("/hello", handler);
    /// ```

    pub fn delete<F, Fut>(&mut self, path: &'static str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req: &mut HttpRequest, res: HttpResponse| {
            box_future(handler(req.clone(), res))
        });
        self.add_route(HttpMethods::DELETE, path, wrapped_handler);
    }

    /// Add a PATCH route to the application.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse} };
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.patch("/hello", handler);
    /// ```

    pub fn patch<F, Fut>(&mut self, path: &'static str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req: &mut HttpRequest, res: HttpResponse| {
            box_future(handler(req.clone(), res))
        });
        self.add_route(HttpMethods::PATCH, path, wrapped_handler);
    }

    /// Add a HEAD route to the application.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse} };
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.head("/hello", handler);
    /// ```

    pub fn head<F, Fut>(&mut self, path: &'static str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req: &mut HttpRequest, res: HttpResponse| {
            box_future(handler(req.clone(), res))
        });
        self.add_route(HttpMethods::HEAD, path, wrapped_handler);
    }

    /// Add a route to the application that matches all HTTP methods.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// ## Example
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse} };
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.all("/hello", handler);
    ///
    /// ```

    pub fn all<F, Fut>(&mut self, path: &'static str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req: &mut HttpRequest, res: HttpResponse| {
            box_future(handler(req.clone(), res))
        });
        self.add_route(HttpMethods::GET, path, wrapped_handler.clone());
        self.add_route(HttpMethods::POST, path, wrapped_handler.clone());
        self.add_route(HttpMethods::PUT, path, wrapped_handler.clone());
        self.add_route(HttpMethods::DELETE, path, wrapped_handler.clone());
        self.add_route(HttpMethods::PATCH, path, wrapped_handler.clone());
        self.add_route(HttpMethods::HEAD, path, wrapped_handler.clone());
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
        self.static_files
            .insert("serve_from".to_string(), file.to_string());

        self.static_files
            .insert("mount_path".to_string(), path.to_string());
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

    pub async fn listen<F: FnOnce()>(self, port: u16, cb: F) {
        let mut router = Router::<Body, ApiError>::builder();

        async fn error_handler(err: routerify::RouteError) -> Response<Body> {
            let api_err = err.downcast::<ApiError>().unwrap();

            match api_err.as_ref() {
                ApiError::Generic(msg, status_code) => Response::builder()
                    .status(*status_code)
                    .body(Body::from(msg.to_string()))
                    .unwrap(),
            }
        }

        for middleware in self.middlewares {
            let middleware = middleware.clone();
            router = router.middleware(routerify::Middleware::pre(move |req| {
                exec_middleware(req, middleware.clone())
            }));
        }

        for (path, methods) in &self.routes {
            for (method, handler) in methods {
                let handler = handler.clone();
                match method {
                    HttpMethods::GET => {
                        router = router.get(*path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req =
                                    HttpRequest::from_hyper_request(&mut req).await.unwrap();
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });

                                let our_res = HttpResponse::new();
                                let response = handler(&mut our_req, our_res).await;
                                Ok(response.to_responder().unwrap())
                            }
                        });
                    }
                    HttpMethods::POST => {
                        router = router.post(*path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req =
                                    HttpRequest::from_hyper_request(&mut req).await.unwrap();
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });

                                let our_res = HttpResponse::new();
                                let response = handler(&mut our_req, our_res).await;
                                Ok(response.to_responder().unwrap())
                            }
                        });
                    }
                    HttpMethods::PUT => {
                        router = router.put(*path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req =
                                    HttpRequest::from_hyper_request(&mut req).await.unwrap();
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(&mut our_req, our_res).await;
                                Ok(response.to_responder().unwrap())
                            }
                        });
                    }
                    HttpMethods::DELETE => {
                        router = router.delete(*path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req =
                                    HttpRequest::from_hyper_request(&mut req).await.unwrap();
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(&mut our_req, our_res).await;
                                Ok(response.to_responder().unwrap())
                            }
                        });
                    }
                    HttpMethods::PATCH => {
                        router = router.patch(*path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req =
                                    HttpRequest::from_hyper_request(&mut req).await.unwrap();
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(&mut our_req, our_res).await;
                                Ok(response.to_responder().unwrap())
                            }
                        });
                    }
                    HttpMethods::HEAD => {
                        router = router.head(*path, move |mut req| {
                            let handler = handler.clone();
                            async move {
                                let mut our_req =
                                    HttpRequest::from_hyper_request(&mut req).await.unwrap();
                                req.params().iter().for_each(|(key, value)| {
                                    our_req.set_param(key, value);
                                });
                                let our_res = HttpResponse::new();
                                let response = handler(&mut our_req, our_res).await;
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

    /// Adds a route to the application.
    ///
    /// ## Arguments
    ///
    /// * `method` - The HTTP method (GET, POST, PUT, DELETE) of the route.
    /// * `path` - The path of the route.
    /// * `handler` - The handler function for the route.
    ///
    pub(crate) fn add_route(&mut self, method: HttpMethods, path: &'static str, handler: Handler) {
        let path_handlers = self.routes.entry(path).or_insert_with(HashMap::new);
        path_handlers.insert(method, handler);
    }
}

#[cfg(test)]
impl App {
    pub(crate) fn get_routes(&self, path: &str, method: HttpMethods) -> Option<&Handler> {
        Some(self.routes.get(path).unwrap().get(&method).unwrap())
    }

    pub(crate) fn get_middlewares(&self) -> &Vec<Box<Middleware>> {
        &self.middlewares
    }
}
