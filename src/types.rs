#![warn(missing_docs)]
use crate::app::box_future;
use crate::req::HttpRequest;
use crate::res::HttpResponse;
use bytes::Bytes;
use hyper::Method;
use mime_guess::MimeGuess;
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

    pub(crate) fn new_binary<T: Into<Bytes>>(bytes: T) -> Self {
        ResponseContentBody::BINARY(bytes.into())
    }
}
#[derive(Eq, PartialEq, Debug, Clone)]
pub(crate) enum ResponseContentType {
    TEXT,
    JSON,
    HTML,
    BINARY,
}

impl From<MimeGuess> for ResponseContentType {
    fn from(guess: MimeGuess) -> Self {
        let mime = guess.first_or_octet_stream();

        match (mime.type_(), mime.subtype()) {
            (mime::TEXT, mime::HTML) => ResponseContentType::HTML,
            (mime::TEXT, _) => ResponseContentType::TEXT,
            (mime::APPLICATION, mime::JSON) => ResponseContentType::JSON,
            _ => ResponseContentType::BINARY,
        }
    }
}

impl ResponseContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResponseContentType::TEXT => "text/plain",
            ResponseContentType::JSON => "application/json",
            ResponseContentType::HTML => "text/html",
            ResponseContentType::BINARY => "application/octet-stream",
        }
    }
}

pub(crate) type Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>;

pub(crate) type Handler = Arc<dyn Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static>;

/// Represents the supported HTTP methods for routing and request handling.
///
/// # Variants
/// - `GET`: The HTTP GET method, typically used for retrieving resources.
/// - `POST`: The HTTP POST method, commonly used for creating resources or submitting data.
/// - `PUT`: The HTTP PUT method, generally used for updating or replacing resources.
/// - `HEAD`: The HTTP HEAD method, used to retrieve headers for a resource without the body.
/// - `DELETE`: The HTTP DELETE method, used to remove resources.
/// - `PATCH`: The HTTP PATCH method, used for making partial updates to resources.
/// - `OPTIONS`: The HTTP OPTIONS method, used to describe the communication options for the target resource.
#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum HttpMethods {
    /// The HTTP GET method, typically used for retrieving resources.
    GET,
    /// The HTTP POST method, commonly used for creating resources or submitting data.
    POST,
    /// The HTTP PUT method, generally used for updating or replacing resources.
    PUT,
    /// The HTTP HEAD method, used to retrieve headers for a resource without the body.
    HEAD,
    /// The HTTP DELETE method, used to remove resources.
    DELETE,
    /// The HTTP PATCH method, used for making partial updates to resources.
    PATCH,
    /// The HTTP OPTIONS method, used to describe the communication options for the target resource.
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

/// Represents possible errors that can occur when handling an HTTP request.
#[derive(Debug, PartialEq)]
pub enum HttpRequestError {
    /// Indicates that a required cookie is missing.
    /// The associated `String` is the name of the missing cookie.
    MissingCookie(String),
    /// Indicates that a required URL parameter is missing.
    /// The associated `String` is the name of the missing parameter.
    MissingParam(String),
    /// Indicates that a required HTTP header is missing.
    /// The associated `String` is the name of the missing header.
    MissingHeader(String),
    /// Indicates that a required query parameter is missing.
    /// The associated `String` is the name of the missing query parameter.
    MissingQuery(String),
    /// Indicates that the request body contains invalid JSON.
    /// The associated `String` provides details about the JSON error.
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

#[cfg_attr(feature = "with-wynd", visibility::make(pub))]
pub(crate) type FutMiddleware =
    Pin<Box<dyn Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static>>;

pub(crate) type HandlerMiddleware =
    Arc<dyn Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static>;

#[cfg(feature = "with-wynd")]
pub(crate) type WyndMiddlewareHandler = Arc<
    dyn Fn(
            hyper::Request<hyper::Body>,
        )
            -> Pin<Box<dyn Future<Output = hyper::Result<hyper::Response<hyper::Body>>> + Send>>
        + Send
        + Sync,
>;

/// Trait providing routing functionality for applications and routers.
///
/// This trait defines methods for managing and registering HTTP routes,
/// including adding handlers for specific HTTP methods and retrieving
/// registered route handlers. Types that implement this trait must provide
/// access to their internal route storage.
pub trait RouterFns {
    /// Returns a mutable reference to the internal `Routes` map.
    ///
    /// Types implementing this trait should use this method to expose
    /// their underlying route storage for manipulation and registration.
    ///
    /// # Returns
    ///
    /// A mutable reference to the internal `Routes` map.
    fn routes(&mut self) -> &mut Routes;

    /// Adds a new route handler for a specific HTTP method and path.
    ///
    /// This method registers a handler function for the given HTTP method and path.
    /// If a handler for the method and path already exists, it will be replaced.
    ///
    /// # Type Parameters
    ///
    /// * `F` - The handler function type. Must accept a `HttpRequest` and `HttpResponse` and return a future.
    /// * `HFut` - The future returned by the handler, which resolves to a `HttpResponse`.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method (e.g., GET, POST) for which the route is registered.
    /// * `path` - The path string for the route (e.g., "/users").
    /// * `handler` - The handler function to be called when the route is matched.
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

    /// Registers a GET route handler.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self` for chaining.
    ///
    /// # Example (App)
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
    /// # Example (Router)
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

    /// Registers an OPTIONS route handler.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self` for chaining.
    ///
    /// # Example (App)
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
    /// # Example (Router)
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

    /// Registers a POST route handler.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self` for chaining.
    ///
    /// # Example (App)
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
    /// # Example (Router)
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

    /// Registers a PUT route handler.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self` for chaining.
    ///
    /// # Example (App)
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
    /// # Example (Router)
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

    /// Registers a DELETE route handler.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self` for chaining.
    ///
    /// # Example (App)
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
    /// # Example (Router)
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

    /// Registers a HEAD route handler.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self` for chaining.
    ///
    /// # Example (App)
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
    /// # Example (Router)
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

    /// Registers a PATCH route handler.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the route.
    /// * `handler` - The handler function for the route.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self` for chaining.
    ///
    /// # Example (App)
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
    /// # Example (Router)
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

    /// Retrieves a reference to the handler for a given path and HTTP method, if it exists.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the route.
    /// * `method` - The HTTP method for the route.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the handler if found, or `None` if not found.
    fn get_routes(&mut self, path: &str, method: HttpMethods) -> Option<&Handler> {
        let routes = self.routes();
        routes.get(path).and_then(|handlers| handlers.get(&method))
    }
}

#[cfg(test)]
mod test {
    use crate::types::ResponseContentBody;

    impl ResponseContentBody {
        pub(crate) fn get_content_as_bytes(&self) -> Vec<u8> {
            match self {
                ResponseContentBody::TEXT(text) => text.as_bytes().to_vec(),
                ResponseContentBody::HTML(html) => html.as_bytes().to_vec(),
                ResponseContentBody::JSON(json) => serde_json::to_vec(json).unwrap_or_default(),
                ResponseContentBody::BINARY(bytes) => bytes.to_vec(),
            }
        }
    }
}
