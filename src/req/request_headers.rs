#![warn(missing_docs)]
use std::collections::HashMap;

/// A case-insensitive collection of HTTP request headers.
///
/// `RequestHeaders` stores HTTP request header names and their values in a
/// normalized (lowercased) form. It supports multiple values per header and
/// provides helper methods for commonly used headers.
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
/// assert_eq!(headers.get_all("set-cookie").unwrap().len(), 2);
/// ```

#[derive(Debug, Clone)]
pub struct RequestHeaders {
    inner: HashMap<String, Vec<String>>,
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
            inner: HashMap::new(),
        }
    }

    /// Creates a `RequestHeaders` instance with pre-allocated capacity.
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    /// Creates a `RequestHeaders` instance from a `HashMap<String, String>`.
    ///
    /// Each header in the map will be stored with a single value.
    /// Primarily intended for internal use.

    pub(crate) fn _from_map(map: HashMap<String, String>) -> Self {
        let mut headers = Self::with_capacity(map.len());
        for (key, value) in map {
            headers.insert(key, value);
        }
        headers
    }

    /// Inserts a header value, replacing any existing values for the header name.
    ///
    /// Header names are stored in lowercase for case-insensitive matching.
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
        let key = key.as_ref().to_lowercase();
        let value = value.as_ref().to_string();
        self.inner.insert(key, vec![value]);
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
    /// assert_eq!(headers.get_all("Set-Cookie").unwrap().len(), 2);
    /// ```

    pub fn append<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        let value = value.as_ref().to_string();
        self.inner.entry(key).or_default().push(value);
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
        let key = key.as_ref().to_lowercase();
        self.inner.get(&key)?.first().map(|s| s.as_str())
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
    /// assert_eq!(headers.get_all("Accept").unwrap().len(), 2);
    /// ```

    pub fn get_all<K>(&self, key: K) -> Option<&Vec<String>>
    where
        K: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        self.inner.get(&key)
    }

    /// Checks whether a header exists.
    pub fn contains_key<K>(&self, key: K) -> bool
    where
        K: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        self.inner.contains_key(&key)
    }

    /// Removes a header entirely, returning its values if present.
    pub fn remove<K>(&mut self, key: K) -> Option<Vec<String>>
    where
        K: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        self.inner.remove(&key)
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

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.inner.keys()
    }

    /// Returns the number of unique header names.

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if there are no headers.

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Iterates over all headers as `(name, first_value)` pairs.
    ///
    /// Useful when you only need the first value for each header.

    pub fn iter(&self) -> impl Iterator<Item = (&String, &str)> {
        self.inner
            .iter()
            .filter_map(|(k, v)| v.first().map(|first_val| (k, first_val.as_str())))
    }

    /// Iterates over all headers as `(name, all_values)` pairs.
    pub fn iter_all(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.inner.iter()
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
        for (key, values) in &self.inner {
            for value in values {
                writeln!(f, "{}: {}", key, value)?;
            }
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
