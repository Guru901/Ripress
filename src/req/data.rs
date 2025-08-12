use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
};

/// A high-performance wrapper around bytes that implements Hash and Eq
/// by comparing the underlying byte content
#[derive(Clone, Debug)]
pub struct ByteKey(Vec<u8>);

impl ByteKey {
    pub fn new(data: impl AsRef<[u8]>) -> Self {
        Self(data.as_ref().to_vec())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

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

/// High-performance request data structure optimized for byte storage
#[derive(Clone, Debug, Default)]
pub struct RequestData {
    inner: HashMap<ByteKey, Vec<u8>>,
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
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    /// Insert data - takes anything that can be converted to bytes
    pub fn insert(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        let key = ByteKey::new(key);
        let value = value.as_ref().to_vec();
        self.inner.insert(key, value);
    }

    /// Insert without copying if you already have owned data
    pub fn insert_owned(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.inner.insert(ByteKey(key), value);
    }

    /// Get value as bytes
    pub fn get(&self, key: impl AsRef<[u8]>) -> Option<String> {
        let key = ByteKey::new(key);
        let data = self.inner.get(&key);

        match data {
            Some(data) => match String::from_utf8(data.clone()) {
                Ok(s) => Some(s),
                Err(_) => None,
            },
            None => None,
        }
    }

    /// Remove and return the value
    pub fn remove(&mut self, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
        let key = ByteKey::new(key);
        self.inner.remove(&key)
    }

    pub fn contains_key(&self, key: impl AsRef<[u8]>) -> bool {
        let key = ByteKey::new(key);
        self.inner.contains_key(&key)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Iterate over key-value pairs as byte slices
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        self.inner.iter().map(|(k, v)| (k.as_bytes(), v.as_slice()))
    }

    /// Iterate over keys as byte slices
    pub fn keys(&self) -> impl Iterator<Item = &[u8]> {
        self.inner.keys().map(|k| k.as_bytes())
    }

    /// Iterate over values as byte slices
    pub fn values(&self) -> impl Iterator<Item = &[u8]> {
        self.inner.values().map(|v| v.as_slice())
    }

    /// Create from existing HashMap
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

    /// Get the total size in bytes (approximate memory usage)
    pub fn byte_size(&self) -> usize {
        self.inner
            .iter()
            .map(|(k, v)| k.0.len() + v.len())
            .sum::<usize>()
            + self.inner.capacity() * std::mem::size_of::<(ByteKey, Vec<u8>)>()
    }

    /// Shrink the capacity to fit the current data
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
        for value in self.inner.values_mut() {
            value.shrink_to_fit();
        }
    }
}

// Zero-copy iteration over owned data
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
