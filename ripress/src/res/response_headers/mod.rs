#![warn(missing_docs)]
use std::collections::HashMap;
use std::fmt;

use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

/// Response headers builder extension methods.
///
/// This module adds builder pattern methods to [`ResponseHeaders`](super::ResponseHeaders),
/// allowing for chainable construction of response headers using a fluent API.
///
/// # Example
/// ```
/// use ripress::res::response_headers::ResponseHeaders;
///
/// let headers = ResponseHeaders::new()
///     .with_header("X-Foo", "bar")
///     .with_content_type("application/json")
///     .with_cors(None)
///     .with_security();
/// ```
pub mod builder;

/// HTTP Response Headers with support for dynamic values and response-specific features.
///
/// `ResponseHeaders` provides a type-safe, case-insensitive way to manage HTTP response headers.
/// It supports multiple values per header, convenient builder methods, and includes specialized
/// methods for common HTTP patterns like CORS, security headers, and content negotiation.
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseHeaders {
    inner: HeaderMap,
}

impl ResponseHeaders {
    /// Creates a new empty ResponseHeaders collection.
    pub fn new() -> Self {
        Self {
            inner: HeaderMap::new(),
        }
    }

    /// Creates ResponseHeaders with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HeaderMap::with_capacity(capacity),
        }
    }

    /// Creates ResponseHeaders from a HashMap<&'static str, &'static str>.
    pub fn from_static_map(map: HashMap<&'static str, &'static str>) -> Self {
        let mut headers = Self::with_capacity(map.len());
        for (key, value) in map {
            headers.insert(key, value);
        }
        headers
    }

    /// Inserts a single header value, replacing any existing values.
    ///
    /// **Performance Note:** This method parses header names/values. For hot paths,
    /// consider using static header constants directly via `insert_raw()`.
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let key_str = key.as_ref();

        // Fast path: use static constants for common headers
        let name = match key_str.to_lowercase().as_str() {
            "content-type" => hyper::header::CONTENT_TYPE,
            "content-length" => hyper::header::CONTENT_LENGTH,
            "content-disposition" => hyper::header::CONTENT_DISPOSITION,
            "cache-control" => hyper::header::CACHE_CONTROL,
            "location" => hyper::header::LOCATION,
            "set-cookie" => hyper::header::SET_COOKIE,
            "etag" => hyper::header::ETAG,
            "last-modified" => hyper::header::LAST_MODIFIED,
            "server" => hyper::header::SERVER,
            "access-control-allow-origin" => hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            "access-control-allow-methods" => hyper::header::ACCESS_CONTROL_ALLOW_METHODS,
            "access-control-allow-headers" => hyper::header::ACCESS_CONTROL_ALLOW_HEADERS,
            "access-control-allow-credentials" => hyper::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            "strict-transport-security" => hyper::header::STRICT_TRANSPORT_SECURITY,
            "content-security-policy" => hyper::header::CONTENT_SECURITY_POLICY,
            _ => {
                // Slow path: parse custom header
                if let Ok(n) = HeaderName::from_bytes(key_str.as_bytes()) {
                    n
                } else {
                    return;
                }
            }
        };

        if let Ok(val) = HeaderValue::from_bytes(value.as_ref().as_bytes()) {
            self.inner.insert(name, val);
        }
    }

    /// Direct insert using HeaderName and HeaderValue (zero parsing overhead).
    #[inline]
    pub fn insert_raw(&mut self, name: HeaderName, value: HeaderValue) {
        self.inner.insert(name, value);
    }

    /// Appends a header value, preserving existing values.
    pub fn append<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let key_str = key.as_ref();

        let name = match key_str.to_lowercase().as_str() {
            "set-cookie" => hyper::header::SET_COOKIE,
            "access-control-allow-origin" => hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            _ => {
                if let Ok(n) = HeaderName::from_bytes(key_str.as_bytes()) {
                    n
                } else {
                    return;
                }
            }
        };

        if let Ok(val) = HeaderValue::from_bytes(value.as_ref().as_bytes()) {
            self.inner.append(name, val);
        }
    }

    /// Direct append using HeaderName and HeaderValue (zero parsing overhead).
    #[inline]
    pub fn append_raw(&mut self, name: HeaderName, value: HeaderValue) {
        self.inner.append(name, value);
    }

    /// Gets the first value for a header.
    #[inline]
    pub fn get<K>(&self, key: K) -> Option<&str>
    where
        K: AsRef<str>,
    {
        let name = HeaderName::from_bytes(key.as_ref().as_bytes()).ok()?;
        self.inner.get(&name)?.to_str().ok()
    }

    /// Gets all values for a header.
    pub fn get_all<K>(&self, key: K) -> Vec<&str>
    where
        K: AsRef<str> + 'static,
    {
        if let Ok(name) = HeaderName::from_bytes(key.as_ref().as_bytes()) {
            self.inner
                .get_all(name)
                .iter()
                .filter_map(|v| v.to_str().ok())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Checks if a header exists.
    #[inline]
    pub fn contains_key<K>(&self, key: K) -> bool
    where
        K: AsRef<str>,
    {
        HeaderName::from_bytes(key.as_ref().as_bytes())
            .ok()
            .and_then(|name| self.inner.get(&name))
            .is_some()
    }

    /// Removes a header completely.
    pub fn remove<K>(&mut self, key: K) -> Option<String>
    where
        K: AsRef<str>,
    {
        let name = HeaderName::from_bytes(key.as_ref().as_bytes()).ok()?;
        self.inner.remove(&name)?.to_str().ok().map(String::from)
    }

    // === Content Headers ===

    /// Sets the Content-Type header.
    ///
    /// This is one of the most commonly used HTTP headers, specifying the media type
    /// of the response body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.content_type("application/json");
    /// assert_eq!(headers.get("content-type"), Some("application/json"));
    /// ```
    #[inline]
    pub fn content_type<V>(&mut self, content_type: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(content_type.as_ref().as_bytes()) {
            self.inner.insert(hyper::header::CONTENT_TYPE, val);
        }
    }

    /// Sets the Content-Length header.
    ///
    /// Specifies the size of the response body in bytes. This is important for
    /// HTTP/1.1 persistent connections and helps clients allocate appropriate buffers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.content_length(1024);
    /// assert_eq!(headers.get("content-length"), Some("1024"));
    /// ```
    #[inline]
    pub fn content_length(&mut self, length: u64) {
        if let Ok(val) = HeaderValue::from_str(&length.to_string()) {
            self.inner.insert(hyper::header::CONTENT_LENGTH, val);
        }
    }

    /// Sets the Location header for redirects.
    ///
    /// Used with 3xx status codes to indicate where the client should redirect.
    /// The URL can be absolute or relative.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.location("https://example.com/new-path");
    /// assert_eq!(headers.get("location"), Some("https://example.com/new-path"));
    /// ```
    #[inline]
    pub fn location<V>(&mut self, url: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(url.as_ref().as_bytes()) {
            self.inner.insert(hyper::header::LOCATION, val);
        }
    }

    /// Sets the Cache-Control header.
    ///
    /// Controls caching behavior for the response. Common values include:
    /// - `"no-cache"` - Must revalidate
    /// - `"max-age=3600"` - Cache for 1 hour
    /// - `"private"` - Only cache in private caches
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.cache_control("max-age=3600");
    /// assert_eq!(headers.get("cache-control"), Some("max-age=3600"));
    /// ```
    #[inline]
    pub fn cache_control<V>(&mut self, value: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(value.as_ref().as_bytes()) {
            self.inner.insert(hyper::header::CACHE_CONTROL, val);
        }
    }

    /// Sets headers to prevent caching.
    ///
    /// This method sets multiple headers to ensure the response is not cached
    /// by browsers, proxies, or CDNs. Useful for dynamic content or sensitive data.
    ///
    /// Headers set:
    /// - `Cache-Control: no-cache, no-store, must-revalidate`
    /// - `Pragma: no-cache` (for HTTP/1.0 compatibility)
    /// - `Expires: 0` (for HTTP/1.0 compatibility)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.no_cache();
    ///
    /// assert_eq!(headers.get("cache-control"), Some("no-cache, no-store, must-revalidate"));
    /// assert_eq!(headers.get("pragma"), Some("no-cache"));
    /// assert_eq!(headers.get("expires"), Some("0"));
    /// ```
    pub fn no_cache(&mut self) {
        let val = HeaderValue::from_static("no-cache, no-store, must-revalidate");
        self.inner.insert(hyper::header::CACHE_CONTROL, val);

        let val = HeaderValue::from_static("no-cache");
        self.inner.insert(hyper::header::PRAGMA, val);

        let val = HeaderValue::from_static("0");
        self.inner.insert(hyper::header::EXPIRES, val);
    }

    /// Sets the ETag header.
    ///
    /// ETags are used for cache validation. When a client has a cached response
    /// with an ETag, it can send an `If-None-Match` request header to check if
    /// the resource has changed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.etag("\"abc123\"");
    /// assert_eq!(headers.get("etag"), Some("\"abc123\""));
    /// ```
    #[inline]
    pub fn etag<V>(&mut self, etag: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(etag.as_ref().as_bytes()) {
            self.inner.insert(hyper::header::ETAG, val);
        }
    }

    /// Sets the Last-Modified header.
    ///
    /// Indicates when the resource was last modified. Used for cache validation
    /// with the `If-Modified-Since` request header.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.last_modified("Wed, 21 Oct 2015 07:28:00 GMT");
    /// assert_eq!(headers.get("last-modified"), Some("Wed, 21 Oct 2015 07:28:00 GMT"));
    /// ```
    #[inline]
    pub fn last_modified<V>(&mut self, date: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(date.as_ref().as_bytes()) {
            self.inner.insert(hyper::header::LAST_MODIFIED, val);
        }
    }

    /// Sets the Server header.
    ///
    /// Identifies the server software. While optional, it can be useful for
    /// debugging and statistics. Consider security implications of revealing
    /// server information.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.server("MyApp/1.0");
    /// assert_eq!(headers.get("server"), Some("MyApp/1.0"));
    /// ```
    #[inline]
    pub fn server<V>(&mut self, server: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(server.as_ref().as_bytes()) {
            self.inner.insert(hyper::header::SERVER, val);
        }
    }

    /// Sets the X-Powered-By header.
    ///
    /// Indicates the technology stack powering the application. Note that this
    /// can be a security risk as it reveals information to potential attackers.
    /// Consider using [`remove_powered_by()`](Self::remove_powered_by) instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.powered_by("Rust/Tokio");
    /// assert_eq!(headers.get("x-powered-by"), Some("Rust/Tokio"));
    /// ```
    pub fn powered_by<V>(&mut self, value: V)
    where
        V: AsRef<str>,
    {
        self.insert("x-powered-by", value);
    }

    /// Removes the X-Powered-By header (security best practice).
    ///
    /// Many frameworks automatically add this header, but it can be a security
    /// risk as it reveals information about your technology stack to potential
    /// attackers. This method removes it entirely.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.powered_by("Express");
    /// headers.remove_powered_by();
    /// assert!(!headers.contains_key("x-powered-by"));
    /// ```
    pub fn remove_powered_by(&mut self) {
        let name = HeaderName::from_static("x-powered-by");
        self.inner.remove(&name);
    }

    // === CORS Headers ===

    /// Sets the Access-Control-Allow-Origin header.
    ///
    /// Specifies which origins are allowed to access the resource. Use `"*"` for
    /// public APIs, or specify specific origins for better security.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.cors_allow_origin("https://example.com");
    /// assert_eq!(headers.get("access-control-allow-origin"), Some("https://example.com"));
    /// ```
    #[inline]
    pub fn cors_allow_origin<V>(&mut self, origin: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(origin.as_ref().as_bytes()) {
            self.inner
                .insert(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, val);
        }
    }

    /// Sets the Access-Control-Allow-Methods header.
    ///
    /// Specifies which HTTP methods are allowed for CORS requests.
    /// Common values include combinations of GET, POST, PUT, DELETE, OPTIONS.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.cors_allow_methods("GET, POST, PUT");
    /// assert_eq!(headers.get("access-control-allow-methods"), Some("GET, POST, PUT"));
    /// ```
    #[inline]
    pub fn cors_allow_methods<V>(&mut self, methods: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(methods.as_ref().as_bytes()) {
            self.inner
                .insert(hyper::header::ACCESS_CONTROL_ALLOW_METHODS, val);
        }
    }

    /// Sets the Access-Control-Allow-Headers header.
    ///
    /// Specifies which headers can be used in CORS requests. Commonly includes
    /// Content-Type, Authorization, and custom headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.cors_allow_headers("Content-Type, Authorization");
    /// assert_eq!(headers.get("access-control-allow-headers"), Some("Content-Type, Authorization"));
    /// ```
    #[inline]
    pub fn cors_allow_headers<V>(&mut self, headers: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(headers.as_ref().as_bytes()) {
            self.inner
                .insert(hyper::header::ACCESS_CONTROL_ALLOW_HEADERS, val);
        }
    }

    /// Sets the Access-Control-Allow-Credentials header.
    ///
    /// Indicates whether credentials (cookies, authorization headers, TLS certificates)
    /// can be included in CORS requests. When `true`, the Access-Control-Allow-Origin
    /// cannot be `"*"`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.cors_allow_credentials(true);
    /// assert_eq!(headers.get("access-control-allow-credentials"), Some("true"));
    /// ```
    pub fn cors_allow_credentials(&mut self, allow: bool) {
        let value = if allow { "true" } else { "false" };
        let val = HeaderValue::from_static(value);
        self.inner
            .insert(hyper::header::ACCESS_CONTROL_ALLOW_CREDENTIALS, val);
    }

    /// Sets basic CORS headers for simple requests.
    ///
    /// This is a convenience method that sets commonly used CORS headers:
    /// - Access-Control-Allow-Origin (to specified origin or "*")
    /// - Access-Control-Allow-Methods (GET, POST, PUT, DELETE, OPTIONS)
    /// - Access-Control-Allow-Headers (Content-Type, Authorization)
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
    /// let mut headers = ResponseHeaders::new();
    /// headers.cors_simple(Some("https://example.com"));
    ///
    /// assert_eq!(headers.get("access-control-allow-origin"), Some("https://example.com"));
    /// assert_eq!(headers.get("access-control-allow-methods"), Some("GET, POST, PUT, DELETE, OPTIONS"));
    /// ```
    pub fn cors_simple(&mut self, origin: Option<&str>) {
        match origin {
            Some(origin) => self.cors_allow_origin(origin),
            None => {
                let val = HeaderValue::from_static("*");
                self.inner
                    .insert(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, val);
            }
        }
        let val = HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS");
        self.inner
            .insert(hyper::header::ACCESS_CONTROL_ALLOW_METHODS, val);
        let val = HeaderValue::from_static("Content-Type, Authorization");
        self.inner
            .insert(hyper::header::ACCESS_CONTROL_ALLOW_HEADERS, val);
    }

    // === Security Headers ===

    /// Sets the X-Frame-Options header to prevent clickjacking.
    ///
    /// Common values:
    /// - `"DENY"` - Never allow framing
    /// - `"SAMEORIGIN"` - Allow framing from same origin
    /// - `"ALLOW-FROM uri"` - Allow framing from specific URI
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.frame_options("DENY");
    /// assert_eq!(headers.get("x-frame-options"), Some("DENY"));
    /// ```
    pub fn frame_options<V>(&mut self, value: V)
    where
        V: AsRef<str>,
    {
        self.insert("x-frame-options", value);
    }

    /// Sets X-Content-Type-Options to prevent MIME type sniffing.
    ///
    /// This header prevents browsers from MIME-sniffing a response away from the
    /// declared content-type. This helps prevent XSS attacks that rely on MIME
    /// type confusion.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.no_sniff();
    /// assert_eq!(headers.get("x-content-type-options"), Some("nosniff"));
    /// ```
    pub fn no_sniff(&mut self) {
        self.insert("x-content-type-options", "nosniff");
    }

    /// Sets the X-XSS-Protection header.
    ///
    /// Controls the browser's XSS filtering feature. When enabled, sets the value
    /// to `"1; mode=block"` which enables XSS filtering and blocks the page if
    /// an attack is detected.
    ///
    /// # Parameters
    ///
    /// - `enabled`: Whether to enable XSS protection
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.xss_protection(true);
    /// assert_eq!(headers.get("x-xss-protection"), Some("1; mode=block"));
    /// ```
    pub fn xss_protection(&mut self, enabled: bool) {
        let value = if enabled { "1; mode=block" } else { "0" };
        self.insert("x-xss-protection", value);
    }

    /// Sets the Strict-Transport-Security header (HSTS).
    ///
    /// Forces clients to use HTTPS for future requests to the domain.
    /// This helps prevent man-in-the-middle attacks and protocol downgrade attacks.
    ///
    /// # Parameters
    ///
    /// - `max_age`: Time in seconds that the browser should remember to use HTTPS
    /// - `include_subdomains`: Whether to apply HSTS to all subdomains
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.hsts(31536000, true); // 1 year with subdomains
    /// assert_eq!(headers.get("strict-transport-security"), Some("max-age=31536000; includeSubDomains"));
    /// ```
    pub fn hsts(&mut self, max_age: u64, include_subdomains: bool) {
        let value = if include_subdomains {
            format!("max-age={}; includeSubDomains", max_age)
        } else {
            format!("max-age={}", max_age)
        };

        if let Ok(val) = HeaderValue::from_str(&value) {
            self.inner
                .insert(hyper::header::STRICT_TRANSPORT_SECURITY, val);
        }
    }

    /// Sets the Content-Security-Policy header.
    ///
    /// CSP helps prevent XSS attacks by controlling which resources the browser
    /// is allowed to load. Policies are specified as directives separated by semicolons.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.csp("default-src 'self'; script-src 'self' 'unsafe-inline'");
    /// assert_eq!(
    ///     headers.get("content-security-policy"),
    ///     Some("default-src 'self'; script-src 'self' 'unsafe-inline'")
    /// );
    /// ```
    #[inline]
    pub fn csp<V>(&mut self, policy: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(policy.as_ref().as_bytes()) {
            self.inner
                .insert(hyper::header::CONTENT_SECURITY_POLICY, val);
        }
    }

    /// Sets a collection of basic security headers.
    ///
    /// This convenience method sets multiple security headers at once:
    /// - X-Content-Type-Options: nosniff
    /// - X-XSS-Protection: 1; mode=block
    /// - X-Frame-Options: DENY
    /// - Removes X-Powered-By header
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.security_headers();
    ///
    /// assert_eq!(headers.get("x-content-type-options"), Some("nosniff"));
    /// assert_eq!(headers.get("x-xss-protection"), Some("1; mode=block"));
    /// assert_eq!(headers.get("x-frame-options"), Some("DENY"));
    /// ```
    pub fn security_headers(&mut self) {
        self.no_sniff();
        self.xss_protection(true);
        self.frame_options("DENY");
        self.remove_powered_by();
    }

    // === Content Type Shortcuts ===

    /// Sets content type to JSON (application/json).
    ///
    /// This is a convenience method for the most common API response format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.json();
    /// assert_eq!(headers.get("content-type"), Some("application/json"));
    /// ```
    #[inline]
    pub fn json(&mut self) {
        let val = HeaderValue::from_static("application/json");
        self.inner.insert(hyper::header::CONTENT_TYPE, val);
    }

    /// Sets content type to HTML with UTF-8 charset.
    ///
    /// Appropriate for HTML pages and includes charset specification for proper
    /// character encoding handling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.html();
    /// assert_eq!(headers.get("content-type"), Some("text/html; charset=utf-8"));
    /// ```
    #[inline]
    pub fn html(&mut self) {
        let val = HeaderValue::from_static("text/html; charset=utf-8");
        self.inner.insert(hyper::header::CONTENT_TYPE, val);
    }

    /// Sets content type to plain text with UTF-8 charset.
    ///
    /// Appropriate for plain text responses, logs, or simple data formats.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.text();
    /// assert_eq!(headers.get("content-type"), Some("text/plain; charset=utf-8"));
    /// ```
    #[inline]
    pub fn text(&mut self) {
        let val = HeaderValue::from_static("text/plain; charset=utf-8");
        self.inner.insert(hyper::header::CONTENT_TYPE, val);
    }

    /// Sets content type to XML (application/xml).
    ///
    /// Appropriate for XML data, RSS feeds, and XML-based APIs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.xml();
    /// assert_eq!(headers.get("content-type"), Some("application/xml"));
    /// ```
    #[inline]
    pub fn xml(&mut self) {
        let val = HeaderValue::from_static("application/xml");
        self.inner.insert(hyper::header::CONTENT_TYPE, val);
    }

    /// Sets the Content-Disposition header for file downloads.
    ///
    /// This header indicates that the response should be downloaded as a file
    /// rather than displayed in the browser. The filename parameter specifies
    /// the suggested filename for the download.
    ///
    /// # Parameters
    ///
    /// - `filename`: The suggested filename for the download
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.attachment("document.pdf");
    /// assert_eq!(headers.get("content-disposition"), Some("attachment; filename=\"document.pdf\""));
    /// ```
    pub fn attachment<V>(&mut self, filename: V)
    where
        V: Into<String>,
    {
        let value = format!("attachment; filename=\"{}\"", filename.into());
        if let Ok(val) = HeaderValue::from_str(&value) {
            self.inner.insert(hyper::header::CONTENT_DISPOSITION, val);
        }
    }

    /// Sets the Content-Disposition header to inline.
    ///
    /// This header indicates that the response should be displayed inline
    /// in the browser rather than downloaded as a file. This is the default
    /// behavior for most content types, but can be explicitly set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.inline();
    /// assert_eq!(headers.get("content-disposition"), Some("inline"));
    /// ```
    pub fn inline(&mut self) {
        let val = HeaderValue::from_static("inline");
        self.inner.insert(hyper::header::CONTENT_DISPOSITION, val);
    }

    // === Utility Methods ===

    /// Returns an iterator over all header names.
    ///
    /// This method provides access to all header names (keys) stored in the
    /// ResponseHeaders collection. All names are returned in lowercase.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.insert("Content-Type", "application/json");
    /// headers.insert("X-Custom", "value");

    /// let keys = headers.keys();
    /// assert_eq!(keys.len(), 2);
    /// assert!(
    ///     keys.iter()
    ///         .any(|k| k.as_str().eq_ignore_ascii_case("content-type"))
    /// );
    /// assert!(
    ///     keys.iter()
    ///         .any(|k| k.as_str().eq_ignore_ascii_case("x-custom"))
    /// );
    /// ```
    pub fn keys(&self) -> Vec<&HeaderName> {
        self.inner.keys().collect()
    }

    /// Returns the number of headers.
    ///
    /// This counts the number of unique header names, not the total number
    /// of header values (since headers can have multiple values).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// assert_eq!(headers.len(), 0);
    ///
    /// headers.insert("Content-Type", "application/json");
    /// headers.append("Set-Cookie", "session=abc");
    /// headers.append("Set-Cookie", "theme=dark");
    /// assert_eq!(headers.len(), 2); // 2 unique header names
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.keys().len()
    }

    /// Returns `true` if the headers collection contains no headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// assert!(headers.is_empty());
    ///
    /// headers.insert("Content-Type", "application/json");
    /// assert!(!headers.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns an iterator over header name-value pairs.
    ///
    /// For headers with multiple values, only the first value is included
    /// in the iteration. Use [`iter_all()`](Self::iter_all) to access all values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.insert("Content-Type", "application/json");
    /// headers.append("Set-Cookie", "session=abc");
    /// headers.append("Set-Cookie", "theme=dark");
    ///
    /// for (name, value) in headers.iter() {
    ///     println!("{}: {}", name, value);
    /// }
    /// // Output:
    /// // content-type: application/json
    /// // set-cookie: session=abc
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|val| (k.as_str(), val)))
    }

    /// Converts headers to HTTP header lines format.
    ///
    /// Returns a vector of strings in the format "header-name: header-value",
    /// suitable for HTTP protocol transmission. Headers with multiple values
    /// will generate multiple lines.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.insert("Content-Type", "application/json");
    /// headers.append("Set-Cookie", "session=abc");
    /// headers.append("Set-Cookie", "theme=dark");
    ///
    /// let lines = headers.to_header_lines();
    /// // lines will contain:
    /// // ["content-type: application/json", "set-cookie: session=abc", "set-cookie: theme=dark"]
    /// ```
    pub fn to_header_lines(&self) -> Vec<String> {
        self.inner
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|val| format!("{}: {}", k, val)))
            .collect()
    }

    /// Returns a reference to the inner HeaderMap
    #[inline]
    pub fn as_header_map(&self) -> &HeaderMap {
        &self.inner
    }

    /// Consumes self and returns the inner HeaderMap
    #[inline]
    pub fn into_header_map(self) -> HeaderMap {
        self.inner
    }
}

impl Default for ResponseHeaders {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ResponseHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, value) in self.iter() {
            writeln!(f, "{}: {}", key, value)?;
        }
        Ok(())
    }
}

impl std::ops::Index<&str> for ResponseHeaders {
    type Output = str;

    fn index(&self, key: &str) -> &Self::Output {
        self.get(key)
            .unwrap_or_else(|| panic!("Header '{}' not found", key))
    }
}

impl From<HashMap<&'static str, &'static str>> for ResponseHeaders {
    fn from(map: HashMap<&'static str, &'static str>) -> Self {
        Self::from_static_map(map)
    }
}

impl From<HeaderMap> for ResponseHeaders {
    fn from(map: HeaderMap) -> Self {
        Self { inner: map }
    }
}

impl From<ResponseHeaders> for HeaderMap {
    fn from(headers: ResponseHeaders) -> Self {
        headers.into_header_map()
    }
}
