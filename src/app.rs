use std::{collections::HashMap, future::Future, sync::Arc};

use crate::request::HttpRequest;
use crate::response::HttpResponse;
use crate::types::{Fut, Handler, HttpMethods, Routes};

fn box_future<F>(future: F) -> Fut
where
    F: Future<Output = HttpResponse> + Send + 'static,
{
    Box::pin(future)
}

pub struct App {
    routes: Routes,
}

impl Clone for App {
    fn clone(&self) -> Self {
        App {
            routes: self.routes.clone(),
        }
    }
}

impl App {
    pub fn new() -> App {
        return App {
            routes: HashMap::new(),
        };
    }

    /// Add a GET route to the application.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// # Example
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
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
        self.add_route(HttpMethods::GET, path, wrapped_handler);
    }

    /// Add a POST route to the application.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// # Example
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
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
        self.add_route(HttpMethods::POST, path, wrapped_handler);
    }

    /// Add a PUT route to the application.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// # Example
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
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
        self.add_route(HttpMethods::PUT, path, wrapped_handler);
    }

    /// Add a DELETE route to the application.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// # Example
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
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
        self.add_route(HttpMethods::DELETE, path, wrapped_handler);
    }

    /// Starts the server and listens on the specified address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to listen on e.g. "127.0.0.1:3000".
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse} };
    /// use tokio;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut app = App::new();
    ///     app.listen("127.0.0.1:3000").await;
    /// }
    ///
    /// ```

    pub async fn listen(self, addr: &str) {
        println!("Server listening on {}", addr);

        actix_web::HttpServer::new(move || {
            let mut app = actix_web::App::new();

            for (path, methods) in self.routes.clone() {
                for (method, handler) in methods {
                    let method = method.clone();

                    match method {
                        HttpMethods::GET => {
                            app = app.route(
                                &path,
                                actix_web::web::get().to(
                                     move |req: actix_web::HttpRequest,  payload: actix_web::web::Payload|  {
                                        let handler_clone = handler.clone();
                                        async move {
                                            let our_req = HttpRequest::from_actix_request(req, payload).await.unwrap();
                                            let our_res = HttpResponse::new();
                                            let future = handler_clone(our_req, our_res);
                                            let response = future.await;
                                            response.to_responder()
                                        }
                                    },
                                ),
                            );
                        }
                        HttpMethods::POST => {
                            app = app.route(
                                &path,
                                actix_web::web::post().to( move |req: actix_web::HttpRequest,  payload: actix_web::web::Payload| {
                                        let handler_clone = handler.clone();
                                        async move {
                                        let our_req = HttpRequest::from_actix_request(req,payload).await.unwrap();
                                        let our_res = HttpResponse::new();
                                        let future = handler_clone(our_req, our_res);
                                            let response = future.await;
                                            response.to_responder()
                                        }
                                }),
                            );
                        }
                        HttpMethods::PUT => {
                            app = app.route(
                                &path,
                                actix_web::web::post().to(move |req: actix_web::HttpRequest,  payload: actix_web::web::Payload| {
                                    let handler_clone = handler.clone();
                                    async move {

                                    let our_req = HttpRequest::from_actix_request(req, payload).await.unwrap();
                                    let our_res = HttpResponse::new();
                                    let future = handler_clone(our_req, our_res);
                                        let response = future.await;
                                        response.to_responder()
                                    }
                                }),
                            );
                        }
                        HttpMethods::DELETE => {
                            app = app.route(
                                &path,
                                actix_web::web::post().to(move |req: actix_web::HttpRequest,  payload: actix_web::web::Payload| {
                                    let handler_clone = handler.clone();
                                    async move {
                                    let our_req = HttpRequest::from_actix_request(req,payload).await.unwrap();
                                    let our_res = HttpResponse::new();
                                    let future = handler_clone(our_req, our_res);
                                        let response = future.await;
                                        response.to_responder()
                                    }
                                }),
                            );
                        }
                    }
                }
            }
            app
        })
        .bind(addr)
        .unwrap()
        .run()
        .await
        .unwrap();
    }

    /// Adds a route to the application.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method (GET, POST, PUT, DELETE) of the route.
    /// * `path` - The path of the route.
    /// * `handler` - The handler function for the route.
    ///
    fn add_route(&mut self, method: HttpMethods, path: &'static str, handler: Handler) {
        let path_handlers = self.routes.entry(path).or_insert_with(HashMap::new);
        path_handlers.insert(method, handler);
    }
}
#[cfg(test)]
impl App {
    pub fn get_routes(&self) -> &Routes {
        &self.routes
    }
}
