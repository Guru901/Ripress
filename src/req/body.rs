use std::{collections::HashMap, fmt::Display};
use urlencoding::decode;

/// A convenient wrapper around `HashMap<String, String>` for handling form data.
///
/// This struct provides a clean interface for working with key-value pairs commonly
/// found in HTML forms, query parameters, and similar data structures.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use ripress::req::body::FormData;
///
/// let mut form = FormData::new();
/// form.insert("username", "alice");
/// form.insert("email", "alice@example.com");
///
/// assert_eq!(form.get("username"), Some("alice"));
/// assert_eq!(&form["email"], "alice@example.com");
///
/// // Convert from HashMap
/// let mut map = HashMap::new();
/// map.insert("key".to_string(), "value".to_string());
/// let form = FormData::from(map);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormData {
    pub(crate) inner: HashMap<String, String>,
}

impl FormData {
    /// Creates a new, empty `FormData` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let form = FormData::new();
    /// assert!(form.is_empty());
    /// ```
    pub fn new() -> Self {
        FormData {
            inner: HashMap::new(),
        }
    }

    /// Creates a new `FormData` with the specified capacity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let form = FormData::with_capacity(10);
    /// assert!(form.is_empty());
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        FormData {
            inner: HashMap::with_capacity(capacity),
        }
    }

    /// Inserts a key-value pair into the form data.
    ///
    /// If the key already exists, the old value is returned and replaced.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// assert_eq!(form.insert("key", "value"), None);
    /// assert_eq!(form.insert("key", "new_value"), Some("value".to_string()));
    /// ```
    pub fn insert<K, V>(&mut self, key: K, value: V) -> Option<String>
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.inner.insert(key.into(), value.into())
    }

    /// Gets a reference to the value associated with the key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.insert("key", "value");
    /// assert_eq!(form.get("key"), Some("value"));
    /// assert_eq!(form.get("missing"), None);
    /// ```
    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|s| s.as_str())
    }

    /// Gets a reference to the value associated with the key, or a default value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.insert("key", "value");
    /// assert_eq!(form.get_or("key", "default"), "value");
    /// assert_eq!(form.get_or("missing", "default"), "default");
    /// ```
    pub fn get_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).unwrap_or(default)
    }

    /// Gets a mutable reference to the value associated with the key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.insert("key", "value");
    /// if let Some(value) = form.get_mut("key") {
    ///     value.push_str("_modified");
    /// }
    /// assert_eq!(form.get("key"), Some("value_modified"));
    /// ```
    pub fn get_mut(&mut self, key: &str) -> Option<&mut String> {
        self.inner.get_mut(key)
    }

    /// Returns a reference to the underlying HashMap.
    ///
    /// This method name is more descriptive than `get_all`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let form = FormData::new();
    /// let map = form.as_map();
    /// assert!(map.is_empty());
    /// ```
    pub fn as_map(&self) -> &HashMap<String, String> {
        &self.inner
    }

    /// Returns an iterator over the keys.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.insert("a", "1");
    /// form.insert("b", "2");
    ///
    /// let keys: Vec<_> = form.keys().collect();
    /// assert_eq!(keys.len(), 2);
    /// ```
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.inner.keys().map(|s| s.as_str())
    }

    /// Returns an iterator over the values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.insert("a", "1");
    /// form.insert("b", "2");
    ///
    /// let values: Vec<_> = form.values().collect();
    /// assert_eq!(values.len(), 2);
    /// ```
    pub fn values(&self) -> impl Iterator<Item = &str> {
        self.inner.values().map(|s| s.as_str())
    }

    /// Returns the number of key-value pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// assert_eq!(form.len(), 0);
    /// form.insert("key", "value");
    /// assert_eq!(form.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the form data contains no key-value pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// assert!(form.is_empty());
    /// form.insert("key", "value");
    /// assert!(!form.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Removes and returns the value associated with the key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.insert("key", "value");
    /// assert_eq!(form.remove("key"), Some("value".to_string()));
    /// assert_eq!(form.remove("key"), None);
    /// ```
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.inner.remove(key)
    }

    /// Returns `true` if the form data contains the specified key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.insert("key", "value");
    /// assert!(form.contains_key("key"));
    /// assert!(!form.contains_key("missing"));
    /// ```
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// Creates a `FormData` from an existing HashMap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use ripress::req::body::FormData;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("key".to_string(), "value".to_string());
    /// let form = FormData::from_map(map);
    /// assert_eq!(form.get("key"), Some("value"));
    /// ```
    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self { inner: map }
    }

    /// Returns an iterator over key-value pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.insert("a", "1");
    /// form.insert("b", "2");
    ///
    /// for (key, value) in form.iter() {
    ///     println!("{}: {}", key, value);
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Clears all key-value pairs from the form data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.insert("key", "value");
    /// form.clear();
    /// assert!(form.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Retains only the key-value pairs that satisfy the predicate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.insert("keep", "yes");
    /// form.insert("remove", "no");
    ///
    /// form.retain(|key, _| key.starts_with("keep"));
    /// assert!(form.contains_key("keep"));
    /// assert!(!form.contains_key("remove"));
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&str, &str) -> bool,
    {
        self.inner.retain(|k, v| f(k, v));
    }

    /// Extends the form data with key-value pairs from an iterator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// let pairs = vec![("a", "1"), ("b", "2")];
    /// form.extend(pairs);
    /// assert_eq!(form.len(), 2);
    /// ```
    pub fn extend<I, K, V>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.inner
            .extend(iter.into_iter().map(|(k, v)| (k.into(), v.into())));
    }

    /// Appends or updates a value, creating a comma-separated list if the key already exists.
    ///
    /// This is useful for form fields that can have multiple values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.append("tags", "rust");
    /// form.append("tags", "web");
    /// assert_eq!(form.get("tags"), Some("rust,web"));
    /// ```
    pub fn append<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        let key_string = key.into();
        let value_string = value.into();

        match self.inner.get_mut(&key_string) {
            Some(existing) => {
                existing.push(',');
                existing.push_str(&value_string);
            }
            None => {
                self.inner.insert(key_string, value_string);
            }
        }
    }

    /// Converts the form data to a URL-encoded query string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.insert("name", "John Doe");
    /// form.insert("age", "30");
    ///
    /// let query = form.to_query_string();
    /// // Order may vary due to HashMap
    /// assert!(query.contains("name=John%20Doe"));
    /// assert!(query.contains("age=30"));
    /// ```
    pub fn to_query_string(&self) -> String {
        self.inner
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    }

    ///
    /// Parses a URL-encoded query string into form data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let form = FormData::from_query_string("name=John%20Doe&age=30").unwrap();
    /// assert_eq!(form.get("name"), Some("John Doe"));
    /// assert_eq!(form.get("age"), Some("30"));
    /// ```
    pub fn from_query_string(query: &str) -> Result<Self, String> {
        let mut form_data = FormData::new();

        if query.is_empty() {
            return Ok(form_data);
        }

        if query.contains(", ") && !query.contains("&") {
            return Self::from_comma_separated(query);
        }

        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let decoded_key =
                    decode(key).map_err(|e| format!("Failed to decode key '{}': {}", key, e))?;
                let decoded_value = decode(value)
                    .map_err(|e| format!("Failed to decode value '{}': {}", value, e))?;

                form_data.insert(decoded_key.into_owned(), decoded_value.into_owned());
            } else {
                // Handle key without value
                let decoded_key =
                    decode(pair).map_err(|e| format!("Failed to decode key '{}': {}", pair, e))?;
                form_data.insert(decoded_key.into_owned(), String::new());
            }
        }
        Ok(form_data)
    }

    pub fn from_comma_separated(query: &str) -> Result<Self, String> {
        let mut form_data = FormData::new();

        if query.is_empty() {
            return Ok(form_data);
        }

        // Try comma separation first, then fall back to ampersand
        let separator = if query.contains(", ") { ", " } else { "&" };

        for pair in query.split(separator) {
            let pair = pair.trim(); // Remove any extra whitespace
            if let Some((key, value)) = pair.split_once('=') {
                let decoded_key = decode(key.trim())
                    .map_err(|e| format!("Failed to decode key '{}': {}", key, e))?;
                let decoded_value = decode(value.trim())
                    .map_err(|e| format!("Failed to decode value '{}': {}", value, e))?;
                form_data.insert(decoded_key.into_owned(), decoded_value.into_owned());
            }
        }

        Ok(form_data)
    }
}

impl Default for FormData {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for FormData {
    /// Formats the form data as a comma-separated list of key=value pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.insert("name", "Alice");
    /// form.insert("age", "30");
    ///
    /// let display = format!("{}", form);
    /// // Order may vary due to HashMap
    /// assert!(display.contains("name=Alice"));
    /// assert!(display.contains("age=30"));
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let form_data_strings: Vec<String> = self
            .inner
            .iter()
            .map(|(name, value)| format!("{}={}", name, value))
            .collect();
        write!(f, "{}", form_data_strings.join(", "))
    }
}

impl From<HashMap<String, String>> for FormData {
    fn from(map: HashMap<String, String>) -> Self {
        Self::from_map(map)
    }
}

impl From<FormData> for HashMap<String, String> {
    fn from(form_data: FormData) -> Self {
        form_data.inner
    }
}

impl<K, V> FromIterator<(K, V)> for FormData
where
    K: Into<String>,
    V: Into<String>,
{
    /// Creates a `FormData` from an iterator of key-value pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let pairs = vec![("name", "Alice"), ("age", "30")];
    /// let form: FormData = pairs.into_iter().collect();
    /// assert_eq!(form.len(), 2);
    /// ```
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut form_data = FormData::new();
        form_data.extend(iter);
        form_data
    }
}

impl<K, V> Extend<(K, V)> for FormData
where
    K: Into<String>,
    V: Into<String>,
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        FormData::extend(self, iter);
    }
}

impl std::ops::Index<&str> for FormData {
    type Output = str;

    /// Provides convenient indexing syntax for accessing form data values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form_data = FormData::new();
    /// form_data.insert("username", "alice");
    /// assert_eq!(&form_data["username"], "alice");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the key does not exist. Use [`get`](Self::get) for safe access.
    fn index(&self, key: &str) -> &Self::Output {
        self.get(key)
            .unwrap_or_else(|| panic!("FormData parameter '{}' not found", key))
    }
}

impl IntoIterator for FormData {
    type Item = (String, String);
    type IntoIter = std::collections::hash_map::IntoIter<String, String>;

    /// Converts the `FormData` into an iterator of owned key-value pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::FormData;
    ///
    /// let mut form = FormData::new();
    /// form.insert("a", "1");
    /// form.insert("b", "2");
    ///
    /// for (key, value) in form {
    ///     println!("{}: {}", key, value);
    /// }
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a FormData {
    type Item = (&'a String, &'a String);
    type IntoIter = std::collections::hash_map::Iter<'a, String, String>;

    /// Creates an iterator over references to key-value pairs.
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a mut FormData {
    type Item = (&'a String, &'a mut String);
    type IntoIter = std::collections::hash_map::IterMut<'a, String, String>;

    /// Creates an iterator over references to key-value pairs.
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}
