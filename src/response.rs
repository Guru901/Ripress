use actix_web::Responder;

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
}

impl HttpResponse {
    pub fn new() -> Self {
        HttpResponse {
            status_code: 200,
            body: ResponseContentBody::TEXT(String::new()),
            content_type: ResponseContentType::JSON,
        }
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
        self.status_code = 401;
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

    pub fn json(mut self, json: serde_json::Value) -> Self {
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
        actix_web::http::StatusCode::from_u16(self.status_code as u16)
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
            })
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
}
