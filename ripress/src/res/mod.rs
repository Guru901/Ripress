//! # HTTP Response Module
//!
//! This module provides the core [`HttpResponse`] struct and related utilities for
//! constructing HTTP responses in Ripress. It offers a fluent, expressive API to
//! set status codes, headers, cookies, and different body types (JSON, text, HTML,
//! binary, and streams).
//!
//! ## Key Features
//!
//! - **Fluent API**: Chainable methods for status, headers, cookies, and body
//! - **Typed Bodies**: JSON, text, HTML, binary, and streaming responses
//! - **Cookie Helpers**: Set and clear cookies with options (SameSite, HttpOnly, etc.)
//! - **Content-Type Handling**: Sensible defaults with explicit overrides
//! - **Streaming Support**: Send Server-Sent Events or chunked responses
//! - **Interoperability**: Convert to and from Hyper `Response<Body>`
//!
//! ## Basic Usage
//!
//! ```rust
//! use ripress::context::HttpResponse;
//!
//! // Plain text response
//! let res = HttpResponse::new().ok().text("Hello, World!");
//!
//! // JSON response
//! let res = HttpResponse::new().ok().json(serde_json::json!({
//!     "message": "Success",
//!     "code": 200
//! }));
//!
//! // HTML response
//! let res = HttpResponse::new().ok().html("<h1>Welcome</h1>");
//! ```
//!
//! ## Setting Status and Headers
//!
//! ```rust
//! use ripress::context::HttpResponse;
//!
//! let res = HttpResponse::new()
//!     .status(201)
//!     .set_header("x-request-id", "abc-123")
//!     .text("Created");
//! ```
//!
//! ## Cookies
//!
//! ```rust
//! use ripress::res::{HttpResponse, CookieOptions};
//!
//! let res = HttpResponse::new()
//!     .ok()
//!     .set_cookie(
//!         "session",
//!         "abc123",
//!         Some(CookieOptions { http_only: true, secure: true, ..Default::default() })
//!     )
//!     .text("Logged in");
//!
//! // Clear cookie
//! let res = HttpResponse::new().ok().clear_cookie("session").text("Logged out");
//! ```
//!
//! ## Redirects
//!
//! ```rust
//! use ripress::context::HttpResponse;
//!
//! let res = HttpResponse::new().redirect("/login");
//! let res = HttpResponse::new().permanent_redirect("/docs");
//! ```
//!
//! ## Streaming (SSE / chunked)
//!
//! ```rust
//! use ripress::context::HttpResponse;
//! use bytes::Bytes;
//! use futures::stream;
//! use futures::StreamExt;
//!
//! let sse = stream::iter(0..3).map(|n| Ok::<Bytes, std::io::Error>(Bytes::from(format!("data: {}\n\n", n))));
//! let res = HttpResponse::new().ok().write(sse);
//! ```
//!
//! ## Conversions with Hyper
//!
//! Internally Ripress converts `HttpResponse` to Hyper `Response<Body>` when sending, and can
//! reconstruct `HttpResponse` from Hyper responses in tests.
//!
//! - Build response to send: `to_hyper_response()`
//! - Parse response in tests: `from_hyper_response()`
//!
//! These helpers ensure consistent content-type detection and body decoding.

#![warn(missing_docs)]

use crate::res::response_status::StatusCode;
use crate::types::{ResponseBodyType, ResponseContentBody};
use bytes::Bytes;
use futures::{stream, Stream, StreamExt};
use mime_guess::from_ext;
use serde::Serialize;
use std::pin::Pin;

/// Contains the response headers struct and its methods.
pub mod response_headers;

/// Contains the response status enum and its methods.
pub mod response_status;

/// Contains cookie types used by HttpResponse (options, enums).
pub mod response_cookie;

// Re-export for backward compatibility: crate::res::CookieOptions / CookieSameSiteOptions
pub use response_cookie::{CookieOptions, CookieSameSiteOptions};
// Cookie stays crate-private
use response_cookie::Cookie;

use response_headers::ResponseHeaders;

/// Module providing type conversions from and to hyper structs into the custom structs of this lib.
pub mod conversions;

mod response_error;
pub use response_error::ResponseError;

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
    pub(crate) content_type: ResponseBodyType,

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
            content_type: ResponseBodyType::TEXT,
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

    /// Returns the current HTTP status code as a `u16`.
    pub fn status_code(&self) -> u16 {
        self.status_code.as_u16()
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
        self.content_type = ResponseBodyType::TEXT;
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
        self.content_type = ResponseBodyType::JSON;
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
        self.content_type = ResponseBodyType::BINARY;
        return self;
    }

    /// Sets a header in the response.
    ///
    /// # Example
    /// ```
    /// use ripress::context::HttpResponse;
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
    /// use ripress::res::response_cookie::CookieOptions;
    ///
    /// let res = HttpResponse::new()
    ///     .set_cookie("session", "abc123", None)
    ///     .ok()
    ///     .text("Logged in");
    /// ```

    pub fn set_cookie(
        mut self,
        cookie_name: &'static str,
        cookie_value: &'static str,
        options: Option<CookieOptions>,
    ) -> Self {
        self.cookies.push(Cookie {
            name: cookie_name,
            value: cookie_value,
            options: options.unwrap_or_default(),
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
        self.content_type = ResponseBodyType::HTML;
        self
    }

    /// Sends the contents of a file as the response body.
    /// This method reads the file at the given path asynchronously and sets the response body to its contents.
    /// The content type is inferred from the file's bytes using the `infer` crate and then mapped to a MIME
    /// type via `mime_guess`. If the type cannot be determined, it falls back to `application/octet-stream`.
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
    /// use ripress::context::HttpRequest;
    ///
    /// async fn handler(req: HttpRequest, res: HttpResponse) -> HttpResponse {
    ///     // Send a file as the response
    ///     res.ok().send_file("static/image.png").await
    /// }
    /// ```
    pub async fn send_file(mut self, path: &'static str) -> Self {
        let file = tokio::fs::read(path).await;

        match file {
            Ok(file) => {
                let file_extension = infer::get(&file)
                    .map(|info| info.extension())
                    .unwrap_or("bin");

                let mime_type = from_ext(file_extension);
                self.content_type = ResponseBodyType::from(mime_type);
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
}
