#![warn(missing_docs)]
use std::collections::HashMap;
use std::fmt;

use hyper::HeaderMap;
use hyper::header::{HeaderName, HeaderValue};

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

    #[inline]
    pub fn content_type<V>(&mut self, content_type: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(content_type.as_ref().as_bytes()) {
            self.inner.insert(hyper::header::CONTENT_TYPE, val);
        }
    }

    #[inline]
    pub fn content_length(&mut self, length: u64) {
        if let Ok(val) = HeaderValue::from_str(&length.to_string()) {
            self.inner.insert(hyper::header::CONTENT_LENGTH, val);
        }
    }

    #[inline]
    pub fn location<V>(&mut self, url: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(url.as_ref().as_bytes()) {
            self.inner.insert(hyper::header::LOCATION, val);
        }
    }

    #[inline]
    pub fn cache_control<V>(&mut self, value: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(value.as_ref().as_bytes()) {
            self.inner.insert(hyper::header::CACHE_CONTROL, val);
        }
    }

    pub fn no_cache(&mut self) {
        let val = HeaderValue::from_static("no-cache, no-store, must-revalidate");
        self.inner.insert(hyper::header::CACHE_CONTROL, val);

        let val = HeaderValue::from_static("no-cache");
        self.inner.insert(hyper::header::PRAGMA, val);

        let val = HeaderValue::from_static("0");
        self.inner.insert(hyper::header::EXPIRES, val);
    }

    #[inline]
    pub fn etag<V>(&mut self, etag: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(etag.as_ref().as_bytes()) {
            self.inner.insert(hyper::header::ETAG, val);
        }
    }

    #[inline]
    pub fn last_modified<V>(&mut self, date: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(date.as_ref().as_bytes()) {
            self.inner.insert(hyper::header::LAST_MODIFIED, val);
        }
    }

    #[inline]
    pub fn server<V>(&mut self, server: V)
    where
        V: AsRef<str>,
    {
        if let Ok(val) = HeaderValue::from_bytes(server.as_ref().as_bytes()) {
            self.inner.insert(hyper::header::SERVER, val);
        }
    }

    pub fn powered_by<V>(&mut self, value: V)
    where
        V: AsRef<str>,
    {
        self.insert("x-powered-by", value);
    }

    pub fn remove_powered_by(&mut self) {
        let name = HeaderName::from_static("x-powered-by");
        self.inner.remove(&name);
    }

    // === CORS Headers ===

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

    pub fn cors_allow_credentials(&mut self, allow: bool) {
        let value = if allow { "true" } else { "false" };
        let val = HeaderValue::from_static(value);
        self.inner
            .insert(hyper::header::ACCESS_CONTROL_ALLOW_CREDENTIALS, val);
    }

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

    pub fn frame_options<V>(&mut self, value: V)
    where
        V: AsRef<str>,
    {
        self.insert("x-frame-options", value);
    }

    pub fn no_sniff(&mut self) {
        self.insert("x-content-type-options", "nosniff");
    }

    pub fn xss_protection(&mut self, enabled: bool) {
        let value = if enabled { "1; mode=block" } else { "0" };
        self.insert("x-xss-protection", value);
    }

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

    pub fn security_headers(&mut self) {
        self.no_sniff();
        self.xss_protection(true);
        self.frame_options("DENY");
        self.remove_powered_by();
    }

    // === Content Type Shortcuts ===

    #[inline]
    pub fn json(&mut self) {
        let val = HeaderValue::from_static("application/json");
        self.inner.insert(hyper::header::CONTENT_TYPE, val);
    }

    #[inline]
    pub fn html(&mut self) {
        let val = HeaderValue::from_static("text/html; charset=utf-8");
        self.inner.insert(hyper::header::CONTENT_TYPE, val);
    }

    #[inline]
    pub fn text(&mut self) {
        let val = HeaderValue::from_static("text/plain; charset=utf-8");
        self.inner.insert(hyper::header::CONTENT_TYPE, val);
    }

    #[inline]
    pub fn xml(&mut self) {
        let val = HeaderValue::from_static("application/xml");
        self.inner.insert(hyper::header::CONTENT_TYPE, val);
    }

    pub fn attachment<V>(&mut self, filename: V)
    where
        V: Into<String>,
    {
        let value = format!("attachment; filename=\"{}\"", filename.into());
        if let Ok(val) = HeaderValue::from_str(&value) {
            self.inner.insert(hyper::header::CONTENT_DISPOSITION, val);
        }
    }

    pub fn inline(&mut self) {
        let val = HeaderValue::from_static("inline");
        self.inner.insert(hyper::header::CONTENT_DISPOSITION, val);
    }

    // === Utility Methods ===

    pub fn keys(&self) -> Vec<&HeaderName> {
        self.inner.keys().collect()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.keys().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|val| (k.as_str(), val)))
    }

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

// === Builder Pattern ===

impl ResponseHeaders {
    pub fn with_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.insert(key, value);
        self
    }

    pub fn with_content_type<V>(mut self, content_type: V) -> Self
    where
        V: AsRef<str>,
    {
        self.content_type(content_type);
        self
    }

    pub fn with_cors(mut self, origin: Option<&str>) -> Self {
        self.cors_simple(origin);
        self
    }

    pub fn with_security(mut self) -> Self {
        self.security_headers();
        self
    }
}
