#![warn(missing_docs)]
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
    ops::Deref,
};

use ahash::AHashMap;

use crate::helpers::FromRequest;

/// A high-performance wrapper around bytes that implements Hash and Eq
/// by comparing the underlying byte content.
///
/// `ByteKey` is designed for use as a HashMap key when you need to store
/// arbitrary byte data efficiently. It provides UTF-8 string conversion
/// methods while maintaining the ability to store non-UTF-8 data.
///
/// # Examples
///
/// ```rust
/// use ripress::req::request_data::ByteKey;
///
/// // Create from string
/// let key1 = ByteKey::new("hello");
/// let key2 = ByteKey::new(b"world");
///
/// // Use as HashMap key
/// use std::collections::HashMap;
/// let mut map = HashMap::new();
/// map.insert(key1, "value1");
/// map.insert(key2, "value2");
///
/// // Convert back to string if valid UTF-8
/// let key = ByteKey::new("test");
/// if let Ok(s) = key.as_str() {
///     println!("Key as string: {}", s);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ByteKey(Vec<u8>);

impl ByteKey {
    /// Creates a new `ByteKey` from any data that can be referenced as bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::ByteKey;
    ///
    /// let key1 = ByteKey::new("string");
    /// let key2 = ByteKey::new(b"bytes");
    /// let key3 = ByteKey::new(vec![1, 2, 3, 4]);
    /// ```

    pub fn new(data: impl AsRef<[u8]>) -> Self {
        Self(data.as_ref().to_vec())
    }

    /// Returns a reference to the underlying bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::ByteKey;
    ///
    /// let key = ByteKey::new("hello");
    /// assert_eq!(key.as_bytes(), b"hello");
    /// ```

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Attempts to convert the bytes to a string slice.
    ///
    /// Returns an error if the bytes are not valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::ByteKey;
    ///
    /// let key = ByteKey::new("hello");
    /// assert_eq!(key.as_str().unwrap(), "hello");
    ///
    /// let invalid_key = ByteKey::new(&[0xFF, 0xFE]);
    /// assert!(invalid_key.as_str().is_err());
    /// ```

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0)
    }
}

impl Hash for ByteKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq for ByteKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ByteKey {}

impl AsRef<[u8]> for ByteKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Display for ByteKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match std::str::from_utf8(&self.0) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "{:?}", self.0), // Show as byte array if not valid UTF-8
        }
    }
}

/// High-performance request data structure optimized for byte storage.
///
/// `RequestData` is a specialized HashMap-like container that stores arbitrary
/// byte data using `ByteKey` for keys and `Vec<u8>` for values. It's designed
/// for scenarios where you need to handle mixed text and binary data efficiently,
/// such as HTTP request processing, form data handling, or general key-value
/// byte storage.
///
/// # Features
///
/// - **Flexible keys**: Accepts any data that can be referenced as bytes
/// - **Flexible values**: Stores arbitrary byte data
/// - **UTF-8 aware**: Provides string conversion methods with fallbacks
/// - **Memory efficient**: Minimizes allocations and provides capacity management
/// - **Iteration support**: Multiple ways to iterate over data
/// - **Size tracking**: Methods to monitor memory usage
///
/// # Examples
///
/// ```
/// use ripress::req::request_data::RequestData;
///
/// let mut data = RequestData::new();
///
/// // Insert text data
/// data.insert("name", "John Doe");
/// data.insert("email", "john@example.com");
///
/// // Insert binary data
/// data.insert("image", &[0xFF, 0xD8, 0xFF, 0xE0]); // JPEG header
///
/// // Retrieve data
/// if let Some(name) = data.get("name") {
///     println!("Name: {}", name);
/// }
///
/// // Check size and optimize memory
/// println!("Total size: {} bytes", data.byte_size());
/// data.shrink_to_fit();
/// ```

#[derive(Clone, Debug, Default)]
pub struct RequestData {
    inner: AHashMap<ByteKey, Vec<u8>>,
}

impl Display for RequestData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RequestData {{ ")?;
        let mut first = true;
        for (k, v) in &self.inner {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}: ", k)?;
            // Try to display value as string, fallback to byte array
            match std::str::from_utf8(v) {
                Ok(s) => write!(f, "{}", s)?,
                Err(_) => write!(f, "{:?}", v)?,
            }
            first = false;
        }
        write!(f, " }}")
    }
}

impl RequestData {
    /// Creates a new empty `RequestData` collection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let data = RequestData::new();
    /// assert!(data.is_empty());
    /// ```

    pub fn new() -> Self {
        Self {
            inner: AHashMap::new(),
        }
    }

    /// Creates a new empty `RequestData` collection with the specified capacity.
    ///
    /// The container will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the container will not allocate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let data = RequestData::with_capacity(10);
    /// assert!(data.is_empty());
    /// ```

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: AHashMap::with_capacity(capacity),
        }
    }

    /// Insert data - takes anything that can be converted to bytes.
    ///
    /// Replaces any existing value for the given key.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut data = RequestData::new();
    /// data.insert("key1", "string value");
    /// data.insert("key2", b"byte array");
    /// data.insert("key3", vec![1, 2, 3, 4]);
    /// ```

    pub fn insert(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        let key = ByteKey::new(key);
        let value = value.as_ref().to_vec();
        self.inner.insert(key, value);
    }

    /// Insert without copying if you already have owned data.
    ///
    /// This method avoids allocating new Vec instances if you already
    /// have owned byte vectors. More efficient than `insert` when you
    /// have `Vec<u8>` data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut data = RequestData::new();
    /// let key = b"my_key".to_vec();
    /// let value = b"my_value".to_vec();
    /// data.insert_owned(key, value);
    /// ```

    pub fn insert_owned(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.inner.insert(ByteKey(key), value);
    }

    /// Get value as a UTF-8 string.
    ///
    /// Returns `Some(String)` if the key exists and the value is valid UTF-8,
    /// otherwise returns `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut data = RequestData::new();
    /// data.insert("name", "John");
    ///
    /// assert_eq!(data.get("name"), Some("John".to_string()));
    /// assert_eq!(data.get("missing"), None);
    /// ```

    pub fn get(&self, key: impl AsRef<[u8]>) -> Option<String> {
        let key = ByteKey::new(key);
        self.inner
            .get(&key)
            .and_then(|data| String::from_utf8(data.clone()).ok())
    }

    /// Remove and return the value as raw bytes.
    ///
    /// Returns the removed value if the key existed, otherwise returns `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut data = RequestData::new();
    /// data.insert("temp", "temporary");
    ///
    /// let removed = data.remove("temp");
    /// assert_eq!(removed, Some(b"temporary".to_vec()));
    /// assert!(data.is_empty());
    /// ```

    pub fn remove(&mut self, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
        let key = ByteKey::new(key);
        self.inner.remove(&key)
    }

    /// Returns `true` if the data contains a value for the specified key.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut data = RequestData::new();
    /// data.insert("key", "value");
    ///
    /// assert!(data.contains_key("key"));
    /// assert!(!data.contains_key("missing"));
    /// ```

    pub fn contains_key(&self, key: impl AsRef<[u8]>) -> bool {
        let key = ByteKey::new(key);
        self.inner.contains_key(&key)
    }

    /// Returns the number of key-value pairs in the data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut data = RequestData::new();
    /// assert_eq!(data.len(), 0);
    ///
    /// data.insert("key1", "value1");
    /// data.insert("key2", "value2");
    /// assert_eq!(data.len(), 2);
    /// ```

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the data contains no key-value pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut data = RequestData::new();
    /// assert!(data.is_empty());
    ///
    /// data.insert("key", "value");
    /// assert!(!data.is_empty());
    /// ```

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clears the data, removing all key-value pairs.
    ///
    /// Keeps the allocated memory for reuse.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut data = RequestData::new();
    /// data.insert("key", "value");
    ///
    /// data.clear();
    /// assert!(data.is_empty());
    /// ```

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns an iterator over key-value pairs as byte slices.
    ///
    /// The iterator yields `(&[u8], &[u8])` pairs where the first element
    /// is the key bytes and the second is the value bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut data = RequestData::new();
    /// data.insert("key1", "value1");
    /// data.insert("key2", "value2");
    ///
    /// for (key, value) in data.iter() {
    ///     println!("Key: {:?}, Value: {:?}", key, value);
    /// }
    /// ```

    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        self.inner.iter().map(|(k, v)| (k.as_bytes(), v.as_slice()))
    }

    /// Returns an iterator over keys as byte slices.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut data = RequestData::new();
    /// data.insert("key1", "value1");
    /// data.insert("key2", "value2");
    ///
    /// for key in data.keys() {
    ///     println!("Key: {:?}", key);
    /// }
    /// ```

    pub fn keys(&self) -> impl Iterator<Item = &[u8]> {
        self.inner.keys().map(|k| k.as_bytes())
    }

    /// Returns an iterator over values as byte slices.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut data = RequestData::new();
    /// data.insert("key1", "value1");
    /// data.insert("key2", "value2");
    ///
    /// for value in data.values() {
    ///     println!("Value: {:?}", value);
    /// }
    /// ```

    pub fn values(&self) -> impl Iterator<Item = &[u8]> {
        self.inner.values().map(|v| v.as_slice())
    }

    /// Create `RequestData` from an existing HashMap.
    ///
    /// Converts any HashMap with keys and values that can be referenced
    /// as bytes into a `RequestData` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("key1", "value1");
    /// map.insert("key2", "value2");
    ///
    /// let data = RequestData::from_map(map);
    /// assert_eq!(data.len(), 2);
    /// ```

    pub fn from_map<K, V>(map: HashMap<K, V>) -> Self
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let inner = map
            .into_iter()
            .map(|(k, v)| (ByteKey::new(k), v.as_ref().to_vec()))
            .collect();
        Self { inner }
    }

    /// Get the approximate total size in bytes of stored data.
    ///
    /// This includes the size of all keys and values, plus an estimate
    /// of the HashMap overhead. Useful for monitoring memory usage.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut data = RequestData::new();
    /// data.insert("small", "data");
    ///
    /// let size = data.byte_size();
    /// println!("Data uses approximately {} bytes", size);
    /// ```

    pub fn byte_size(&self) -> usize {
        self.inner
            .iter()
            .map(|(k, v)| k.0.len() + v.len())
            .sum::<usize>()
            + self.inner.capacity() * std::mem::size_of::<(ByteKey, Vec<u8>)>()
    }

    /// Shrink the capacity to fit the current data.
    ///
    /// This operation reduces memory usage by shrinking both the HashMap
    /// capacity and the capacity of all stored byte vectors to exactly
    /// fit their current content.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::request_data::RequestData;
    ///
    /// let mut data = RequestData::with_capacity(1000);
    /// data.insert("key", "value");
    ///
    /// // Reduce memory usage
    /// data.shrink_to_fit();
    /// ```

    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
        for value in self.inner.values_mut() {
            value.shrink_to_fit();
        }
    }
}

/// Zero-copy iteration over owned data.
///
/// When consuming a `RequestData` instance, this iterator yields
/// `(Vec<u8>, Vec<u8>)` pairs representing the owned key and value data.
impl IntoIterator for RequestData {
    type Item = (Vec<u8>, Vec<u8>);
    type IntoIter = std::iter::Map<
        std::collections::hash_map::IntoIter<ByteKey, Vec<u8>>,
        fn((ByteKey, Vec<u8>)) -> (Vec<u8>, Vec<u8>),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter().map(|(k, v)| (k.0, v))
    }
}

/// Extractor that pulls structured data from request data storage.
///
/// The `Data<T>` wrapper is used to extract strongly-typed application data
/// from a [`RequestData`] instance based on the implementation of [`FromData`] for `T`.
///
/// This enables you to easily access parsed and validated request data directly in
/// your route handlers:
///
/// # Example
///
/// ```rust,ignore
/// use ripress::req::request_data::{Data, FromData};
///
/// #[derive(FromData)]
/// struct Token {
///     token: String,
/// }
///
/// app.get("/", |data: Data<Token>, res| async move {
///     let token = &data.token;
///     // ... Use token
/// });
/// ```
///
/// `Data<T>` implements [`Deref`], so you can access fields of `T` directly by dereferencing.
///
/// [`RequestData`]: crate::req::request_data::RequestData
/// [`FromData`]: crate::req::request_data::FromData
pub struct Data<T>(T);

impl<T> Deref for Data<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: FromData> FromRequest for Data<T> {
    type Error = String;

    /// Extract structured `Data<T>` from the request.
    ///
    /// This will invoke `T::from_data` on the request's data storage.
    fn from_request(req: &super::HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self(T::from_data(&req.data)?))
    }
}

/// Trait for extracting a type from [`RequestData`] storage.
///
/// You can implement this trait manually, or automatically derive it (see `ripress_derive`).
pub trait FromData: Sized {
    /// Attempt to extract `Self` from the given [`RequestData`].
    ///
    /// Returns `Ok(Self)` if extraction is successful, or `Err(String)` if it fails.
    fn from_data(data: &RequestData) -> Result<Self, String>;
}
