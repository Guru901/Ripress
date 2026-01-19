use crate::res::response_headers::ResponseHeaders;

impl ResponseHeaders {
    /// Builder method to set a header and return self.
    ///
    /// This method allows for fluent, chainable header construction using
    /// the builder pattern. It sets a single header value, replacing any
    /// existing values for that header.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let headers = ResponseHeaders::new()
    ///     .with_header("Content-Type", "application/json")
    ///     .with_header("X-Custom", "value")
    ///     .with_header("Cache-Control", "no-cache");
    ///
    /// assert_eq!(headers.get("content-type"), Some("application/json"));
    /// assert_eq!(headers.get("x-custom"), Some("value"));
    /// ```
    pub fn with_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.insert(key, value);
        self
    }

    /// Builder method to set content type and return self.
    ///
    /// This is a convenience builder method for setting the Content-Type header
    /// in a fluent, chainable manner.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let headers = ResponseHeaders::new()
    ///     .with_content_type("application/json")
    ///     .with_header("X-Custom", "value");
    ///
    /// assert_eq!(headers.get("content-type"), Some("application/json"));
    /// ```
    pub fn with_content_type<V>(mut self, content_type: V) -> Self
    where
        V: AsRef<str>,
    {
        self.content_type(content_type);
        self
    }

    /// Builder method to set CORS headers and return self.
    ///
    /// This convenience builder method sets basic CORS headers using
    /// the [`cors_simple()`](Self::cors_simple) method in a chainable manner.
    ///
    /// # Parameters
    ///
    /// - `origin`: Specific origin to allow, or `None` for wildcard ("*")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let headers = ResponseHeaders::new()
    ///     .with_cors(Some("https://example.com"))
    ///     .with_content_type("application/json");
    ///
    /// assert_eq!(headers.get("access-control-allow-origin"), Some("https://example.com"));
    /// ```
    pub fn with_cors(mut self, origin: Option<&str>) -> Self {
        self.cors_simple(origin);
        self
    }

    /// Builder method to set security headers and return self.
    ///
    /// This convenience builder method applies basic security headers using
    /// the [`security_headers()`](Self::security_headers) method in a chainable manner.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let headers = ResponseHeaders::new()
    ///     .with_security()
    ///     .with_content_type("application/json");
    ///
    /// assert_eq!(headers.get("x-content-type-options"), Some("nosniff"));
    /// assert_eq!(headers.get("x-frame-options"), Some("DENY"));
    /// ```
    pub fn with_security(mut self) -> Self {
        self.security_headers();
        self
    }
}
