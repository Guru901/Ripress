use crate::request::HttpRequest;
use crate::response::HttpResponse;
use crate::types::{Fut, Handler, HttpMethods, Middleware, Next, Routes};
use actix_files as fs;
use std::{collections::HashMap, future::Future, sync::Arc};

pub(crate) fn box_future<F>(future: F) -> Fut
where
    F: Future<Output = HttpResponse> + Send + 'static,
{
    Box::pin(future)
}

pub struct App {
    routes: Routes,
    middlewares: Vec<Box<dyn Middleware>>,
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
        // Create a new vector and clone each middleware box
        let mut cloned_middlewares = Vec::new();

        // Clone each middleware using clone_box
        for middleware in &self.middlewares {
            cloned_middlewares.push(middleware.clone_box());
        }

        App {
            routes: self.routes.clone(),
            middlewares: cloned_middlewares,
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
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
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
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
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
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
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
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
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
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
        self.add_route(HttpMethods::PATCH, path, wrapped_handler);
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
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res)));
        self.add_route(HttpMethods::GET, path, wrapped_handler.clone());
        self.add_route(HttpMethods::POST, path, wrapped_handler.clone());
        self.add_route(HttpMethods::PUT, path, wrapped_handler.clone());
        self.add_route(HttpMethods::DELETE, path, wrapped_handler.clone());
        self.add_route(HttpMethods::PATCH, path, wrapped_handler.clone());
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
            fn clone_box(&self) -> Box<dyn Middleware> {
                Box::new(Wrapper {
                    func: self.func.clone(),
                    path: self.path.clone(),
                })
            }

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
                    if req.get_path().starts_with(self.path.as_str()) {
                        let fut = (self.func)(req, res, next);
                        Box::pin(fut)
                    } else {
                        Box::pin(async move { next.run(req, res).await })
                    }
                }
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

    pub async fn listen<F: FnOnce()>(self, port: i32, cb: F) {
        cb();
        let app_clone = self.clone_app();
        actix_web::HttpServer::new(move || {
            let mut app = actix_web::App::new();

      for (path, methods) in app_clone.routes.clone() {
        for (method, handler) in methods {
          let method = method.clone();
          let middlewares = app_clone.middlewares.clone();

          match method {
            HttpMethods::GET | HttpMethods::POST | HttpMethods::PUT | HttpMethods::DELETE | HttpMethods::PATCH => {
              let route_method = match method {
                HttpMethods::GET => actix_web::web::get(),
                HttpMethods::POST => actix_web::web::post(),
                HttpMethods::PUT => actix_web::web::put(),
                HttpMethods::DELETE => actix_web::web::delete(),
                HttpMethods::PATCH => actix_web::web::patch(),
              };

              app = app.route(
                &path,
                route_method.to(move |req: actix_web::HttpRequest, payload: actix_web::web::Payload| {
                  let handler_clone = handler.clone();
                  let middlewares_clone = middlewares.clone();

                  async move {
                    let our_req = HttpRequest::from_actix_request(req, payload).await.unwrap();
                    let our_res = HttpResponse::new();
                    let middleware_clone = middlewares_clone.clone();

                    // If we have middlewares, run the request through them
                    if !middlewares_clone.is_empty() {
                      // Create a Next with our middlewares and handler
                      let next = Next {
                        middleware: middleware_clone,
                        handler: handler_clone.clone(),
                      };

                      // Run the middleware chain
                      let response = next.run(our_req, our_res).await;
                      response.to_responder()
                    } else {
                      // No middlewares, just call the handler directly
                      let future = handler_clone(our_req, our_res);
                      let response = future.await;
                      response.to_responder()
                    }
                  }
                }),
              );
            }
          }
        }
      }

      if self.static_files.len() > 0 {
        let static_files = self.static_files.clone();
        app = app.service(fs::Files::new(static_files.get("mount_path").unwrap(), self.static_files.get("serve_from").unwrap()).show_files_listing());
      }

     app
    })
      .bind(format!("127.0.0.1:{port}"))
      .unwrap()
      .run()
      .await
      .unwrap();
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

    pub(crate) fn get_middlewares(&self) -> &Vec<Box<dyn Middleware>> {
        &self.middlewares
    }
}
