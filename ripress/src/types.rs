#![warn(missing_docs)]
use crate::helpers::{box_future, ExtractFromOwned};
use crate::middlewares::Middleware;
use crate::req::HttpRequest;
use crate::res::HttpResponse;
use bytes::Bytes;
#[cfg(feature = "with-wynd")]
use http_body_util::Full;
use hyper::Method;
use mime_guess::MimeGuess;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::future::Future;
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

    #[cfg(feature = "logger")]
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

#[cfg(feature = "with-wynd")]
pub type FutMiddleware =
    Pin<Box<dyn Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static>>;

#[cfg(not(feature = "with-wynd"))]
pub(crate) type FutMiddleware =
    Pin<Box<dyn Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static>>;

pub(crate) type HandlerMiddleware =
    Arc<dyn Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static>;

#[cfg(feature = "with-wynd")]
pub(crate) type WyndMiddlewareHandler = Arc<
    dyn Fn(
            hyper::Request<Full<Bytes>>,
        ) -> Pin<
            Box<
                dyn Future<Output = hyper::Result<hyper::Response<Full<hyper::body::Bytes>>>>
                    + Send,
            >,
        > + Send
        + Sync,
>;
pub struct RouteBuilder {
    pub(crate) path: String,
    pub(crate) method: HttpMethods,
    pub(crate) handler: Handler,
    pub(crate) middlewares: Vec<Arc<Middleware>>,
}

impl RouteBuilder {
    pub fn middleware(&self) {
        println!("Middleware");
    }
}

/// Trait providing routing functionality for applications and routers.
///
/// This trait defines methods for managing and registering HTTP routes,
/// including adding handlers for specific HTTP methods and retrieving
/// registered route handlers. Types that implement this trait must provide
/// access to their internal route storage.
pub trait RouterFns {
    /// Get a mutable reference to the internal routes map.
    ///
    /// This is used by trait default implementations to access or modify
    /// the underlying route storage for this type.
    fn routes(&mut self) -> &mut Vec<Arc<RouteBuilder>>;

    /// Register a handler for a specific HTTP method/path.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Function that handles the request, with the signature `(HttpRequest, HttpResponse) -> HFut`
    /// * `HFut` - Future outputting the final `HttpResponse`
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `path` - Route pattern (e.g., "/users")
    /// * `handler` - Handler function
    ///
    /// If a handler for a given method/path already exists, it is replaced.
    fn add_route<F, HFut>(
        &mut self,
        method: &HttpMethods,
        path: &str,
        handler: F,
    ) -> Arc<RouteBuilder>
    where
        F: Fn(HttpRequest, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
    {
        let wrapped_handler =
            Arc::new(move |req: HttpRequest, res| box_future(handler(req, res))) as Handler;

        let routes = self.routes();

        let route_builder = Arc::new(RouteBuilder {
            path: path.to_string(),
            method: method.clone(),
            handler: wrapped_handler,
            middlewares: Vec::new(),
        });

        routes.push(Arc::clone(&route_builder));

        route_builder
    }

    /// Register a GET handler for a path, with extractor integration.
    ///
    /// # Example
    /// ```
    /// use ripress::{app::App, context::{HttpRequest, HttpResponse}, types::RouterFns};
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     res.ok().text("Hello, World!")
    /// }
    ///
    /// let mut app = App::new();
    /// app.get("/hello", handler);
    /// ```
    fn get<F, HFut, P>(&mut self, path: &str, handler: F) -> Arc<RouteBuilder>
    where
        F: Fn(P, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        P: ExtractFromOwned + Send + 'static,
    {
        self.add_route_with_extraction(HttpMethods::GET, path, handler)
    }

    /// Register an OPTIONS handler for a path, with extractor integration.
    fn options<F, HFut, P>(&mut self, path: &str, handler: F) -> Arc<RouteBuilder>
    where
        F: Fn(P, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        P: ExtractFromOwned + Send + 'static,
    {
        self.add_route_with_extraction(HttpMethods::OPTIONS, path, handler)
    }

    /// Register a POST handler for a path, with extractor integration.
    fn post<F, HFut, P>(&mut self, path: &str, handler: F) -> Arc<RouteBuilder>
    where
        F: Fn(P, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        P: ExtractFromOwned + Send + 'static,
    {
        self.add_route_with_extraction(HttpMethods::POST, path, handler)
    }

    /// Register a PUT handler for a path, with extractor integration.
    fn put<F, HFut, P>(&mut self, path: &str, handler: F) -> Arc<RouteBuilder>
    where
        F: Fn(P, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        P: ExtractFromOwned + Send + 'static,
    {
        self.add_route_with_extraction(HttpMethods::PUT, path, handler)
    }

    /// Register a DELETE handler for a path, with extractor integration.
    fn delete<F, HFut, P>(&mut self, path: &str, handler: F) -> Arc<RouteBuilder>
    where
        F: Fn(P, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        P: ExtractFromOwned + Send + 'static,
    {
        self.add_route_with_extraction(HttpMethods::DELETE, path, handler)
    }

    /// Register a HEAD handler for a path, with extractor integration.
    fn head<F, HFut, P>(&mut self, path: &str, handler: F) -> Arc<RouteBuilder>
    where
        F: Fn(P, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        P: ExtractFromOwned + Send + 'static,
    {
        self.add_route_with_extraction(HttpMethods::HEAD, path, handler)
    }

    /// Register a PATCH handler for a path, with extractor integration.
    fn patch<F, HFut, P>(&mut self, path: &str, handler: F) -> Arc<RouteBuilder>
    where
        F: Fn(P, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        P: ExtractFromOwned + Send + 'static,
    {
        self.add_route_with_extraction(HttpMethods::PATCH, path, handler)
    }

    /// Retrieve the route handler for a given path/method, if one is registered.
    ///
    /// Returns `Some(&Handler)` if a matching handler exists, else `None`.
    fn get_routes(&mut self, path: &str, method: HttpMethods) -> Option<&Handler> {
        let routes = self.routes();

        for route in routes {
            if route.path == path && route.method == method {
                return Some(&route.handler);
            }
        }

        return None;
    }

    /// Internal helper: Register a handler using extractor integration.
    ///
    /// This wraps the user's handler so the extractor type `P` is populated from the HttpRequest.
    fn add_route_with_extraction<F, HFut, P>(
        &mut self,
        method: HttpMethods,
        path: &str,
        handler: F,
    ) -> Arc<RouteBuilder>
    where
        F: Fn(P, HttpResponse) -> HFut + Send + Sync + 'static,
        HFut: Future<Output = HttpResponse> + Send + 'static,
        P: ExtractFromOwned + Send + 'static,
    {
        let handler = std::sync::Arc::new(handler);

        let wrapped = move |req: HttpRequest, res: HttpResponse| {
            let handler = handler.clone();

            async move {
                let extracted = match P::extract_from_owned(req) {
                    Ok(v) => v,
                    Err(e) => {
                        return res.bad_request().text(format!("Extraction failed: {}", e));
                    }
                };

                handler(extracted, res).await
            }
        };

        self.add_route(&method, path, wrapped)
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

/// A type alias for a vector of middlewares.
///
/// Each middleware is represented as a tuple consisting of:
///   - a `&'static str` representing the path pattern for which the middleware is active,
///   - and a boxed closure.
///
/// The closure has the following signature:
///   - Accepts: `(HttpRequest, HttpResponse)`
///   - Returns: a pinned, boxed future resolving to `(HttpRequest, Option<HttpResponse>)`
///     - If `Some(HttpResponse)` is returned, the middleware chain is short-circuited and the response is sent.
///     - If `None`, processing continues to the next middleware or handler.
///
/// Middlewares can be used in both pre- and post-processing chains.
pub type Middlewares = Vec<(
    &'static str,
    Box<
        dyn Fn(
                HttpRequest,
                HttpResponse,
            )
                -> Pin<Box<dyn Future<Output = (HttpRequest, Option<HttpResponse>)> + Send>>
            + Send
            + Sync,
    >,
)>;
