use crate::request::HttpRequest;
use crate::response::HttpResponse;
use crate::types::{Fut, Handler, HttpMethods, Middleware, Next, Routes};
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
}

impl App {
    pub fn new() -> App {
        return App {
            routes: HashMap::new(),
            middlewares: Vec::new(),
        };
    }

    pub fn clone_app(&self) -> App {
        // Create a new vector and clone each middleware box
        let mut cloned_middlewares = Vec::new();

        // Since we can't directly clone Box<dyn Middleware>, we need to use a different approach
        // We'll create a new empty vector and manually clone each middleware
        for middleware in &self.middlewares {
            cloned_middlewares.push(middleware.clone());
        }

        App {
            routes: self.routes.clone(),
            middlewares: cloned_middlewares,
        }
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

    /// Add a PATCH route to the application.
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
    ///     app.listen(3000, || {println!("server running on port 3000")}).await;
    /// }
    ///
    /// ```

    pub fn use_middleware<M: Middleware>(&mut self, middleware: M) -> &mut Self {
        self.middlewares.push(Box::new(middleware));
        self
    }

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

                    // If we have middlewares, run the request through them
                    if !middlewares_clone.is_empty() {
                      // Create a Next with our middlewares and handler
                      let next = Next {
                        middleware: &middlewares_clone,
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
    pub(crate) fn get_routes(&self, path: &str, method: HttpMethods) -> Option<&Handler> {
        Some(self.routes.get(path).unwrap().get(&method).unwrap())
    }
}
