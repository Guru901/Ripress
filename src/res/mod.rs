use crate::types::{ResponseContentBody, ResponseContentType};
use actix_web::Responder;
use actix_web::http::header::{HeaderName, HeaderValue};
use serde::Serialize;
use std::collections::HashMap;

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
    content_type: ResponseContentType,

    // Status code specified by the developer
    pub(crate) status_code: u16,

    // Sets response headers
    pub headers: HashMap<String, String>,

    // Sets response cookies
    cookies: HashMap<String, String>,

    // Cookies to be removed
    remove_cookies: Vec<String>,
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
            cookies: HashMap::new(),
            remove_cookies: Vec::new(),
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

    pub fn set_cookie(mut self, cookie_name: &str, cookie_value: &str) -> Self {
        self.cookies
            .insert(cookie_name.to_string(), cookie_value.to_string());

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
        self.cookies.remove(key);
        self.remove_cookies.push(key.to_string());
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

    pub fn redirect(mut self, path: &str) -> Self {
        self.status_code = 302;
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

    pub fn to_responder(self) -> actix_web::HttpResponse {
        let mut actix_res = actix_web::http::StatusCode::from_u16(self.status_code as u16)
            .map(|status| match self.body {
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

        self.cookies.iter().for_each(|(key, value)| {
            actix_res
                .add_cookie(&actix_web::cookie::Cookie::build(key, value).finish())
                .expect("Failed to add cookie");
        });

        return actix_res;
    }
}

impl Responder for HttpResponse {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse {
        self.to_responder()
    }
}
