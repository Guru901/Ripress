use std::collections::HashMap;
use std::fmt;

/// HTTP Response Headers with support for dynamic values and response-specific features
#[derive(Debug, Clone)]
pub struct ResponseHeaders {
    // Store with lowercase keys for case-insensitive lookup
    // Values are Vec<String> to support multiple values for the same header
    inner: HashMap<String, Vec<String>>,
}

impl ResponseHeaders {
    /// Create a new empty ResponseHeaders collection
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Create ResponseHeaders from a HashMap<&'static str, &'static str> (for backward compatibility)
    pub fn from_static_map(map: HashMap<&'static str, &'static str>) -> Self {
        let mut headers = Self::new();
        for (key, value) in map {
            headers.insert(key, value);
        }
        headers
    }

    /// Insert a single header value (replaces existing)
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str>,
        V: Into<String>,
    {
        let key = key.as_ref().to_lowercase();
        let value = value.into();
        self.inner.insert(key, vec![value]);
    }

    /// Append a header value (supports multiple values)
    pub fn append<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str>,
        V: Into<String>,
    {
        let key = key.as_ref().to_lowercase();
        let value = value.into();
        self.inner.entry(key).or_default().push(value);
    }

    /// Get the first value for a header (most common case)
    pub fn get<K>(&self, key: K) -> Option<&str>
    where
        K: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        self.inner.get(&key)?.first().map(|s| s.as_str())
    }

    /// Get all values for a header
    pub fn get_all<K>(&self, key: K) -> Option<&Vec<String>>
    where
        K: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        self.inner.get(&key)
    }

    /// Check if a header exists
    pub fn contains_key<K>(&self, key: K) -> bool
    where
        K: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        self.inner.contains_key(&key)
    }

    /// Remove a header completely
    pub fn remove<K>(&mut self, key: K) -> Option<Vec<String>>
    where
        K: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        self.inner.remove(&key)
    }

    /// Set Content-Type header
    pub fn content_type<V>(&mut self, content_type: V)
    where
        V: Into<String>,
    {
        self.insert("content-type", content_type);
    }

    /// Set Content-Length header
    pub fn content_length(&mut self, length: u64) {
        self.insert("content-length", length.to_string());
    }

    /// Set Location header (for redirects)
    pub fn location<V>(&mut self, url: V)
    where
        V: Into<String>,
    {
        self.insert("location", url);
    }

    /// Set Cache-Control header
    pub fn cache_control<V>(&mut self, value: V)
    where
        V: Into<String>,
    {
        self.insert("cache-control", value);
    }

    /// Set no-cache headers
    pub fn no_cache(&mut self) {
        self.insert("cache-control", "no-cache, no-store, must-revalidate");
        self.insert("pragma", "no-cache");
        self.insert("expires", "0");
    }

    /// Set ETag header
    pub fn etag<V>(&mut self, etag: V)
    where
        V: Into<String>,
    {
        self.insert("etag", etag);
    }

    /// Set Last-Modified header
    pub fn last_modified<V>(&mut self, date: V)
    where
        V: Into<String>,
    {
        self.insert("last-modified", date);
    }

    /// Set Server header
    pub fn server<V>(&mut self, server: V)
    where
        V: Into<String>,
    {
        self.insert("server", server);
    }

    /// Set X-Powered-By header
    pub fn powered_by<V>(&mut self, value: V)
    where
        V: Into<String>,
    {
        self.insert("x-powered-by", value);
    }

    /// Remove X-Powered-By header (security best practice)
    pub fn remove_powered_by(&mut self) {
        self.remove("x-powered-by");
    }

    // CORS Headers

    /// Set Access-Control-Allow-Origin header
    pub fn cors_allow_origin<V>(&mut self, origin: V)
    where
        V: Into<String>,
    {
        self.insert("access-control-allow-origin", origin);
    }

    /// Set Access-Control-Allow-Methods header
    pub fn cors_allow_methods<V>(&mut self, methods: V)
    where
        V: Into<String>,
    {
        self.insert("access-control-allow-methods", methods);
    }

    /// Set Access-Control-Allow-Headers header
    pub fn cors_allow_headers<V>(&mut self, headers: V)
    where
        V: Into<String>,
    {
        self.insert("access-control-allow-headers", headers);
    }

    /// Set Access-Control-Allow-Credentials header
    pub fn cors_allow_credentials(&mut self, allow: bool) {
        self.insert("access-control-allow-credentials", allow.to_string());
    }

    /// Set basic CORS headers for simple requests
    pub fn cors_simple(&mut self, origin: Option<&str>) {
        match origin {
            Some(origin) => self.cors_allow_origin(origin),
            None => self.cors_allow_origin("*"),
        }
        self.cors_allow_methods("GET, POST, PUT, DELETE, OPTIONS");
        self.cors_allow_headers("Content-Type, Authorization");
    }

    // Security Headers

    /// Set X-Frame-Options header (prevents clickjacking)
    pub fn frame_options<V>(&mut self, value: V)
    where
        V: Into<String>,
    {
        self.insert("x-frame-options", value);
    }

    /// Set X-Content-Type-Options header (prevents MIME sniffing)
    pub fn no_sniff(&mut self) {
        self.insert("x-content-type-options", "nosniff");
    }

    /// Set X-XSS-Protection header
    pub fn xss_protection(&mut self, enabled: bool) {
        let value = if enabled { "1; mode=block" } else { "0" };
        self.insert("x-xss-protection", value);
    }

    /// Set Strict-Transport-Security header (HSTS)
    pub fn hsts(&mut self, max_age: u64, include_subdomains: bool) {
        let mut value = format!("max-age={}", max_age);
        if include_subdomains {
            value.push_str("; includeSubDomains");
        }
        self.insert("strict-transport-security", value);
    }

    /// Set Content-Security-Policy header
    pub fn csp<V>(&mut self, policy: V)
    where
        V: Into<String>,
    {
        self.insert("content-security-policy", policy);
    }

    /// Set basic security headers
    pub fn security_headers(&mut self) {
        self.no_sniff();
        self.xss_protection(true);
        self.frame_options("DENY");
        self.remove_powered_by();
    }

    // Content Headers

    /// Set content type to JSON
    pub fn json(&mut self) {
        self.content_type("application/json");
    }

    /// Set content type to HTML
    pub fn html(&mut self) {
        self.content_type("text/html; charset=utf-8");
    }

    /// Set content type to plain text
    pub fn text(&mut self) {
        self.content_type("text/plain; charset=utf-8");
    }

    /// Set content type to XML
    pub fn xml(&mut self) {
        self.content_type("application/xml");
    }

    /// Set headers for file download
    pub fn attachment<V>(&mut self, filename: V)
    where
        V: Into<String>,
    {
        self.insert(
            "content-disposition",
            format!("attachment; filename=\"{}\"", filename.into()),
        );
    }

    /// Set headers for inline file display
    pub fn inline(&mut self) {
        self.insert("content-disposition", "inline");
    }

    /// Get all header names (keys)
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.inner.keys()
    }

    /// Get the number of unique headers
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if headers are empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Iterate over all headers as (key, first_value) pairs
    pub fn iter(&self) -> impl Iterator<Item = (&String, &str)> {
        self.inner
            .iter()
            .filter_map(|(k, v)| v.first().map(|first_val| (k, first_val.as_str())))
    }

    /// Iterate over all headers including multiple values
    pub fn iter_all(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.inner.iter()
    }

    /// Convert to HashMap<String, String> (first value only)
    pub fn to_map(&self) -> HashMap<String, String> {
        self.inner
            .iter()
            .filter_map(|(k, v)| v.first().map(|val| (k.clone(), val.clone())))
            .collect()
    }

    /// Generate all header lines for HTTP response
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
    /// Builder method to set a header and return self
    pub fn with_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: Into<String>,
    {
        self.insert(key, value);
        self
    }

    /// Builder method to set content type and return self
    pub fn with_content_type<V>(mut self, content_type: V) -> Self
    where
        V: Into<String>,
    {
        self.content_type(content_type);
        self
    }

    /// Builder method to set CORS headers and return self
    pub fn with_cors(mut self, origin: Option<&str>) -> Self {
        self.cors_simple(origin);
        self
    }

    /// Builder method to set security headers and return self
    pub fn with_security(mut self) -> Self {
        self.security_headers();
        self
    }
}
