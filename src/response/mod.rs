use crate::types::{HttpResponseError, ResponseContentBody, ResponseContentType};
use bytes::Bytes;
use cookie::Cookie;
use futures::{stream, Stream, StreamExt};
use hyper::header::{HeaderName, HeaderValue, SET_COOKIE};
use hyper::{Body, Response};
use std::pin::Pin;
use std::{collections::HashMap, convert::Infallible};

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

#[derive(Debug)]
pub enum ResponseError {
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

pub(crate) type BoxError = ResponseError;

pub struct HttpResponse {
    // Status code specified by the developer
    pub status_code: u16,

    // Response body content
    body: ResponseContentBody,

    // Content type of the response
    content_type: ResponseContentType,

    // Sets response cookies
    cookies: HashMap<String, String>,

    // Sets response headers
    pub headers: HashMap<String, String>,

    // Cookies to be removed
    remove_cookies: Vec<String>,

    pub(crate) is_stream: bool,

    pub(crate) stream: Pin<Box<dyn Stream<Item = Result<Bytes, BoxError>> + Send + 'static>>,
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
        HttpResponse {
            status_code: 200,
            body: ResponseContentBody::TEXT(String::new()),
            content_type: ResponseContentType::JSON,
            cookies: HashMap::new(),
            headers: HashMap::new(),
            remove_cookies: Vec::new(),
            is_stream: false,
            stream: Box::pin(stream::empty::<Result<Bytes, BoxError>>()),
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
    ///
    /// let res = HttpResponse::new()
    ///     .set_cookie("session", "abc123")
    ///     .ok()
    ///     .text("Logged in");
    /// ```

    pub fn set_cookie(mut self, key: &str, value: &str) -> Self {
        self.cookies.insert(key.to_string(), value.to_string());
        return self;
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
        self.cookies.remove(key);
        self.remove_cookies.push(key.to_string());
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

    pub fn set_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        return self;
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

    pub fn status(mut self, code: i32) -> Self {
        self.status_code = code as u16;
        return self;
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
        return self;
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

    /// Sets the Content-Type of the response.
    ///
    /// # Arguments
    ///
    /// * `content_type` - The `ResponseContentType` to set
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
    /// let res = HttpResponse::new()
    ///     .set_content_type(ResponseContentType::JSON)
    ///     .ok()
    ///     .json(serde_json::json!({"status": "success"}));
    /// ```

    pub fn set_content_type(mut self, content_type: ResponseContentType) -> Self {
        self.content_type = content_type;
        return self;
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

    pub fn redirect(mut self, url: &str) -> Self {
        self.status_code = 302;
        self.headers.insert("Location".to_string(), url.to_string());
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

    pub fn json(mut self, json: impl serde::Serialize) -> Self {
        let json = serde_json::to_value(json).unwrap();
        self.body = ResponseContentBody::JSON(json);
        self.content_type = ResponseContentType::JSON;
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
        E: Into<BoxError> + Send + 'static,
    {
        self.is_stream = true;
        self.stream = Box::pin(stream.map(|result| result.map_err(Into::into)));
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

    pub fn html<T: Into<String>>(mut self, html_content: T) -> Self {
        self.body = ResponseContentBody::new_html(html_content);
        self.content_type = ResponseContentType::HTML;
        return self;
    }

    pub fn to_responder(self) -> Result<Response<Body>, Infallible> {
        let body = self.body;
        // if self.is_stream {
        //     let mut response = Response::builder().status(self.status_code as u16);
        //     response.header("Content-Type", "text/event-stream");

        //     self.headers.iter().for_each(|(key, value)| {
        //         response.header(key.as_str(), value.as_str());
        //     });

        //     response.header("Connection", "keep-alive");

        //     self.cookies.iter().for_each(|(key, value)| {
        //         let cookie = Cookie::build((key, value)).path("/").http_only(true);
        //         response.header(
        //             SET_COOKIE,
        //             HeaderValue::from_str(&cookie.to_string()).unwrap(),
        //         );
        //     });

        //     self.remove_cookies.iter().for_each(|key| {
        //         let expired_cookie = Cookie::build((key, ""))
        //             .path("/")
        //             .max_age(cookie::time::Duration::seconds(0));

        //         response.header(
        //             SET_COOKIE,
        //             HeaderValue::from_str(&expired_cookie.to_string()).unwrap(),
        //         );
        //     });

        //     return response;
        // } else {
        let mut response = match body {
            ResponseContentBody::JSON(json) => Response::builder()
                .status(self.status_code as u16)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&json).unwrap())),
            ResponseContentBody::TEXT(text) => Response::builder()
                .status(self.status_code as u16)
                .header("Content-Type", "text/plain")
                .body(Body::from(text)),
            ResponseContentBody::HTML(html) => Response::builder()
                .status(self.status_code as u16)
                .header("Content-Type", "text/html")
                .body(Body::from(html)),
        }
        .unwrap();

        for (key, value) in self.headers.iter() {
            response.headers_mut().append(
                HeaderName::from_bytes(key.as_bytes()).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }

        self.cookies.iter().for_each(|(key, value)| {
            let cookie = Cookie::build((key, value)).path("/").http_only(true);
            response.headers_mut().append(
                SET_COOKIE,
                HeaderValue::from_str(&cookie.to_string()).unwrap(),
            );
        });

        self.remove_cookies.iter().for_each(|key| {
            let expired_cookie = Cookie::build((key, ""))
                .path("/")
                .max_age(cookie::time::Duration::seconds(0));

            response.headers_mut().append(
                SET_COOKIE,
                HeaderValue::from_str(&expired_cookie.to_string()).unwrap(),
            );
        });

        return Ok(response);
        // }
    }

    pub(crate) fn get_body(self) -> ResponseContentBody {
        self.body
    }
}

#[cfg(test)]
impl HttpResponse {
    pub(crate) fn get_content_type(&self) -> ResponseContentType {
        self.content_type.clone()
    }

    pub(crate) fn get_cookie(self, key: String) -> Option<String> {
        self.cookies.get(&key).cloned()
    }
}
