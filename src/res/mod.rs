#![warn(missing_docs)]

use crate::types::{HttpResponseError, ResponseContentBody, ResponseContentType};
use actix_web::Responder;
use actix_web::http::header::{HeaderName, HeaderValue};
use bytes::Bytes;
use futures::{Stream, StreamExt, stream};
use serde::Serialize;
use std::collections::HashMap;
use std::pin::Pin;

#[derive(Debug)]
pub(crate) enum ResponseError {
    IoError(std::io::Error),
    Other(String),
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
            ResponseError::Other(e) => write!(f, "Error: {}", e),
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
pub struct CookieOptions {
    /// Sets the HttpOnly attribute
    pub http_only: bool,

    /// Sets the Secure attribute
    pub secure: bool,

    /// Sets the SameSite attribute
    pub same_site: CookieSameSiteOptions,

    /// Sets the Path attribute
    pub path: Option<String>,

    /// Sets the Domain attribute
    pub domain: Option<String>,

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
            path: Some("/".to_string()),
            domain: None,
            max_age: None,
            expires: None,
        }
    }
}

struct Cookie {
    name: String,
    value: String,
    options: CookieOptions,
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
    body: ResponseContentBody,

    // Content type of the response
    pub(crate) content_type: ResponseContentType,

    // Status code specified by the developer
    pub(crate) status_code: u16,

    /// Sets response headers
    pub headers: HashMap<String, String>,

    // Sets response cookies
    cookies: Vec<Cookie>,

    // Cookies to be removed
    remove_cookies: Vec<String>,

    pub(crate) is_stream: bool,

    pub(crate) stream: Pin<Box<dyn Stream<Item = Result<Bytes, ResponseError>> + Send + 'static>>,
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
            status_code: 200,
            body: ResponseContentBody::TEXT(String::new()),
            content_type: ResponseContentType::TEXT,
            headers: HashMap::new(),
            cookies: Vec::new(),
            remove_cookies: Vec::new(),
            is_stream: false,
            stream: Box::pin(stream::empty::<Result<Bytes, ResponseError>>()),
        }
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

    /// Sets the status code of the response.
    ///
    /// # Arguments
    ///
    /// * `code` - The HTTP status code to set
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
    ///     .status(201)
    ///     .text("Resource created");
    /// ```

    pub fn status(mut self, code: u16) -> Self {
        self.status_code = code;
        self
    }

    /// Sets a header in the response.
    ///
    /// # Example
    /// ```
    /// use ripress::context::HttpResponse;
    /// let res = HttpResponse::new();
    /// res.set_header("key", "value"); // Sets the key cookie to value
    /// ```

    pub fn set_header(mut self, header_name: &str, header_value: &str) -> Self {
        self.headers
            .insert(header_name.to_string(), header_value.to_string());

        self
    }

    /// Gets a header from the response.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the header to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Result<String, HttpResponseError>` with the header value if found,
    /// or `HttpResponseError::MissingHeader` if not found.
    ///
    /// # Example
    /// ```rust
    /// use ripress::context::HttpResponse;
    ///
    /// let res = HttpResponse::new()
    ///     .set_header("X-Custom", "value");
    ///
    /// match res.get_header("X-Custom") {
    ///     Ok(value) => println!("Header value: {}", value),
    ///     Err(e) => println!("Error: {:?}", e)
    /// }
    /// ```

    pub fn get_header(&self, key: &str) -> Result<String, HttpResponseError> {
        let header = self.headers.get(key);

        match header {
            Some(header_string) => Ok(header_string.clone()),
            None => Err(HttpResponseError::MissingHeader(key.to_string())),
        }
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
        cookie_name: &str,
        cookie_value: &str,
        options: CookieOptions,
    ) -> Self {
        self.cookies.push(Cookie {
            name: cookie_name.to_owned(),
            value: cookie_value.to_owned(),
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

    pub fn clear_cookie(mut self, key: &str) -> Self {
        self.cookies.retain(|cookie| cookie.name != key);
        self.remove_cookies.push(key.to_string());
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
    /// use ripress::types::ResponseContentType;
    ///
    /// let res = HttpResponse::new();
    /// res.redirect("https://www.example.com");
    /// ```

    pub fn redirect(mut self, path: &str) -> Self {
        self.status_code = 302;
        self.headers
            .insert("Location".to_string(), path.to_string());

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
    /// use ripress::types::ResponseContentType;
    ///
    /// let res = HttpResponse::new();
    /// res.permanent_redirect("https://www.example.com");
    /// ```

    pub fn permanent_redirect(mut self, path: &str) -> Self {
        self.status_code = 301;
        self.headers
            .insert("Location".to_string(), path.to_string());

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

    /// Sets the status code to 200 (OK).
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
    ///     .json(serde_json::json!({
    ///         "status": "success",
    ///         "message": "Operation completed"
    ///     }));
    /// ```

    pub fn ok(mut self) -> Self {
        self.status_code = 200;
        self
    }

    /// Sets the status code to 401 (Unauthorized).
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
    ///     .unauthorized()
    ///     .text("Unauthorized");
    /// ```

    pub fn unauthorized(mut self) -> Self {
        self.status_code = 401;
        return self;
    }

    /// Sets the status code to 400 (Bad Request).
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
    ///     .bad_request()
    ///     .json(serde_json::json!({
    ///         "error": "Invalid input",
    ///         "details": "Missing required fields"
    ///     }));
    /// ```

    pub fn bad_request(mut self) -> Self {
        self.status_code = 400;
        return self;
    }

    /// Sets the status code to 404 (Not Found).
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
    ///     .not_found()
    ///     .json(serde_json::json!({
    ///         "error": "Resource not found",
    ///         "resource": "user/123"
    ///     }));
    /// ```

    pub fn not_found(mut self) -> Self {
        self.status_code = 404;
        return self;
    }

    /// Sets the status code to 500 (Internal Server Error).
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
    ///     .internal_server_error()
    ///     .json(serde_json::json!({
    ///         "error": "Internal server error",
    ///         "message": "Database connection failed"
    ///     }));
    /// ```

    pub fn internal_server_error(mut self) -> Self {
        self.status_code = 500;
        return self;
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
        self.headers
            .insert("transfer-encoding".to_string(), "chunked".to_string());
        self.headers
            .insert("cache-control".to_string(), "no-cache".to_string());
        self.stream = Box::pin(stream.map(|result| result.map_err(Into::into)));
        self
    }

    pub(crate) fn to_responder(self) -> actix_web::HttpResponse {
        let body = self.body;
        if self.is_stream {
            let mut actix_res = actix_web::HttpResponse::build(actix_web::http::StatusCode::OK);
            actix_res.content_type("text/event-stream");

            self.headers.iter().for_each(|(key, value)| {
                actix_res.append_header((key.as_str(), value.as_str()));
            });

            actix_res.append_header(("Connection", "keep-alive"));
            self.remove_cookies.iter().for_each(|key| {
                actix_res.cookie(actix_web::cookie::Cookie::build(key, "").finish());
            });

            self.cookies.iter().for_each(|cookie| {
                actix_res.cookie(
                    actix_web::cookie::Cookie::build(
                        cookie.name.to_string(),
                        cookie.value.to_string(),
                    )
                    .expires(cookie.options.expires.and_then(|ts| {
                        actix_web::cookie::time::OffsetDateTime::from_unix_timestamp(ts).ok()
                    }))
                    .http_only(cookie.options.http_only)
                    .max_age(
                        cookie
                            .options
                            .max_age
                            .map(|secs| actix_web::cookie::time::Duration::seconds(secs))
                            .unwrap_or_else(|| actix_web::cookie::time::Duration::seconds(0)),
                    )
                    .path(cookie.options.path.as_deref().unwrap_or("/"))
                    .secure(cookie.options.secure)
                    .same_site(match cookie.options.same_site {
                        crate::res::CookieSameSiteOptions::Lax => actix_web::cookie::SameSite::Lax,
                        crate::res::CookieSameSiteOptions::Strict => {
                            actix_web::cookie::SameSite::Strict
                        }
                        crate::res::CookieSameSiteOptions::None => {
                            actix_web::cookie::SameSite::None
                        }
                    })
                    .finish(),
                );
            });

            return actix_res.streaming(self.stream);
        } else {
            let mut actix_res = actix_web::http::StatusCode::from_u16(self.status_code as u16)
                .map(|status| match body {
                    ResponseContentBody::JSON(json) => actix_web::HttpResponse::build(status)
                        .content_type("application/json")
                        .json(json),
                    ResponseContentBody::TEXT(text) => actix_web::HttpResponse::build(status)
                        .content_type("text/plain")
                        .body(text),
                    ResponseContentBody::HTML(html) => actix_web::HttpResponse::build(status)
                        .content_type("text/html")
                        .body(html),
                })
                .unwrap_or_else(|_| {
                    actix_web::HttpResponse::InternalServerError().body("Invalid status code")
                });

            self.headers.iter().for_each(|(key, value)| {
                actix_res.headers_mut().append(
                    HeaderName::from_bytes(key.as_bytes()).unwrap(),
                    HeaderValue::from_str(value).unwrap(),
                )
            });

            self.remove_cookies.iter().for_each(|key| {
                actix_res
                    .add_cookie(&actix_web::cookie::Cookie::build(key, "").finish())
                    .unwrap();
            });

            self.cookies.iter().for_each(|cookie| {
                actix_res
                    .add_cookie(
                        &actix_web::cookie::Cookie::build(
                            cookie.name.to_string(),
                            cookie.value.to_string(),
                        )
                        .expires(cookie.options.expires.and_then(|ts| {
                            actix_web::cookie::time::OffsetDateTime::from_unix_timestamp(ts).ok()
                        }))
                        .http_only(cookie.options.http_only)
                        .max_age(
                            cookie
                                .options
                                .max_age
                                .map(|secs| actix_web::cookie::time::Duration::seconds(secs))
                                .unwrap_or_else(|| actix_web::cookie::time::Duration::seconds(0)),
                        )
                        .path(cookie.options.path.as_deref().unwrap_or("/"))
                        .secure(cookie.options.secure)
                        .same_site(match cookie.options.same_site {
                            crate::res::CookieSameSiteOptions::Lax => {
                                actix_web::cookie::SameSite::Lax
                            }
                            crate::res::CookieSameSiteOptions::Strict => {
                                actix_web::cookie::SameSite::Strict
                            }
                            crate::res::CookieSameSiteOptions::None => {
                                actix_web::cookie::SameSite::None
                            }
                        })
                        .finish(),
                    )
                    .expect("Failed to add cookie");
            });
            return actix_res;
        }
    }
}

impl Responder for HttpResponse {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse {
        self.to_responder()
    }
}

#[cfg(test)]
impl HttpResponse {
    pub(crate) fn get_status_code(&self) -> u16 {
        self.status_code
    }

    pub(crate) fn get_content_type(&self) -> &ResponseContentType {
        &self.content_type
    }

    pub(crate) fn get_body(self) -> ResponseContentBody {
        self.body
    }

    pub(crate) fn get_cookie(&self, key: &str) -> Option<String> {
        self.cookies
            .iter()
            .find(|cookie| cookie.name == key)
            .map(|cookie| cookie.value.clone())
    }
}
