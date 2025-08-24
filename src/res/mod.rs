#![warn(missing_docs)]

use crate::req::determine_content_type_response;
use crate::res::response_status::StatusCode;
use crate::types::{ResponseContentBody, ResponseContentType};
use bytes::Bytes;
use futures::{Stream, StreamExt, stream};
use hyper::body::to_bytes;
use hyper::header::{HeaderName, HeaderValue, SET_COOKIE};
use hyper::{Body, Response};
use mime_guess::from_ext;
use serde::Serialize;
use std::convert::Infallible;
use std::pin::Pin;

/// Contains the response headers struct and its methods.
pub mod response_headers;

/// Contains the response status enum and its methods.
pub mod response_status;

use response_headers::ResponseHeaders;

/// Represents errors that can occur when generating an HTTP response.
///
/// This enum is used to encapsulate possible error types that may arise during
/// the process of constructing or streaming an HTTP response. It is primarily
/// used for error handling in streaming responses or when IO operations fail.
///
/// # Variants
///
/// - `IoError(std::io::Error)`: Represents an IO error that occurred, such as a failure
///   to read from or write to a stream.
/// - `_Other(&'static str)`: Represents a generic or custom error with a static string message.
#[derive(Debug)]
pub enum ResponseError {
    /// An IO error occurred.
    IoError(std::io::Error),
    /// A generic or custom error with a static string message.
    _Other(&'static str),
}

impl From<std::io::Error> for ResponseError {
    fn from(err: std::io::Error) -> Self {
        ResponseError::IoError(err)
    }
}

impl std::error::Error for ResponseError {}
impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseError::IoError(e) => write!(f, "IO error: {}", e),
            ResponseError::_Other(e) => write!(f, "Error: {}", e),
        }
    }
}

/// Represents an HTTP response being sent to the client.
///
/// The HttpResponse struct provides methods to construct and manipulate HTTP responses
/// including status codes, headers, cookies, and different types of response bodies.
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// use ripress::context::HttpResponse;
///
/// let res = HttpResponse::new();
/// res.ok().text("Hello, World!");
/// ```
///
/// JSON response:
/// ```rust
/// use ripress::context::HttpResponse;
/// use serde_json::json;
///
/// let res = HttpResponse::new();
/// res.ok().json(json!({
///     "message": "Success",
///     "code": 200
/// }));
/// ```
///
/// # Fields
/// - `status_code` - HTTP status code (e.g., 200, 404, 500)
/// - `body` - Response body content (JSON, text)
/// - `content_type` - Content-Type header value
/// - `cookies` - Response cookies to be set
/// - `headers` - Response headers
/// - `remove_cookies` - Cookies to be removed

/// Options for the SameSite attribute of cookies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookieSameSiteOptions {
    /// Sets the SameSite attribute to Strict
    Strict,

    /// Sets the SameSite attribute to Lax
    Lax,

    /// Sets the SameSite attribute to None
    None,
}

/// Options for setting cookies
#[derive(Debug, Clone, PartialEq)]
pub struct CookieOptions {
    /// Sets the HttpOnly attribute
    pub http_only: bool,

    /// Sets the Secure attribute
    pub secure: bool,

    /// Sets the SameSite attribute
    pub same_site: CookieSameSiteOptions,

    /// Sets the Path attribute
    pub path: Option<&'static str>,

    /// Sets the Domain attribute
    pub domain: Option<&'static str>,

    /// Sets the Max-Age attribute
    pub max_age: Option<i64>,

    /// Sets the Expires attribute
    pub expires: Option<i64>,
}

impl Default for CookieOptions {
    fn default() -> Self {
        Self {
            http_only: true,
            secure: true,
            same_site: CookieSameSiteOptions::None,
            path: Some("/"),
            domain: None,
            max_age: None,
            expires: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Cookie {
    pub name: &'static str,
    pub value: &'static str,
    pub(crate) options: CookieOptions,
}

/// Represents an HTTP response being sent to the client.
///
/// The HttpResponse struct provides methods to construct and manipulate HTTP responses
/// including status codes, headers, cookies, and different types of response bodies.
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// use ripress::context::HttpResponse;
///
/// let res = HttpResponse::new();
/// res.ok().text("Hello, World!");
/// ```
///
/// JSON response:
/// ```rust
/// use ripress::context::HttpResponse;
/// use serde_json::json;
///
/// let res = HttpResponse::new();
/// res.ok().json(json!({
///     "message": "Success",
///     "code": 200
/// }));
/// ```
///
/// # Fields
/// - `status_code` - HTTP status code (e.g., 200, 404, 500)
/// - `body` - Response body content (JSON, text)
/// - `content_type` - Content-Type header value
/// - `cookies` - Response cookies to be set
/// - `headers` - Response headers
/// - `remove_cookies` - Cookies to be removed

pub struct HttpResponse {
    // Response body content
    pub(crate) body: ResponseContentBody,

    // Content type of the response
    pub(crate) content_type: ResponseContentType,

    // Status code specified by the developer
    pub(crate) status_code: StatusCode,

    /// Sets response headers
    pub headers: ResponseHeaders,

    // Sets response cookies
    pub(crate) cookies: Vec<Cookie>,

    // Cookies to be removed
    pub(crate) remove_cookies: Vec<&'static str>,

    pub(crate) is_stream: bool,

    pub(crate) stream: Pin<Box<dyn Stream<Item = Result<Bytes, ResponseError>> + Send + 'static>>,
}

impl std::fmt::Debug for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpResponse")
            .field("status_code", &self.status_code)
            .field("body", &self.body)
            .field("content_type", &self.content_type)
            .field("cookies", &self.cookies)
            .field("headers", &self.headers)
            .field("remove_cookies", &self.remove_cookies)
            .field("is_stream", &self.is_stream)
            .field("stream", &"<stream>")
            .finish()
    }
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for HttpResponse {
    fn clone(&self) -> Self {
        Self {
            status_code: self.status_code,
            body: self.body.clone(),
            content_type: self.content_type.clone(),
            cookies: self.cookies.clone(),
            headers: self.headers.clone(),
            remove_cookies: self.remove_cookies.clone(),
            is_stream: self.is_stream,
            stream: Box::pin(stream::empty()),
        }
    }
}

impl HttpResponse {
    /// Creates a new HTTP response with default values.
    ///
    /// # Returns
    ///
    /// Returns a new `HttpResponse` initialized with:
    /// - Status code: 200
    /// - Empty text body
    /// - JSON content type
    /// - Empty cookies and headers
    ///
    /// # Example
    /// ```rust
    /// use ripress::context::HttpResponse;
    ///
    /// let res = HttpResponse::new();
    /// ```

    pub fn new() -> Self {
        Self {
            status_code: StatusCode::Ok,
            body: ResponseContentBody::TEXT(String::new()),
            content_type: ResponseContentType::TEXT,
            headers: ResponseHeaders::new(),
            cookies: Vec::new(),
            remove_cookies: Vec::new(),
            is_stream: false,
            stream: Box::pin(stream::empty::<Result<Bytes, ResponseError>>()),
        }
    }

    /// Sets the status code to 200 OK.
    pub fn ok(mut self) -> Self {
        self.status_code = StatusCode::Ok;
        self
    }

    /// Sets the status code to 201 Created.
    pub fn created(mut self) -> Self {
        self.status_code = StatusCode::Created;
        self
    }

    /// Sets the status code to 202 Accepted.
    pub fn accepted(mut self) -> Self {
        self.status_code = StatusCode::Accepted;
        self
    }

    /// Sets the status code to 204 No Content.
    pub fn no_content(mut self) -> Self {
        self.status_code = StatusCode::NoContent;
        self
    }

    /// Sets the status code to 400 Bad Request.
    pub fn bad_request(mut self) -> Self {
        self.status_code = StatusCode::BadRequest;
        return self;
    }

    /// Sets the status code to 401 Unauthorized.
    pub fn unauthorized(mut self) -> Self {
        self.status_code = StatusCode::Unauthorized;
        return self;
    }

    /// Sets the status code to 403 Forbidden.
    pub fn forbidden(mut self) -> Self {
        self.status_code = StatusCode::Forbidden;
        return self;
    }

    /// Sets the status code to 404 Not Found.
    pub fn not_found(mut self) -> Self {
        self.status_code = StatusCode::NotFound;
        return self;
    }

    /// Sets the status code to 405 Method Not Allowed.
    pub fn method_not_allowed(mut self) -> Self {
        self.status_code = StatusCode::MethodNotAllowed;
        return self;
    }

    /// Sets the status code to 409 Conflict.
    pub fn conflict(mut self) -> Self {
        self.status_code = StatusCode::Conflict;
        return self;
    }

    /// Sets the status code to 500 Internal Server Error.
    pub fn internal_server_error(mut self) -> Self {
        self.status_code = StatusCode::InternalServerError;
        return self;
    }

    /// Sets the status code to 501 Not Implemented.
    pub fn not_implemented(mut self) -> Self {
        self.status_code = StatusCode::NotImplemented;
        return self;
    }

    /// Sets the status code to 502 Bad Gateway.
    pub fn bad_gateway(mut self) -> Self {
        self.status_code = StatusCode::BadGateway;
        return self;
    }

    /// Sets the status code to 503 Service Unavailable.
    pub fn service_unavailable(mut self) -> Self {
        self.status_code = StatusCode::ServiceUnavailable;
        return self;
    }

    /// Sets the status code to a given u16 value.
    pub fn status(mut self, status_code: u16) -> Self {
        self.status_code = StatusCode::from_u16(status_code);
        return self;
    }

    /// Sets the response body to text.
    ///
    /// # Arguments
    ///
    /// * `text` - Any type that can be converted into a String
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining
    ///
    /// # Example
    /// ```rust
    /// use ripress::context::HttpResponse;
    ///
    /// let res = HttpResponse::new()
    ///     .ok()
    ///     .text("Operation completed successfully");
    ///
    /// // Using with different types
    /// let res = HttpResponse::new()
    ///     .ok()
    ///     .text(format!("Count: {}", 42));
    /// ```

    pub fn text<T: Into<String>>(mut self, text: T) -> Self {
        self.body = ResponseContentBody::new_text(text);
        self.content_type = ResponseContentType::TEXT;
        return self;
    }

    /// Sets the response body to JSON.
    ///
    /// # Arguments
    ///
    /// * `json` - Any type that implements `serde::Serialize`
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining
    ///
    /// # Example
    /// ```rust
    /// use ripress::context::HttpResponse;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct User {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// let user = User {
    ///     name: "John".to_string(),
    ///     age: 30,
    /// };
    ///
    /// let res = HttpResponse::new()
    ///     .ok()
    ///     .json(user);
    /// ```

    pub fn json<T: Serialize>(mut self, json: T) -> Self {
        self.body = ResponseContentBody::new_json(json);
        self.content_type = ResponseContentType::JSON;
        return self;
    }

    /// Sets the response body to binary data.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Any type that can be converted into `Bytes`
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining
    ///
    /// # Example
    /// ```rust
    /// use ripress::context::HttpResponse;
    /// use bytes::Bytes;
    ///
    /// let data = vec![1, 2, 3, 4, 5];
    ///
    /// let res = HttpResponse::new()
    ///     .ok()
    ///     .bytes(data);
    ///
    /// // Using with Bytes directly
    /// let res = HttpResponse::new()
    ///     .ok()
    ///     .bytes(Bytes::from_static(b"hello world"));
    /// ```

    pub fn bytes<T: Into<Bytes>>(mut self, bytes: T) -> Self {
        self.body = ResponseContentBody::new_binary(bytes.into());
        self.content_type = ResponseContentType::BINARY;
        return self;
    }

    /// Sets a header in the response.
    ///
    /// # Example
    /// ```
    /// use ripress::context::HttpResponse;
    /// let res = HttpResponse::new();
    /// res.set_header("key", "value"); // Sets the key cookie to value
    /// ```

    pub fn set_header<T: Into<String>>(
        mut self,
        header_name: &'static str,
        header_value: T,
    ) -> Self {
        self.headers.insert(header_name, header_value.into());
        self
    }

    /// Sets a cookie in the response.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the cookie
    /// * `value` - The value to set
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining
    ///
    /// # Example
    /// ```rust
    /// use ripress::context::HttpResponse;
    /// use ripress::res::CookieOptions;
    ///
    /// let res = HttpResponse::new()
    ///     .set_cookie("session", "abc123", CookieOptions::default())
    ///     .ok()
    ///     .text("Logged in");
    /// ```

    pub fn set_cookie(
        mut self,
        cookie_name: &'static str,
        cookie_value: &'static str,
        options: CookieOptions,
    ) -> Self {
        self.cookies.push(Cookie {
            name: cookie_name,
            value: cookie_value,
            options,
        });

        self
    }
    /// Removes a cookie from the response.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the cookie to remove
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining
    ///
    /// # Example
    /// ```rust
    /// use ripress::context::HttpResponse;
    ///
    /// let res = HttpResponse::new()
    ///     .clear_cookie("session")
    ///     .ok()
    ///     .text("Logged out");
    /// ```

    pub fn clear_cookie(mut self, key: &'static str) -> Self {
        self.cookies.retain(|cookie| cookie.name != key);
        self.remove_cookies.push(key);
        self
    }

    /// Redirects the client to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The url to redirect to
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining
    ///
    /// # Example
    /// ```rust
    /// use ripress::context::HttpResponse;
    ///
    /// let res = HttpResponse::new();
    /// res.redirect("https://www.example.com");
    /// ```

    pub fn redirect(mut self, path: &'static str) -> Self {
        self.status_code = StatusCode::Redirect;
        self.headers.insert("Location", path);
        self
    }

    /// Permanently redirects the client to the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The url to redirect to
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining
    ///
    /// # Example
    /// ```rust
    /// use ripress::context::HttpResponse;
    ///
    /// let res = HttpResponse::new();
    /// res.permanent_redirect("https://www.example.com");
    /// ```

    pub fn permanent_redirect(mut self, path: &'static str) -> Self {
        self.status_code = StatusCode::PermanentRedirect;
        self.headers.insert("Location", path);
        self
    }

    /// Sets the response body to html.
    ///
    /// # Arguments
    ///
    /// * `html_content` - Any type that can be converted into a String
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining
    ///
    /// # Example
    /// ```rust
    /// use ripress::context::HttpResponse;
    ///
    /// let res = HttpResponse::new()
    ///     .ok()
    ///     .html("<h1>Hello, World!</h1>");
    ///
    /// // Using with different types
    /// let res = HttpResponse::new()
    ///     .ok()
    ///     .text(format!("<h1>Count: {}</h1>", 42));
    /// ```

    pub fn html(mut self, html: &str) -> Self {
        self.body = ResponseContentBody::new_html(html);
        self.content_type = ResponseContentType::HTML;
        self
    }

    /// Sends the contents of a file as the response body.
    ///
    /// This method reads the file at the given path asynchronously and sets the response body to its contents.
    /// The content type is inferred from the file's extension using the `infer` crate. If the file type cannot be determined,
    /// it defaults to "bin".
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to be sent. Must be a static string slice.
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining.
    ///
    /// # Example
    /// ```no_run
    /// use ripress::context::HttpResponse;
    ///
    /// // Send a file as the response
    /// let res = HttpResponse::new()
    ///     .ok()
    ///     .send_file("static/image.png")
    ///     .await;
    /// ```
    pub async fn send_file(mut self, path: &'static str) -> Self {
        let file = tokio::fs::read(path).await;

        match file {
            Ok(file) => {
                let file_extension = infer::get(&file)
                    .map(|info| info.extension())
                    .unwrap_or("bin");

                let mime_type = from_ext(file_extension);
                self.content_type = ResponseContentType::from(mime_type);
                self.body = ResponseContentBody::new_binary(file);
            }
            Err(e) => {
                eprintln!("Error reading file: {}", e);
            }
        }

        self
    }

    /// Streams the response
    ///
    /// # Arguments
    ///
    /// * `stream` - The stream to stream
    ///
    /// # Returns
    ///
    /// Returns `Self` for method chaining
    ///
    /// # Example
    /// ```rust
    /// use ripress::context::HttpResponse;
    /// use bytes::Bytes;
    /// use futures::stream;
    /// use futures::StreamExt;
    ///
    /// let res = HttpResponse::new();
    ///
    /// let stream = stream::iter(0..5).map(|n| Ok::<Bytes, std::io::Error>(Bytes::from(format!("Number: {}\n", n))));
    ///
    /// res.write(stream);
    /// ```
    pub fn write<S, E>(mut self, stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes, E>> + Send + 'static,
        E: Into<ResponseError> + Send + 'static,
    {
        self.is_stream = true;
        self.headers.insert("transfer-encoding", "chunked");
        self.headers.insert("cache-control", "no-cache");
        self.stream = Box::pin(stream.map(|result| result.map_err(Into::into)));
        self
    }

    pub(crate) async fn from_hyper_response(res: &mut Response<Body>) -> Self {
        let body_bytes = to_bytes(res.body_mut()).await.unwrap_or(Bytes::new());

        let content_type_hdr = res
            .headers()
            .get(hyper::header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok());

        let content_type = content_type_hdr
            .map(determine_content_type_response)
            .unwrap_or(ResponseContentType::BINARY);

        let body = match content_type {
            ResponseContentType::BINARY => ResponseContentBody::new_binary(body_bytes),
            ResponseContentType::TEXT => {
                let text = String::from_utf8(body_bytes.to_vec())
                    .unwrap_or_else(|_| String::from_utf8_lossy(&body_bytes).into_owned());
                ResponseContentBody::new_text(text)
            }
            ResponseContentType::JSON => {
                // Avoid panic: if JSON parsing fails, fallback to empty object
                let json_value =
                    serde_json::from_slice(&body_bytes).unwrap_or(serde_json::Value::Null);
                ResponseContentBody::new_json(json_value)
            }
            ResponseContentType::HTML => {
                let html = String::from_utf8(body_bytes.to_vec())
                    .unwrap_or_else(|_| String::from_utf8_lossy(&body_bytes).into_owned());
                ResponseContentBody::new_html(html)
            }
        };

        // Heuristic for SSE streams: text/event-stream + keep-alive
        let is_event_stream = content_type_hdr
            .map(|ct| ct.eq_ignore_ascii_case("text/event-stream"))
            .unwrap_or(false);
        let is_keep_alive = res
            .headers()
            .get(hyper::header::CONNECTION)
            .and_then(|h| h.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("keep-alive"))
            .unwrap_or(false);
        let is_stream = is_event_stream && is_keep_alive;

        let status_code = StatusCode::from_u16(res.status().as_u16());
        let mut headers = ResponseHeaders::new();

        for (key, value) in res.headers().iter() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.as_str(), v);
            }
        }
        for value in res.headers().get_all(SET_COOKIE).iter() {
            if let Ok(v) = value.to_str() {
                headers.insert("Set-Cookie", v);
            }
        }

        HttpResponse {
            body,
            content_type,
            status_code,
            headers,
            cookies: Vec::new(),
            remove_cookies: Vec::new(),
            is_stream,
            stream: Box::pin(stream::empty::<Result<Bytes, ResponseError>>()),
        }
    }

    pub(crate) fn to_hyper_response(self) -> Result<Response<Body>, Infallible> {
        let body = self.body;
        if self.is_stream {
            let mut response = Response::builder().status(self.status_code.as_u16());
            response = response.header("Content-Type", "text/event-stream");

            for (key, value) in self.headers.iter() {
                response = response.header(key.as_str(), value);
            }

            response = response.header("Connection", "keep-alive");

            for c in self.cookies.iter() {
                let cookie = cookie::Cookie::build((c.name, c.value))
                    .http_only(c.options.http_only)
                    .same_site(match c.options.same_site {
                        crate::res::CookieSameSiteOptions::Lax => cookie::SameSite::Lax,
                        crate::res::CookieSameSiteOptions::Strict => cookie::SameSite::Strict,
                        crate::res::CookieSameSiteOptions::None => cookie::SameSite::None,
                    })
                    .secure(c.options.secure)
                    .path(c.options.path.unwrap_or("/"))
                    .max_age(cookie::time::Duration::seconds(
                        c.options.max_age.unwrap_or(0),
                    ));
                response = response.header(
                    SET_COOKIE,
                    HeaderValue::from_str(&cookie.to_string()).unwrap(),
                );
            }

            for key in self.remove_cookies.iter() {
                let expired_cookie = cookie::Cookie::build((key.to_owned(), ""))
                    .path("/")
                    .max_age(cookie::time::Duration::seconds(0));

                response = response.header(
                    SET_COOKIE,
                    HeaderValue::from_str(&expired_cookie.to_string()).unwrap(),
                );
            }

            return Ok(response.body(Body::wrap_stream(self.stream)).unwrap());
        } else {
            println!("body: {:?}", body);

            let mut response = match body {
                ResponseContentBody::JSON(json) => Response::builder()
                    .status(self.status_code.as_u16())
                    .header("Content-Type", self.content_type.as_str())
                    .body(Body::from(serde_json::to_string(&json).unwrap())),
                ResponseContentBody::TEXT(text) => Response::builder()
                    .status(self.status_code.as_u16())
                    .header("Content-Type", self.content_type.as_str())
                    .body(Body::from(text)),
                ResponseContentBody::HTML(html) => Response::builder()
                    .status(self.status_code.as_u16())
                    .header("Content-Type", self.content_type.as_str())
                    .body(Body::from(html)),
                ResponseContentBody::BINARY(bytes) => Response::builder()
                    .status(self.status_code.as_u16())
                    .header("Content-Type", self.content_type.as_str())
                    .body(Body::from(bytes)),
            }
            .unwrap();

            for (key, value) in self.headers.iter() {
                if key.eq_ignore_ascii_case("content-type") {
                    // Already set via `.header("Content-Type", ...)` above; skip duplicates
                    continue;
                }

                response.headers_mut().append(
                    HeaderName::from_bytes(key.as_bytes()).unwrap(),
                    HeaderValue::from_str(value).unwrap(),
                );
            }

            self.cookies.iter().for_each(|c| {
                let cookie = cookie::Cookie::build((c.name, c.value))
                    .http_only(c.options.http_only)
                    .same_site(match c.options.same_site {
                        crate::res::CookieSameSiteOptions::Lax => cookie::SameSite::Lax,
                        crate::res::CookieSameSiteOptions::Strict => cookie::SameSite::Strict,
                        crate::res::CookieSameSiteOptions::None => cookie::SameSite::None,
                    })
                    .secure(c.options.secure)
                    .path(c.options.path.unwrap_or("/"))
                    .max_age(cookie::time::Duration::seconds(
                        c.options.max_age.unwrap_or(0),
                    ));
                response.headers_mut().append(
                    SET_COOKIE,
                    HeaderValue::from_str(&cookie.to_string()).unwrap(),
                );
            });

            self.remove_cookies.iter().for_each(|key| {
                let expired_cookie = cookie::Cookie::build((key.to_owned(), ""))
                    .path("/")
                    .max_age(cookie::time::Duration::seconds(0));

                response.headers_mut().append(
                    SET_COOKIE,
                    HeaderValue::from_str(&expired_cookie.to_string()).unwrap(),
                );
            });

            return Ok(response);
        }
    }
}

#[cfg(test)]
impl HttpResponse {
    pub(crate) fn get_status_code(&self) -> u16 {
        self.status_code.as_u16()
    }

    pub(crate) fn get_content_type(&self) -> &ResponseContentType {
        &self.content_type
    }

    pub(crate) fn get_body(self) -> ResponseContentBody {
        self.body
    }

    pub(crate) fn get_cookie(&self, key: &str) -> Option<&'static str> {
        self.cookies
            .iter()
            .find(|cookie| cookie.name == key)
            .map(|cookie| cookie.value)
    }
}
