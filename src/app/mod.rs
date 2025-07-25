use crate::req::HttpRequest;
use crate::res::HttpResponse;
use crate::types::{Fut, HttpMethod, Routes};
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
}

impl App {
    pub fn new() -> Self {
        App {
            routes: HashMap::new(),
        }
    }

    fn add_route<F, Fut>(&mut self, method: HttpMethod, path: &str, handler: F)
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
    /// use ripress_again::{app::App, context::{HttpRequest, HttpResponse} };
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
        self.add_route(HttpMethod::GET, path, handler);
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
    /// use ripress_again::{app::App, context::{HttpRequest, HttpResponse} };
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
        self.add_route(HttpMethod::POST, path, handler);
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
    /// use ripress_again::{app::App, context::{HttpRequest, HttpResponse} };
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
        self.add_route(HttpMethod::PUT, path, handler);
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
    /// use ripress_again::{app::App, context::{HttpRequest, HttpResponse} };
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
        self.add_route(HttpMethod::DELETE, path, handler);
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
    /// use ripress_again::{app::App, context::{HttpRequest, HttpResponse} };
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
        self.add_route(HttpMethod::HEAD, path, handler);
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
    /// use ripress_again::{app::App, context::{HttpRequest, HttpResponse} };
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
        self.add_route(HttpMethod::PATCH, path, handler);
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
    /// use ripress_again::{app::App, context::{HttpRequest, HttpResponse} };
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.all("/hello", handler);
    ///
    /// ```

    pub fn all<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + Clone + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethod::GET, path, handler.clone());
        self.add_route(HttpMethod::PATCH, path, handler.clone());
        self.add_route(HttpMethod::POST, path, handler.clone());
        self.add_route(HttpMethod::PUT, path, handler.clone());
        self.add_route(HttpMethod::DELETE, path, handler.clone());
        self.add_route(HttpMethod::HEAD, path, handler);
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
    /// use ripress_again::{app::App, context::{HttpRequest, HttpResponse} };
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

        actix_web::HttpServer::new(move || {
            routes
                .iter()
                .fold(actix_web::App::new(), |app, (path, (method, handler))| {
                    let route_method = match method {
                        HttpMethod::GET => actix_web::web::get(),
                        HttpMethod::POST => actix_web::web::post(),
                        HttpMethod::PUT => actix_web::web::put(),
                        HttpMethod::HEAD => actix_web::web::head(),
                        HttpMethod::DELETE => actix_web::web::delete(),
                        HttpMethod::PATCH => actix_web::web::patch(),
                    };

                    // Clone the handler to move it into the closure
                    let handler = handler.clone();
                    let path = path.clone();

                    app.route(
                        &path,
                        route_method.to(
                            move |req: actix_web::HttpRequest, payload: actix_web::web::Payload| {
                                let handler = handler.clone();
                                async move {
                                    let our_req = HttpRequest::from_actix_request(req, payload)
                                        .await
                                        .unwrap();
                                    let our_res = HttpResponse::new();
                                    let response = handler(our_req, our_res).await;
                                    response.to_responder()
                                }
                            },
                        ),
                    )
                })
        })
        .bind(format!("127.0.0.1:{}", port))?
        .run()
        .await
    }
}
