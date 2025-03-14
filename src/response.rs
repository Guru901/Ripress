use std::collections::HashMap;

use actix_web::{
    http::header::{HeaderName, HeaderValue},
    Responder,
};

use crate::types::{ResponseContentBody, ResponseContentType};

/// Represents an http response going to the client
///
/// This struct holds various properties of an HTTP response, such as
/// status code, body content, and content type.
///
/// # Example
/// ```
/// use ripress::context::HttpResponse;
/// let req = HttpResponse::new();
/// ```
///
/// # Fields
/// - `status_code`: Stores status code of the response.
/// - `body`: Contains the response body, which may be JSON, text, or form data.
/// - `content_type`: The content type of the response.
pub struct HttpResponse {
    // Status code specified by the developer
    status_code: i32,

    // Response body content
    body: ResponseContentBody,

    // Content type of the response
    content_type: ResponseContentType,

    // Sets response cookies
    cookies: HashMap<String, String>,

    // Sets response headers
    headers: HashMap<String, String>,

    // Cookies to be removed
    remove_cookies: Vec<String>,
}

impl HttpResponse {
    pub fn new() -> Self {
        HttpResponse {
            status_code: 200,
            body: ResponseContentBody::TEXT(String::new()),
            content_type: ResponseContentType::JSON,
            cookies: HashMap::new(),
            headers: HashMap::new(),
            remove_cookies: Vec::new(),
        }
    }

    /// Sets a cookie in the response.
    ///
    /// # Example
    /// ```
    /// use ripress::context::HttpResponse;
    /// let res = HttpResponse::new();
    /// res.set_cookie("key", "value"); // Sets the key cookie to value
    /// ```

    pub fn set_cookie(mut self, key: &str, value: &str) -> Self {
        self.cookies.insert(key.to_string(), value.to_string());
        return self;
    }

    /// Removes a cookie from the response.
    ///
    /// # Example
    /// ```
    /// use ripress::context::HttpResponse;
    /// let res = HttpResponse::new();
    /// res.clear_cookie("key"); // The cookie gets removed
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
    /// # Example
    /// ```
    /// use ripress::context::HttpResponse;
    /// let res = HttpResponse::new();
    /// res.get_header("key"); // Gets the key header
    /// ```

    pub fn get_header(self, key: String) -> Option<String> {
        self.headers.get(&key).cloned()
    }

    /// Sets the status code of the response.
    ///
    /// # Example
    /// ```
    /// use ripress::context::HttpResponse;
    /// let res = HttpResponse::new();
    /// res.status(404); // Sets the status code to 404
    /// ```

    pub fn status(mut self, code: i32) -> Self {
        self.status_code = code;
        return self;
    }

    /// Sets the status code to 200 (OK).
    ///
    /// # Example
    /// ```
    /// use ripress::context::HttpResponse;
    /// let res = HttpResponse::new();
    /// res.ok(); // Sets the status code to 200
    /// ```

    pub fn ok(mut self) -> Self {
        self.status_code = 200;
        return self;
    }

    /// Sets the status code to 400 (Bad Request).
    ///
    /// # Example
    /// ```
    /// use ripress::context::HttpResponse;
    /// let res = HttpResponse::new();
    /// res.bad_request(); // Sets the status code to 400
    /// ```

    pub fn bad_request(mut self) -> Self {
        self.status_code = 400;
        return self;
    }

    /// Sets the status code to 404 (Not Found).
    ///
    /// # Example
    /// ```
    /// use ripress::context::HttpResponse;
    /// let res = HttpResponse::new();
    /// res.not_found(); // Sets the status code to 404
    /// ```

    pub fn not_found(mut self) -> Self {
        self.status_code = 404;
        return self;
    }

    /// Sets the status code to 500 (Internal Server Error).
    ///
    /// # Example
    /// ```
    /// use ripress::context::HttpResponse;
    /// let res = HttpResponse::new();
    /// res.internal_server_error(); // Sets the status code to 500
    /// ```

    pub fn internal_server_error(mut self) -> Self {
        self.status_code = 500;
        return self;
    }

    pub fn set_content_type(mut self, content_type: ResponseContentType) -> Self {
        self.content_type = content_type;
        return self;
    }

    /// Sets the response body to JSON.
    ///
    /// # Example
    /// ```
    /// use ripress::context::HttpResponse;
    /// use serde_json::json;
    ///
    /// let json_body = json!({"key": "value"});
    /// let res = HttpResponse::new();
    ///
    /// res.json(json_body); // Sets the response body to JSON
    /// ```

    pub fn json(mut self, json: impl serde::Serialize) -> Self {
        let json = serde_json::to_value(json).unwrap();
        self.body = ResponseContentBody::JSON(json);
        self.content_type = ResponseContentType::JSON;
        return self;
    }

    /// Sets the response body to text.
    ///
    /// # Example
    /// ```
    /// use ripress::context::HttpResponse;
    /// let res = HttpResponse::new();
    ///
    /// res.text("Hello, World!"); // Sets the response body to text
    /// ```

    pub fn text<T: Into<String>>(mut self, text: T) -> Self {
        self.body = ResponseContentBody::new_text(text);
        self.content_type = ResponseContentType::TEXT;
        return self;
    }

    pub fn to_responder(self) -> actix_web::HttpResponse {
        let body = self.body;
        let mut actix_res = actix_web::http::StatusCode::from_u16(self.status_code as u16)
            .map(|status| match body {
                ResponseContentBody::JSON(json) => actix_web::HttpResponse::build(status)
                    .content_type("application/json")
                    .json(json),
                ResponseContentBody::TEXT(text) => actix_web::HttpResponse::build(status)
                    .content_type("text/plain")
                    .body(text),
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

#[cfg(test)]
impl HttpResponse {
    pub fn get_status_code(&self) -> i32 {
        self.status_code
    }

    pub(crate) fn get_content_type(&self) -> ResponseContentType {
        self.content_type.clone()
    }

    pub(crate) fn get_body(self) -> ResponseContentBody {
        self.body
    }

    pub(crate) fn get_cookie(self, key: String) -> Option<String> {
        self.cookies.get(&key).cloned()
    }
}
