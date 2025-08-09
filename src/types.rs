use crate::app::box_future;
use crate::req::HttpRequest;
use crate::res::HttpResponse;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RequestBody {
    pub content: RequestBodyContent,
    pub content_type: RequestBodyType,
}

impl RequestBody {
    pub fn new_text(text: String) -> Self {
        RequestBody {
            content_type: RequestBodyType::TEXT,
            content: RequestBodyContent::TEXT(Box::leak(text.into_boxed_str())),
        }
    }

    pub fn new_form(form_data: String) -> Self {
        RequestBody {
            content_type: RequestBodyType::FORM,
            content: RequestBodyContent::FORM(Box::leak(form_data.into_boxed_str())),
        }
    }

    pub fn new_json<T: Into<serde_json::Value>>(json: T) -> Self {
        RequestBody {
            content_type: RequestBodyType::JSON,
            content: RequestBodyContent::JSON(json.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestBodyType {
    JSON,
    TEXT,
    FORM,
    EMPTY,
}

impl Copy for RequestBodyType {}

#[derive(Debug, Clone)]
pub enum RequestBodyContent {
    TEXT(&'static str),
    JSON(serde_json::Value),
    FORM(&'static str),
    EMPTY,
}

#[derive(Debug, Clone)]
pub enum ResponseContentBody {
    TEXT(String),
    HTML(String),
    JSON(serde_json::Value),
}

impl ResponseContentBody {
    pub fn new_text<T: Into<String>>(text: T) -> Self {
        ResponseContentBody::TEXT(text.into())
    }

    pub fn new_json<T: Serialize>(json: T) -> Self {
        let value = serde_json::to_value(json).expect("Failed to serialize to JSON");
        ResponseContentBody::JSON(value)
    }
    pub fn new_html<T: Into<String>>(html: T) -> Self {
        ResponseContentBody::HTML(html.into())
    }
}
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ResponseContentType {
    TEXT,
    JSON,
    HTML,
}

pub type Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>;

pub type Handler = Arc<dyn Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static>;

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum HttpMethods {
    GET,
    POST,
    PUT,
    HEAD,
    DELETE,
    PATCH,
    OPTIONS,
}

impl Display for HttpMethods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let method = match self {
            HttpMethods::GET => "GET",
            HttpMethods::PUT => "PUT",
            HttpMethods::POST => "POST",
            HttpMethods::DELETE => "DELETE",
            HttpMethods::PATCH => "PATCH",
            HttpMethods::HEAD => "HEAD",
            HttpMethods::OPTIONS => "OPTION",
        };
        write!(f, "{}", method)
    }
}

pub type Routes = HashMap<String, HashMap<HttpMethods, Handler>>;

#[derive(Debug, PartialEq)]
pub enum HttpRequestError {
    MissingCookie(String),
    MissingParam(String),
    MissingHeader(String),
    MissingQuery(String),
}

impl std::fmt::Display for HttpRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpRequestError::MissingCookie(cookie) => write!(f, "Cookie {} doesn't exist", cookie),
            HttpRequestError::MissingParam(param) => write!(f, "Param {} doesn't exist", param),
            HttpRequestError::MissingHeader(header) => write!(f, "Header {} doesn't exist", header),
            HttpRequestError::MissingQuery(query) => write!(f, "Query {} doesn't exist", query),
        }
    }
}

#[derive(Debug)]
pub enum HttpResponseError {
    MissingHeader(String),
}

impl std::fmt::Display for HttpResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpResponseError::MissingHeader(header) => write!(f, "Header {} doesnt exist", header),
        }
    }
}
pub type FutMiddleware =
    Pin<Box<dyn Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static>>;
pub type HandlerMiddleware =
    Arc<dyn Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static>;

pub trait Middleware: Send + Sync + 'static {
    fn handle(
        &self,
        req: &HttpRequest,
        res: HttpResponse,
        next: Next,
    ) -> Pin<Box<dyn Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static>>;

    // Add this method to allow cloning of Box<dyn Middleware>
    fn clone_box(&self) -> Box<dyn Middleware>;
}

// Implement Clone for Box<dyn Middleware>
impl Clone for Box<dyn Middleware> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub struct Next {
    pub middleware: Vec<Box<dyn Middleware>>,
    pub handler: HandlerMiddleware,
}

impl Next {
    pub fn new() -> Self {
        Next {
            middleware: Vec::new(),
            handler: Arc::new(|_, _| Box::pin(async { (HttpRequest::new(), None) })),
        }
    }
    pub async fn run(
        self,
        req: HttpRequest,
        res: HttpResponse,
    ) -> (HttpRequest, Option<HttpResponse>) {
        if let Some((current, rest)) = self.middleware.split_first() {
            // Call the next middleware
            let next = Next {
                middleware: rest.to_vec(),
                handler: self.handler.clone(),
            };
            current.handle(&req, res, next).await
        } else {
            // No more middleware, call the handler
            (self.handler)(req, res).await
        }
    }
}

pub trait RouterFns {
    fn routes(&mut self) -> &mut Routes;

    fn add_route<F, Fut>(&mut self, method: HttpMethods, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let routes = self.routes();
        if routes.contains_key(path) {
            let wrapped_handler =
                Arc::new(move |req, res| box_future(handler(req, res))) as Handler;

            if let Some(route_map) = routes.get_mut(path) {
                route_map.insert(method, wrapped_handler);
            }
        } else {
            let wrapped_handler =
                Arc::new(move |req, res| box_future(handler(req, res))) as Handler;
            routes.insert(path.to_string(), {
                let mut map = HashMap::new();
                map.insert(method, wrapped_handler);
                map
            });
        }
    }

    /// Add a GET route to the application or router.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// ## Example (App)
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse}};
    /// use ripress::types::RouterFns;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.get("/hello", handler);
    /// ```
    ///
    /// ## Example (Router)
    ///
    /// ```
    /// use ripress::{router::Router, context::{HttpRequest, HttpResponse}};
    /// use ripress::types::RouterFns;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut router = Router::new("/api");
    /// router.get("/hello", handler);
    /// ```

    fn get<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethods::GET, path, handler);
    }

    /// Add a POST route to the application or router.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// ## Example (App)
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse}};
    /// use ripress::types::RouterFns;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.post("/hello", handler);
    /// ```
    ///
    /// ## Example (Router)
    ///
    /// ```
    /// use ripress::{router::Router, context::{HttpRequest, HttpResponse}};
    /// use ripress::types::RouterFns;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut router = Router::new("/api");
    /// router.post("/hello", handler);
    /// ```

    fn post<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethods::POST, path, handler);
    }

    /// Add a PUT route to the application or router.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// ## Example (App)
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse}};
    /// use ripress::types::RouterFns;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.put("/hello", handler);
    /// ```
    ///
    /// ## Example (Router)
    ///
    /// ```
    /// use ripress::{router::Router, context::{HttpRequest, HttpResponse}};
    /// use ripress::types::RouterFns;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut router = Router::new("/api");
    /// router.put("/hello", handler);
    /// ```

    fn put<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethods::PUT, path, handler);
    }

    /// Add a DELETE route to the application or router.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// ## Example (App)
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse}};
    /// use ripress::types::RouterFns;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.delete("/hello", handler);
    /// ```
    ///
    /// ## Example (Router)
    ///
    /// ```
    /// use ripress::{router::Router, context::{HttpRequest, HttpResponse}};
    /// use ripress::types::RouterFns;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut router = Router::new("/api");
    /// router.delete("/hello", handler);
    /// ```

    fn delete<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethods::DELETE, path, handler);
    }

    /// Add a HEAD route to the application or router.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// ## Example (App)
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse}};
    /// use ripress::types::RouterFns;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.head("/hello", handler);
    /// ```
    ///
    /// ## Example (Router)
    ///
    /// ```
    /// use ripress::{router::Router, context::{HttpRequest, HttpResponse}};
    /// use ripress::types::RouterFns;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut router = Router::new("/api");
    /// router.head("/hello", handler);
    /// ```

    fn head<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethods::HEAD, path, handler);
    }

    /// Add a PATCH route to the application or router.
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// ## Example (App)
    ///
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse}};
    /// use ripress::types::RouterFns;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.patch("/hello", handler);
    /// ```
    ///
    /// ## Example (Router)
    ///
    /// ```
    /// use ripress::{router::Router, context::{HttpRequest, HttpResponse}};
    /// use ripress::types::RouterFns;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut router = Router::new("/api");
    /// router.patch("/hello", handler);
    /// ```

    fn patch<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.add_route(HttpMethods::PATCH, path, handler);
    }

    fn get_routes(&mut self, path: &str, method: HttpMethods) -> Option<&Handler> {
        let routes = self.routes();
        routes.get(path).and_then(|handlers| handlers.get(&method))
    }
}
