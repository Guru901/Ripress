use std::collections::HashMap;
use std::fmt;

/// HTTP Response Headers with support for dynamic values and response-specific features.
///
/// `ResponseHeaders` provides a type-safe, case-insensitive way to manage HTTP response headers.
/// It supports multiple values per header, convenient builder methods, and includes specialized
/// methods for common HTTP patterns like CORS, security headers, and content negotiation.
///
/// # Key Features
///
/// - **Case-insensitive**: Header names are normalized to lowercase for consistent lookups
/// - **Multiple values**: Support for headers that can have multiple values (like `Set-Cookie`)
/// - **Type safety**: Generic methods accept various string types (`&str`, `String`, etc.)
/// - **Convenience methods**: Specialized setters for common headers and patterns
/// - **Builder pattern**: Chainable methods for fluent header construction
/// - **HTTP compliance**: Follows HTTP specifications for header handling
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use ripress::res::response_headers::ResponseHeaders;
///
/// let mut headers = ResponseHeaders::new();
/// headers.insert("Content-Type", "application/json");
/// headers.insert("Content-Length", "1024");
///
/// assert_eq!(headers.get("content-type"), Some("application/json"));
/// assert_eq!(headers.get("Content-Length"), Some("1024")); // Case-insensitive
/// ```
///
/// ## Builder Pattern
///
/// ```rust
/// use ripress::res::response_headers::ResponseHeaders;
///
/// let headers = ResponseHeaders::new()
///     .with_content_type("application/json")
///     .with_header("X-Custom", "value")
///     .with_cors(Some("https://example.com"))
///     .with_security();
/// ```
///
/// ## Convenience Methods
///
/// ```rust
/// use ripress::res::response_headers::ResponseHeaders;
///
/// let mut headers = ResponseHeaders::new();
///
/// // Content type shortcuts
/// headers.json(); // Sets Content-Type to application/json
/// headers.html(); // Sets Content-Type to text/html; charset=utf-8
///
/// // Security headers
/// headers.security_headers(); // Sets multiple security headers at once
///
/// // CORS setup
/// headers.cors_simple(Some("https://example.com"));
///
/// // File downloads
/// headers.attachment("document.pdf");
/// ```
///
/// ## Multiple Values
///
/// ```rust
/// use ripress::res::response_headers::ResponseHeaders;
///
/// let mut headers = ResponseHeaders::new();
/// headers.insert("Set-Cookie", "session=abc123");
/// headers.append("Set-Cookie", "theme=dark");
///
/// let cookies = headers.get_all("Set-Cookie").unwrap();
/// assert_eq!(cookies.len(), 2);
/// ```
///
/// # HTTP Header Categories
///
/// ## Content Headers
/// - Content-Type, Content-Length, Content-Disposition
/// - Specialized methods: [`json()`](Self::json), [`html()`](Self::html), [`attachment()`](Self::attachment)
///
/// ## Security Headers
/// - X-Frame-Options, X-Content-Type-Options, X-XSS-Protection, CSP
/// - Batch method: [`security_headers()`](Self::security_headers)
///
/// ## CORS Headers
/// - Access-Control-Allow-Origin, Access-Control-Allow-Methods, etc.
/// - Simple setup: [`cors_simple()`](Self::cors_simple)
///
/// ## Caching Headers
/// - Cache-Control, ETag, Last-Modified
/// - No-cache setup: [`no_cache()`](Self::no_cache)
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseHeaders {
    /// Store with lowercase keys for case-insensitive lookup.
    /// Values are Vec<String> to support multiple values for the same header.
    inner: HashMap<String, Vec<String>>,
}

impl ResponseHeaders {
    /// Creates a new empty ResponseHeaders collection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let headers = ResponseHeaders::new();
    /// assert!(headers.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Creates ResponseHeaders from a HashMap<&'static str, &'static str>.
    ///
    /// This method is provided for backward compatibility with legacy APIs
    /// that use static string maps.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("content-type", "application/json");
    /// map.insert("x-custom", "value");
    ///
    /// let headers = ResponseHeaders::from_static_map(map);
    /// assert_eq!(headers.get("content-type"), Some("application/json"));
    /// ```
    pub fn from_static_map(map: HashMap<&'static str, &'static str>) -> Self {
        let mut headers = Self::new();
        for (key, value) in map {
            headers.insert(key, value);
        }
        headers
    }

    /// Inserts a single header value, replacing any existing values.
    ///
    /// The header name is case-insensitive and will be stored in lowercase.
    /// If the header already exists, all previous values are replaced.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.insert("Content-Type", "application/json");
    /// headers.insert("content-type", "text/html"); // Replaces previous value
    ///
    /// assert_eq!(headers.get("content-type"), Some("text/html"));
    /// ```
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str>,
        V: Into<String>,
    {
        let key = key.as_ref().to_lowercase();
        let value = value.into();
        self.inner.insert(key, vec![value]);
    }

    /// Appends a header value, preserving existing values.
    ///
    /// This method supports headers that can have multiple values, such as
    /// `Set-Cookie` or `Accept`. The new value is added to the list of
    /// existing values for the header.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.insert("Set-Cookie", "session=abc123");
    /// headers.append("Set-Cookie", "theme=dark");
    ///
    /// let cookies = headers.get_all("Set-Cookie").unwrap();
    /// assert_eq!(cookies.len(), 2);
    /// assert_eq!(cookies[0], "session=abc123");
    /// assert_eq!(cookies[1], "theme=dark");
    /// ```
    pub fn append<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str>,
        V: Into<String>,
    {
        let key = key.as_ref().to_lowercase();
        let value = value.into();
        self.inner.entry(key).or_default().push(value);
    }

    /// Gets the first value for a header.
    ///
    /// This is the most common use case for header retrieval, as most headers
    /// have only one value. Returns `None` if the header doesn't exist.
    /// Header lookup is case-insensitive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.insert("Content-Type", "application/json");
    ///
    /// assert_eq!(headers.get("content-type"), Some("application/json"));
    /// assert_eq!(headers.get("Content-Type"), Some("application/json")); // Case-insensitive
    /// assert_eq!(headers.get("missing"), None);
    /// ```
    pub fn get<K>(&self, key: K) -> Option<&str>
    where
        K: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        self.inner.get(&key)?.first().map(|s| s.as_str())
    }

    /// Gets all values for a header.
    ///
    /// Returns a reference to the vector of all values for the specified header.
    /// This is useful for headers that can have multiple values, such as `Set-Cookie`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.append("Set-Cookie", "session=abc");
    /// headers.append("Set-Cookie", "theme=dark");
    ///
    /// let cookies = headers.get_all("Set-Cookie").unwrap();
    /// assert_eq!(cookies.len(), 2);
    /// ```
    pub fn get_all<K>(&self, key: K) -> Option<&Vec<String>>
    where
        K: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        self.inner.get(&key)
    }

    /// Checks if a header exists.
    ///
    /// Returns `true` if the header has at least one value, `false` otherwise.
    /// Header lookup is case-insensitive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.insert("Content-Type", "application/json");
    ///
    /// assert!(headers.contains_key("content-type"));
    /// assert!(headers.contains_key("Content-Type")); // Case-insensitive
    /// assert!(!headers.contains_key("missing"));
    /// ```
    pub fn contains_key<K>(&self, key: K) -> bool
    where
        K: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        self.inner.contains_key(&key)
    }

    /// Removes a header completely.
    ///
    /// Removes all values for the specified header and returns them if the header existed.
    /// Header lookup is case-insensitive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.insert("Content-Type", "application/json");
    ///
    /// let removed = headers.remove("content-type");
    /// assert_eq!(removed, Some(vec!["application/json".to_string()]));
    /// assert!(!headers.contains_key("content-type"));
    /// ```
    pub fn remove<K>(&mut self, key: K) -> Option<Vec<String>>
    where
        K: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        self.inner.remove(&key)
    }

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
    pub fn content_type<V>(&mut self, content_type: V)
    where
        V: Into<String>,
    {
        self.insert("content-type", content_type);
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
    pub fn content_length(&mut self, length: u64) {
        self.insert("content-length", length.to_string());
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
    pub fn location<V>(&mut self, url: V)
    where
        V: Into<String>,
    {
        self.insert("location", url);
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
    pub fn cache_control<V>(&mut self, value: V)
    where
        V: Into<String>,
    {
        self.insert("cache-control", value);
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
        self.insert("cache-control", "no-cache, no-store, must-revalidate");
        self.insert("pragma", "no-cache");
        self.insert("expires", "0");
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
    pub fn etag<V>(&mut self, etag: V)
    where
        V: Into<String>,
    {
        self.insert("etag", etag);
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
    pub fn last_modified<V>(&mut self, date: V)
    where
        V: Into<String>,
    {
        self.insert("last-modified", date);
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
    pub fn server<V>(&mut self, server: V)
    where
        V: Into<String>,
    {
        self.insert("server", server);
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
        V: Into<String>,
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
        self.remove("x-powered-by");
    }

    // CORS Headers

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
    pub fn cors_allow_origin<V>(&mut self, origin: V)
    where
        V: Into<String>,
    {
        self.insert("access-control-allow-origin", origin);
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
    pub fn cors_allow_methods<V>(&mut self, methods: V)
    where
        V: Into<String>,
    {
        self.insert("access-control-allow-methods", methods);
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
    pub fn cors_allow_headers<V>(&mut self, headers: V)
    where
        V: Into<String>,
    {
        self.insert("access-control-allow-headers", headers);
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
        self.insert("access-control-allow-credentials", allow.to_string());
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
            None => self.cors_allow_origin("*"),
        }
        self.cors_allow_methods("GET, POST, PUT, DELETE, OPTIONS");
        self.cors_allow_headers("Content-Type, Authorization");
    }

    // Security Headers

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
        V: Into<String>,
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
        let mut value = format!("max-age={}", max_age);
        if include_subdomains {
            value.push_str("; includeSubDomains");
        }
        self.insert("strict-transport-security", value);
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
    pub fn csp<V>(&mut self, policy: V)
    where
        V: Into<String>,
    {
        self.insert("content-security-policy", policy);
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

    // Content Headers

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
    pub fn json(&mut self) {
        self.content_type("application/json");
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
    pub fn html(&mut self) {
        self.content_type("text/html; charset=utf-8");
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
    pub fn text(&mut self) {
        self.content_type("text/plain; charset=utf-8");
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
    pub fn xml(&mut self) {
        self.content_type("application/xml");
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
        self.insert(
            "content-disposition",
            format!("attachment; filename=\"{}\"", filename.into()),
        );
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
        self.insert("content-disposition", "inline");
    }

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
    ///
    /// let keys: Vec<&String> = headers.keys().collect();
    /// assert_eq!(keys.len(), 2);
    /// assert!(keys.contains(&&"content-type".to_string()));
    /// assert!(keys.contains(&&"x-custom".to_string()));
    /// ```
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.inner.keys()
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
    pub fn len(&self) -> usize {
        self.inner.len()
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
    pub fn iter(&self) -> impl Iterator<Item = (&String, &str)> {
        self.inner
            .iter()
            .filter_map(|(k, v)| v.first().map(|first_val| (k, first_val.as_str())))
    }

    /// Returns an iterator over all header name-values pairs, including multiple values.
    ///
    /// This method provides access to all values for headers that can have multiple
    /// values (like Set-Cookie). Each header name is paired with a vector of all
    /// its values.
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
    /// for (name, values) in headers.iter_all() {
    ///     println!("{}: {} values", name, values.len());
    ///     for value in values {
    ///         println!("  {}", value);
    ///     }
    /// }
    /// ```
    pub fn iter_all(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.inner.iter()
    }

    /// Converts the headers to a HashMap with single values.
    ///
    /// For headers with multiple values, only the first value is included
    /// in the resulting HashMap. This is useful for compatibility with APIs
    /// that expect simple key-value mappings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::res::response_headers::ResponseHeaders;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = ResponseHeaders::new();
    /// headers.insert("Content-Type", "application/json");
    /// headers.append("Set-Cookie", "session=abc");
    /// headers.append("Set-Cookie", "theme=dark");
    ///
    /// let map: HashMap<String, String> = headers.to_map();
    /// assert_eq!(map.get("content-type"), Some(&"application/json".to_string()));
    /// assert_eq!(map.get("set-cookie"), Some(&"session=abc".to_string())); // Only first value
    /// ```
    pub fn to_map(&self) -> HashMap<String, String> {
        self.inner
            .iter()
            .filter_map(|(k, v)| v.first().map(|val| (k.clone(), val.clone())))
            .collect()
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
        let mut lines = Vec::new();
        for (key, values) in &self.inner {
            for value in values {
                lines.push(format!("{}: {}", key, value));
            }
        }
        lines
    }
}

impl Default for ResponseHeaders {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ResponseHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, values) in &self.inner {
            for value in values {
                writeln!(f, "{}: {}", key, value)?;
            }
        }
        Ok(())
    }
}

// Convenient indexing syntax: headers["content-type"]
impl std::ops::Index<&str> for ResponseHeaders {
    type Output = str;

    fn index(&self, key: &str) -> &Self::Output {
        self.get(key)
            .unwrap_or_else(|| panic!("Header '{}' not found", key))
    }
}

// Convert from static HashMap for backward compatibility
impl From<HashMap<&'static str, &'static str>> for ResponseHeaders {
    fn from(map: HashMap<&'static str, &'static str>) -> Self {
        Self::from_static_map(map)
    }
}

// Builder pattern integration
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
        V: Into<String>,
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
        V: Into<String>,
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
