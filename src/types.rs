#[warn(missing_docs)]
use crate::app::box_future;
use crate::req::HttpRequest;
use crate::res::HttpResponse;
use bytes::Bytes;
use hyper::Method;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ResponseContentBody {
    TEXT(String),
    HTML(String),
    JSON(serde_json::Value),
    BINARY(Bytes),
}

impl ResponseContentBody {
    /// Returns the content length in bytes for the current variant.
    /// Note:
    /// - TEXT/HTML: returns `String::len()` (UTF-8 byte length)
    /// - JSON: returns the length of the compact serialized form
    /// - BINARY: returns `Bytes::len()`

    pub fn len(&self) -> usize {
        match self {
            ResponseContentBody::TEXT(text) => text.len(),
            ResponseContentBody::HTML(html) => html.len(),
            ResponseContentBody::JSON(json) => {
                serde_json::to_vec(json).map(|v| v.len()).unwrap_or(0)
            }
            ResponseContentBody::BINARY(bytes) => bytes.len(),
        }
    }
}

impl ResponseContentBody {
    pub(crate) fn new_text<T: Into<String>>(text: T) -> Self {
        ResponseContentBody::TEXT(text.into())
    }

    pub(crate) fn new_json<T: Serialize>(json: T) -> Self {
        Self::try_new_json(json).expect("Failed to serialize to JSON")
    }

    pub(crate) fn try_new_json<T: Serialize>(json: T) -> Result<Self, serde_json::Error> {
        serde_json::to_value(json).map(ResponseContentBody::JSON)
    }

    pub(crate) fn new_html<T: Into<String>>(html: T) -> Self {
        ResponseContentBody::HTML(html.into())
    }

    pub(crate) fn new_binary(bytes: Bytes) -> Self {
        ResponseContentBody::BINARY(bytes)
    }
}
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ResponseContentType {
    TEXT,
    JSON,
    HTML,
    BINARY,
}

pub(crate) type Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>;

pub(crate) type Handler = Arc<dyn Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static>;

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

impl From<&Method> for HttpMethods {
    fn from(method: &Method) -> Self {
        match method {
            &Method::GET => HttpMethods::GET,
            &Method::POST => HttpMethods::POST,
            &Method::PUT => HttpMethods::PUT,
            &Method::DELETE => HttpMethods::DELETE,
            &Method::PATCH => HttpMethods::PATCH,
            &Method::HEAD => HttpMethods::HEAD,
            &Method::OPTIONS => HttpMethods::OPTIONS,
            _ => HttpMethods::GET,
        }
    }
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
            HttpMethods::OPTIONS => "OPTIONS",
        };
        write!(f, "{}", method)
    }
}

pub(crate) type Routes = HashMap<String, HashMap<HttpMethods, Handler>>;

#[derive(Debug, PartialEq)]
pub enum HttpRequestError {
    MissingCookie(String),
    MissingParam(String),
    MissingHeader(String),
    MissingQuery(String),
    InvalidJson(String),
}

impl std::fmt::Display for HttpRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpRequestError::MissingCookie(cookie) => write!(f, "Cookie {} doesn't exist", cookie),
            HttpRequestError::MissingParam(param) => write!(f, "Param {} doesn't exist", param),
            HttpRequestError::MissingHeader(header) => write!(f, "Header {} doesn't exist", header),
            HttpRequestError::MissingQuery(query) => write!(f, "Query {} doesn't exist", query),
            HttpRequestError::InvalidJson(json) => write!(f, "JSON is invalid: {}", json),
        }
    }
}

#[derive(Debug)]
pub(crate) enum _HttpResponseError {
    MissingHeader(String),
}

impl std::fmt::Display for _HttpResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            _HttpResponseError::MissingHeader(header) => {
                write!(f, "Header {} doesn't exist", header)
            }
        }
    }
}
pub(crate) type FutMiddleware =
    Pin<Box<dyn Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static>>;
pub(crate) type HandlerMiddleware =
    Arc<dyn Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static>;

pub trait RouterFns {
    fn routes(&mut self) -> &mut Routes;

    fn add_route<F, HFut>(&mut self, method: HttpMethods, path: &str, handler: F)
    where
        F: Fn(HttpRequest, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
    {
        let routes = self.routes();
        let wrapped_handler = Arc::new(move |req, res| box_future(handler(req, res))) as Handler;
        use std::collections::hash_map::Entry;
        match routes.entry(path.to_string()) {
            Entry::Occupied(mut e) => {
                e.get_mut().insert(method, wrapped_handler);
            }
            Entry::Vacant(e) => {
                let mut map = HashMap::new();
                map.insert(method, wrapped_handler);
                e.insert(map);
            }
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

    fn get<F, HFut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(HttpRequest, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        Self: Sized,
    {
        self.add_route(HttpMethods::GET, path, handler);
        self
    }

    /// Add an OPTIONS route to the application or router.
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
    /// app.options("/hello", handler);
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
    /// router.options("/hello", handler);
    /// ```

    fn options<F, HFut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(HttpRequest, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        Self: Sized,
    {
        self.add_route(HttpMethods::OPTIONS, path, handler);
        self
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

    fn post<F, HFut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(HttpRequest, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        Self: Sized,
    {
        self.add_route(HttpMethods::POST, path, handler);
        self
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

    fn put<F, HFut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(HttpRequest, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        Self: Sized,
    {
        self.add_route(HttpMethods::PUT, path, handler);
        self
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

    fn delete<F, HFut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(HttpRequest, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        Self: Sized,
    {
        self.add_route(HttpMethods::DELETE, path, handler);
        self
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

    fn head<F, HFut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(HttpRequest, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        Self: Sized,
    {
        self.add_route(HttpMethods::HEAD, path, handler);
        self
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

    fn patch<F, HFut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(HttpRequest, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        Self: Sized,
    {
        self.add_route(HttpMethods::PATCH, path, handler);
        self
    }

    fn get_routes(&mut self, path: &str, method: HttpMethods) -> Option<&Handler> {
        let routes = self.routes();
        routes.get(path).and_then(|handlers| handlers.get(&method))
    }
}
