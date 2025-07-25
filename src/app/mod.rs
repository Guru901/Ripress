use crate::req::HttpRequest;
use crate::res::HttpResponse;
use crate::types::{Fut, Handler, HttpMethods, Middleware, Next, Routes};
use std::collections::HashMap;

pub(crate) fn box_future<F>(future: F) -> Fut
where
    F: Future<Output = HttpResponse> + Send + 'static,
{
    Box::pin(future)
}
use std::sync::Arc;

pub struct App {
    routes: Routes,
    middlewares: Vec<Box<dyn Middleware>>,
}

impl App {
    pub fn new() -> Self {
        App {
            routes: HashMap::new(),
            middlewares: Vec::new(),
        }
    }

    fn add_route<F, Fut>(&mut self, method: HttpMethods, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
        self.routes
            .insert(path.to_string(), (method, wrapped_handler));
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

    pub fn get<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethods::GET, path, handler);
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

    pub fn post<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethods::POST, path, handler);
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

    pub fn put<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethods::PUT, path, handler);
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

    pub fn delete<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethods::DELETE, path, handler);
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

    pub fn head<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethods::HEAD, path, handler);
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

    pub fn patch<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethods::PATCH, path, handler);
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

            routes.iter().fold(
                actix_web::App::new(),
                move |app, (path, (method, handler))| {
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

                    app.route(
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
                                        // No middlewares, just call the handler directly
                                        let response = handler(our_req, our_res).await;
                                        response.to_responder()
                                    }
                                }
                            },
                        ),
                    )
                },
            )
        })
        .bind(format!("127.0.0.1:{}", port))?
        .run()
        .await
    }
}

#[cfg(test)]
impl App {
    pub(crate) fn get_routes(&self, path: &str, method: HttpMethods) -> Option<&Handler> {
        if self.routes.get(path).unwrap().0 == method {
            Some(&self.routes.get(path).unwrap().1)
        } else {
            None
        }
    }
}
