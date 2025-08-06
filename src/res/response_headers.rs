use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ResponseHeaders {
    inner: HashMap<String, Vec<String>>,
}

impl ResponseHeaders {
    /// Create a new empty Headers collection
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Create Headers from a HashMap<String, String>
    pub(crate) fn from_map(map: HashMap<String, String>) -> Self {
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
        V: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        let value = value.as_ref().to_string();
        self.inner.insert(key, vec![value]);
    }

    /// Append a header value (supports multiple values)
    pub fn append<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let key = key.as_ref().to_lowercase();
        let value = value.as_ref().to_string();
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

    /// Get Content-Type header
    pub fn content_type(&self) -> Option<&str> {
        self.get("content-type")
    }

    /// Get Authorization header
    pub fn authorization(&self) -> Option<&str> {
        self.get("authorization")
    }

    /// Get User-Agent header
    pub fn user_agent(&self) -> Option<&str> {
        self.get("user-agent")
    }

    /// Get Accept header
    pub fn accept(&self) -> Option<&str> {
        self.get("accept")
    }

    /// Get Host header
    pub fn host(&self) -> Option<&str> {
        self.get("host")
    }

    /// Get X-Forwarded-For header (useful for getting real IP behind proxies)
    pub fn x_forwarded_for(&self) -> Option<&str> {
        self.get("x-forwarded-for")
    }

    /// Check if request accepts JSON
    pub fn accepts_json(&self) -> bool {
        self.accept()
            .map(|accept| accept.contains("application/json") || accept.contains("*/*"))
            .unwrap_or(false)
    }

    /// Check if request accepts HTML
    pub fn accepts_html(&self) -> bool {
        self.accept()
            .map(|accept| accept.contains("text/html") || accept.contains("*/*"))
            .unwrap_or(false)
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
}

impl Default for ResponseHeaders {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ResponseHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
