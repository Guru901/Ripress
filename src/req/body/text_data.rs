use std::fmt::{Debug, Display, Formatter};
use std::str::Utf8Error;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextDataError {
    InvalidUtf8(Utf8Error),
    TooLarge { size: usize, limit: usize },
    Empty,
}

impl Display for TextDataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TextDataError::InvalidUtf8(e) => write!(f, "Invalid UTF-8: {}", e),
            TextDataError::TooLarge { size, limit } => {
                write!(f, "Text too large: {} bytes (limit: {} bytes)", size, limit)
            }
            TextDataError::Empty => write!(f, "Text data is empty"),
        }
    }
}

impl std::error::Error for TextDataError {}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct TextData {
    inner: Vec<u8>,
    charset: Option<String>,
}

impl TryFrom<TextData> for String {
    type Error = TextDataError;
    fn try_from(value: TextData) -> Result<Self, Self::Error> {
        value.into_string()
    }
}

impl TextData {
    /// Create TextData from a String
    pub fn new(text: String) -> Self {
        Self {
            inner: text.into_bytes(),
            charset: Some("utf-8".to_string()),
        }
    }

    /// Create TextData from bytes with UTF-8 validation
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, TextDataError> {
        // Validate UTF-8
        std::str::from_utf8(&bytes).map_err(TextDataError::InvalidUtf8)?;

        Ok(Self {
            inner: bytes,
            charset: Some("utf-8".to_string()),
        })
    }

    /// Create TextData from bytes with size limit
    pub fn from_bytes_with_limit(bytes: Vec<u8>, limit: usize) -> Result<Self, TextDataError> {
        if bytes.len() > limit {
            return Err(TextDataError::TooLarge {
                size: bytes.len(),
                limit,
            });
        }
        Self::from_bytes(bytes)
    }

    /// Create TextData from raw bytes without UTF-8 validation (unsafe for display)
    pub fn from_raw_bytes(bytes: Vec<u8>, charset: Option<String>) -> Self {
        Self {
            inner: bytes,
            charset,
        }
    }

    /// Get the text as a string slice (validates UTF-8)
    pub fn as_str(&self) -> Result<&str, TextDataError> {
        std::str::from_utf8(&self.inner).map_err(TextDataError::InvalidUtf8)
    }

    /// Get the text as a string slice, replacing invalid UTF-8 with replacement characters
    pub fn as_str_lossy(&self) -> std::borrow::Cow<str> {
        String::from_utf8_lossy(&self.inner)
    }

    /// Convert to String (validates UTF-8)
    pub fn into_string(self) -> Result<String, TextDataError> {
        String::from_utf8(self.inner).map_err(|e| TextDataError::InvalidUtf8(e.utf8_error()))
    }

    /// Convert to String, replacing invalid UTF-8 with replacement characters
    pub fn into_string_lossy(self) -> String {
        String::from_utf8_lossy(&self.inner).into_owned()
    }

    /// Get raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Get raw bytes mutably (internal only; may violate UTF-8 invariants)
    pub(crate) fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }

    /// Convert to raw bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.inner
    }

    /// Get the length in bytes
    pub fn len_bytes(&self) -> usize {
        self.inner.len()
    }

    /// Get the length in characters (if valid UTF-8)
    pub fn len_chars(&self) -> Result<usize, TextDataError> {
        Ok(self.as_str()?.chars().count())
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the charset
    pub fn charset(&self) -> Option<&str> {
        self.charset.as_deref()
    }

    /// Set the charset
    pub fn set_charset(&mut self, charset: String) {
        self.charset = Some(charset);
    }

    /// Check if the text is valid UTF-8
    pub fn is_valid_utf8(&self) -> bool {
        std::str::from_utf8(&self.inner).is_ok()
    }

    /// Get lines iterator (if valid UTF-8)
    pub fn lines(&self) -> Result<std::str::Lines, TextDataError> {
        Ok(self.as_str()?.lines())
    }

    /// Trim whitespace (if valid UTF-8)
    pub fn trim(&self) -> Result<&str, TextDataError> {
        Ok(self.as_str()?.trim())
    }

    /// Check if text contains a substring (if valid UTF-8)
    pub fn contains(&self, needle: &str) -> Result<bool, TextDataError> {
        Ok(self.as_str()?.contains(needle))
    }

    /// Split text by delimiter (if valid UTF-8)
    pub fn split<'a>(
        &'a self,
        delimiter: &'a str,
    ) -> Result<std::str::Split<'a, &'a str>, TextDataError> {
        Ok(self.as_str()?.split(delimiter))
    }

    /// Truncate to maximum byte length
    pub fn truncate_bytes(&mut self, max_len: usize) {
        if self.inner.len() > max_len {
            self.inner.truncate(max_len);
            // Ensure we don't cut in the middle of a UTF-8 character
            while !self.inner.is_empty() && !std::str::from_utf8(&self.inner).is_ok() {
                self.inner.pop();
            }
        }
    }

    /// Create a truncated copy with maximum byte length
    pub fn truncated_bytes(&self, max_len: usize) -> Self {
        let mut copy = self.clone();
        copy.truncate_bytes(max_len);
        copy
    }
}

impl Display for TextData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{}", s),
            Err(_) => {
                // Fallback to lossy conversion for display
                write!(f, "{}", self.as_str_lossy())
            }
        }
    }
}

impl Debug for TextData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextData")
            .field("len_bytes", &self.len_bytes())
            .field("charset", &self.charset)
            .field("is_valid_utf8", &self.is_valid_utf8())
            .field("preview", &format!("{:.50}...", self.as_str_lossy()))
            .finish()
    }
}

impl From<String> for TextData {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for TextData {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl TryFrom<Vec<u8>> for TextData {
    type Error = TextDataError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_bytes(bytes)
    }
}

// Deref to bytes for low-level operations
impl std::ops::Deref for TextData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
