use crate::request::HttpRequest;
use crate::response::HttpResponse;
use crate::types::{Fut, Handler, HttpMethods, Middleware, Next, Routes};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::{Error, FromRequest};
use std::{collections::HashMap, future::Future, sync::Arc};

pub(crate) fn box_future<F>(future: F) -> Fut
where
    F: Future<Output = HttpResponse> + Send + 'static,
{
    Box::pin(future)
}

pub struct App {
    routes: Routes,
    middlewares: Vec<Middleware>,
}

impl Clone for App {
    fn clone(&self) -> Self {
        App {
            routes: self.routes.clone(),
            middlewares: self.middlewares.clone(),
        }
    }
}

impl App {
    pub fn new() -> App {
        return App {
            routes: HashMap::new(),
            middlewares: Vec::new(),
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

    pub fn r#use<F, Fut>(&mut self, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler = Arc::new(move |req, res, next| box_future(handler(req, res, next)));
        self.middlewares.push(wrapped_handler);
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

      for middleware in self.middlewares.clone() {
        let middleware = middleware.clone();
        app.wrap_fn(move |req, srv| {
            let middleware = middleware.clone();

            async  move{
              middleware_wrapper(req, srv, middleware).await
            }
        });
      }

      for (path, methods) in self.routes.clone() {
        for (method, handler) in methods {
          let method = method.clone();

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

fn middleware_wrapper<S, Fut>(
    req: ServiceRequest,
    srv: S,
    function: Arc<dyn Fn(HttpRequest, HttpResponse, Next) -> Fut + Send + Sync>,
) -> impl Future<Output = Result<ServiceResponse, Error>> + 'static
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    Fut: Future<Output = HttpResponse> + Send + 'static,
{
    let function = function.clone();

    async move {
        let (actix_req, mut payload) = req.into_parts();
        let web_payload = actix_web::web::Payload::from_request(&actix_req.clone(), &mut payload)
            .await
            .unwrap();

        let our_req = HttpRequest::from_actix_request(actix_req.clone(), web_payload)
            .await
            .unwrap();

        // Create your custom response
        let our_res = HttpResponse::new();

        // Create your custom Next handler
        let next = Next::new(|req: HttpRequest| {
            // Convert back to actix request and call the next service
            let actix_req_clone = actix_req.clone();

            Box::pin(async {
                let reconstructed_req = ServiceRequest::from_request(actix_req_clone);

                match srv.call(reconstructed_req).await {
                    Ok(res) => {
                        // Convert actix response to your custom HttpResponse
                        let mut our_res = HttpResponse::new();
                        // Map the response... actual implementation depends on your HttpResponse
                        our_res
                    }
                    Err(_) => HttpResponse::new()
                        .internal_server_error()
                        .text("Internal server error"),
                }
            });
        });

        // Call your middleware function
        let response = function(our_req, our_res, next).await;

        // Convert your custom response back to actix response
        Ok(ServiceResponse::new(actix_req, response.to_responder()))
    }
}

#[cfg(test)]
impl App {
    pub(crate) fn get_routes(&self, path: &str, method: HttpMethods) -> Option<&Handler> {
        Some(self.routes.get(path).unwrap().get(&method).unwrap())
    }
}
