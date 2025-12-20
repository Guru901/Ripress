#![warn(missing_docs)]

use std::ops::Deref;

use crate::helpers::FromRequest;
use hyper::HeaderMap;
use hyper::header::{HeaderName, HeaderValue};

/// A case-insensitive collection of HTTP request headers.
///
/// `RequestHeaders` wraps Hyper's `HeaderMap` to provide a convenient API
/// for working with HTTP headers without unnecessary allocations.
///
/// ## Example
///
/// ```
/// use ripress::req::request_headers::RequestHeaders;
///
/// let mut headers = RequestHeaders::new();
/// headers.insert("Content-Type", "application/json");
/// headers.append("Set-Cookie", "id=123");
/// headers.append("Set-Cookie", "theme=dark");
///
/// assert_eq!(headers.content_type(), Some("application/json"));
/// assert_eq!(headers.get_all("set-cookie").len(), 2);
/// ```

#[derive(Debug, Clone)]
pub struct RequestHeaders {
    inner: HeaderMap,
}

impl RequestHeaders {
    /// Creates a new, empty `RequestHeaders` collection.
    ///
    /// # Example
    /// ```
    /// use ripress::req::request_headers::RequestHeaders;
    ///
    /// let headers = RequestHeaders::new();
    /// assert!(headers.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            inner: HeaderMap::new(),
        }
    }

    /// Creates a `RequestHeaders` instance with pre-allocated capacity.
    pub(crate) fn _with_capacity(capacity: usize) -> Self {
        Self {
            inner: HeaderMap::with_capacity(capacity),
        }
    }

    /// Creates a `RequestHeaders` directly from Hyper's HeaderMap (zero-cost).
    pub(crate) fn from_header_map(map: HeaderMap) -> Self {
        Self { inner: map }
    }

    /// Inserts a header value, replacing any existing values for the header name.
    ///
    /// Header names are case-insensitive.
    ///
    /// # Example
    /// ```
    /// use ripress::req::request_headers::RequestHeaders;
    ///
    /// let mut headers = RequestHeaders::new();
    /// headers.insert("Content-Type", "application/json");
    /// assert_eq!(headers.content_type(), Some("application/json"));
    /// ```
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(key.as_ref().as_bytes()),
            HeaderValue::from_bytes(value.as_ref().as_bytes()),
        ) {
            self.inner.insert(name, val);
        }
    }

    /// Appends a value to an existing header or creates it if not present.
    ///
    /// Useful for headers that allow multiple values, such as `Set-Cookie` or `Accept`.
    ///
    /// # Example
    /// ```
    /// use ripress::req::request_headers::RequestHeaders;
    ///
    /// let mut headers = RequestHeaders::new();
    /// headers.append("Set-Cookie", "id=1");
    /// headers.append("Set-Cookie", "theme=dark");
    /// assert_eq!(headers.get_all("Set-Cookie").len(), 2);
    /// ```
    pub fn append<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(key.as_ref().as_bytes()),
            HeaderValue::from_bytes(value.as_ref().as_bytes()),
        ) {
            self.inner.append(name, val);
        }
    }

    /// Returns the **first** value for the given header name, if present.
    ///
    /// # Example
    /// ```
    /// use ripress::req::request_headers::RequestHeaders;
    ///
    /// let mut headers = RequestHeaders::new();
    /// headers.append("Accept", "application/json");
    /// headers.append("Accept", "text/html");
    /// assert_eq!(headers.get("Accept"), Some("application/json"));
    /// ```
    pub fn get<K>(&self, key: K) -> Option<&str>
    where
        K: AsRef<str>,
    {
        let name = HeaderName::from_bytes(key.as_ref().as_bytes()).ok()?;
        self.inner.get(&name)?.to_str().ok()
    }

    /// Returns **all values** for the given header name.
    ///
    /// # Example
    /// ```
    /// use ripress::req::request_headers::RequestHeaders;
    ///
    /// let mut headers = RequestHeaders::new();
    /// headers.append("Accept", "application/json");
    /// headers.append("Accept", "text/html");
    /// assert_eq!(headers.get_all("Accept").len(), 2);
    /// ```
    pub fn get_all<K>(&self, key: K) -> Vec<&str>
    where
        K: AsRef<str>,
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

    /// Checks whether a header exists.
    pub fn contains_key<K>(&self, key: K) -> bool
    where
        K: AsRef<str>,
    {
        HeaderName::from_bytes(key.as_ref().as_bytes())
            .ok()
            .and_then(|name| self.inner.get(&name))
            .is_some()
    }

    /// Removes a header entirely, returning its first value if present.
    pub fn remove<K>(&mut self, key: K) -> Option<String>
    where
        K: AsRef<str>,
    {
        let name: HeaderName = HeaderName::from_bytes(key.as_ref().as_bytes()).ok()?;
        self.inner.remove(&name)?.to_str().ok().map(String::from)
    }

    /// Returns the value of the `Content-Type` header, if present.
    pub fn content_type(&self) -> Option<&str> {
        self.get("content-type")
    }

    /// Returns the value of the `Authorization` header, if present.
    pub fn authorization(&self) -> Option<&str> {
        self.get("authorization")
    }

    /// Returns the value of the `User-Agent` header, if present.
    pub fn user_agent(&self) -> Option<&str> {
        self.get("user-agent")
    }

    /// Returns the value of the `Accept` header, if present.
    pub fn accept(&self) -> Option<&str> {
        self.get("accept")
    }

    /// Returns the value of the `Host` header, if present.
    pub fn host(&self) -> Option<&str> {
        self.get("host")
    }

    /// Returns the value of the `X-Forwarded-For` header, if present.
    ///
    /// This can be useful for retrieving the real IP address of a client
    /// behind proxies.
    pub fn x_forwarded_for(&self) -> Option<&str> {
        self.get("x-forwarded-for")
    }

    /// Returns `true` if the `Accept` header indicates the client accepts JSON.
    ///
    /// Matches if the `Accept` header contains `application/json` or `*/*`.
    pub fn accepts_json(&self) -> bool {
        self.accept()
            .map(|accept| accept.contains("application/json") || accept.contains("*/*"))
            .unwrap_or(false)
    }

    /// Returns `true` if the `Accept` header indicates the client accepts HTML.
    ///
    /// Matches if the `Accept` header contains `text/html` or `*/*`.
    pub fn accepts_html(&self) -> bool {
        self.accept()
            .map(|accept| accept.contains("text/html") || accept.contains("*/*"))
            .unwrap_or(false)
    }

    /// Returns an iterator over all header names.
    ///
    /// # Example
    /// ```
    /// use ripress::req::request_headers::RequestHeaders;
    ///
    /// let mut headers = RequestHeaders::new();
    /// headers.insert("Content-Type", "application/json");
    /// for key in headers.keys() {
    ///     println!("{}", key);
    /// }
    /// ```
    pub fn keys(&self) -> impl Iterator<Item = &HeaderName> {
        self.inner.keys()
    }

    /// Returns the number of unique header names.
    pub fn len(&self) -> usize {
        self.inner.keys().len()
    }

    /// Returns `true` if there are no headers.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Iterates over all headers as `(name, first_value)` pairs.
    ///
    /// Useful when you only need the first value for each header.
    pub fn iter(&self) -> impl Iterator<Item = (&HeaderName, &HeaderValue)> {
        self.inner.iter()
    }

    /// Iterates over all headers as `(name, value)` pairs, including duplicates.
    pub fn iter_all(&self) -> impl Iterator<Item = (&HeaderName, &str)> {
        self.inner
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|val| (k, val)))
    }

    /// Returns a reference to the inner HeaderMap for advanced usage.
    pub fn as_header_map(&self) -> &HeaderMap {
        &self.inner
    }

    /// Consumes self and returns the inner HeaderMap.
    pub fn into_header_map(self) -> HeaderMap {
        self.inner
    }
}

impl Default for RequestHeaders {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RequestHeaders {
    /// Formats the headers as `key: value` lines.
    ///
    /// # Example
    /// ```
    /// use ripress::req::request_headers::RequestHeaders;
    ///
    /// let mut headers = RequestHeaders::new();
    /// headers.insert("Content-Type", "application/json");
    /// println!("{}", headers);
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.iter() {
            writeln!(f, "{}: {:?}", key, value)?;
        }
        Ok(())
    }
}

impl std::ops::Index<&str> for RequestHeaders {
    type Output = str;

    /// Provides convenient indexing syntax:
    ///
    /// ```
    /// use ripress::req::request_headers::RequestHeaders;
    ///
    /// let mut headers = RequestHeaders::new();
    /// headers.insert("Content-Type", "application/json");
    /// assert_eq!(&headers["content-type"], "application/json");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the header does not exist.
    fn index(&self, key: &str) -> &Self::Output {
        self.get(key)
            .unwrap_or_else(|| panic!("Header '{}' not found", key))
    }
}

impl From<HeaderMap> for RequestHeaders {
    fn from(map: HeaderMap) -> Self {
        Self::from_header_map(map)
    }
}

impl From<RequestHeaders> for HeaderMap {
    fn from(headers: RequestHeaders) -> Self {
        headers.into_header_map()
    }
}

/// A wrapper around [`RequestHeaders`] that allows extracting headers from an [`HttpRequest`] using the [`FromRequest`] trait.
///
/// This enables ergonomic extraction of headers as a parameter in route handlers:
/// ```
/// pub fn handler(headers: Headers) {
///    for (key, value) in headers.iter() {
///         println!("{}: {}", key, value);
///     }
/// }
/// ```
///
pub struct Headers(RequestHeaders);

impl FromRequest for Headers {
    type Error = String;

    fn from_request(req: &super::HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self(req.headers.clone()))
    }
}

impl Deref for Headers {
    type Target = RequestHeaders;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
