#![warn(missing_docs)]
use std::fmt::{Debug, Display, Formatter};
use std::str::Utf8Error;

use serde::Serialize;

use crate::error::RipressError;

/// Represents various errors that can occur when working with text data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TextDataError {
    /// The byte sequence is not valid UTF-8.
    ///
    /// This error occurs when attempting to interpret bytes as UTF-8 text
    /// but the bytes don't form a valid UTF-8 sequence.
    InvalidUtf8(Utf8Error),

    /// The text data exceeds the specified size limit.
    ///
    /// Contains the actual size and the enforced limit in bytes.
    TooLarge {
        /// How much big the data was
        size: usize,

        /// How much was the limit set to
        limit: usize,
    },
}

impl Display for TextDataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TextDataError::InvalidUtf8(e) => write!(f, "Invalid UTF-8: {}", e),
            TextDataError::TooLarge { size, limit } => {
                write!(f, "Text too large: {} bytes (limit: {} bytes)", size, limit)
            }
        }
    }
}

impl std::error::Error for TextDataError {}

/// A flexible container for text data that can handle both valid UTF-8 and raw bytes.
///
/// `TextData` provides a safe wrapper around byte data that may or may not be valid UTF-8.
/// It tracks the charset information and provides both strict UTF-8 methods (that return errors
/// for invalid sequences) and lossy methods (that replace invalid sequences with replacement characters).
///
/// # Examples
///
/// ```rust
/// // Creating from a string (always valid UTF-8)
/// use ripress::req::body::text_data::TextData;
///
/// let text = TextData::new("Hello, world!".to_string());
/// assert_eq!(text.as_str().unwrap(), "Hello, world!");
///
/// // Creating from bytes with validation
/// let bytes = "Hello, 世界!".as_bytes().to_vec();
/// let text = TextData::from_bytes(bytes).unwrap();
/// assert_eq!(text.len_chars().unwrap(), 10);
///
/// // Working with potentially invalid UTF-8
/// let mixed_bytes = vec![72, 101, 108, 108, 111, 0xFF, 0xFE]; // "Hello" + invalid bytes
/// let text = TextData::from_raw_bytes(mixed_bytes, Some("mixed".to_string()));
/// assert!(!text.is_valid_utf8());
/// assert!(text.as_str_lossy().contains("Hello")); // Lossy conversion works
/// ```
#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct TextData {
    inner: Vec<u8>,
    charset: Option<String>,
}

impl TryFrom<TextData> for String {
    type Error = RipressError;

    /// Attempts to convert `TextData` to a `String`.
    ///
    /// # Errors
    ///
    /// Returns `TextDataError::InvalidUtf8` if the underlying bytes are not valid UTF-8.
    fn try_from(value: TextData) -> Result<Self, Self::Error> {
        value.into_string()
    }
}

impl TextData {
    /// Creates new `TextData` from a `String`.
    ///
    /// The resulting `TextData` is guaranteed to contain valid UTF-8 and has its charset
    /// set to "utf-8".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let text = TextData::new("Hello, world!".to_string());
    /// assert_eq!(text.charset(), Some("utf-8"));
    /// assert!(text.is_valid_utf8());
    /// ```
    pub fn new<T: Into<String>>(text: T) -> Self {
        Self {
            inner: text.into().into_bytes(),
            charset: Some("utf-8".to_string()),
        }
    }

    /// Creates `TextData` from a byte vector with UTF-8 validation.
    ///
    /// This method validates that the provided bytes form a valid UTF-8 sequence.
    /// If validation succeeds, the charset is automatically set to "utf-8".
    ///
    /// # Arguments
    ///
    /// * `bytes` - The byte vector to convert to text data
    ///
    /// # Returns
    ///
    /// * `Ok(TextData)` if the bytes are valid UTF-8
    /// * `Err(TextDataError::InvalidUtf8)` if the bytes are not valid UTF-8
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// // Valid UTF-8
    ///
    /// let bytes = "Hello, 世界!".as_bytes().to_vec();
    /// let text = TextData::from_bytes(bytes).unwrap();
    /// assert_eq!(text.as_str().unwrap(), "Hello, 世界!");
    ///
    /// // Invalid UTF-8
    /// let invalid_bytes = vec![0xFF, 0xFE];
    /// assert!(TextData::from_bytes(invalid_bytes).is_err());
    /// ```
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, RipressError> {
        std::str::from_utf8(&bytes).map_err(TextDataError::InvalidUtf8)?;

        Ok(Self {
            inner: bytes,
            charset: Some("utf-8".to_string()),
        })
    }

    /// Creates `TextData` from bytes with both UTF-8 validation and size limit enforcement.
    ///
    /// This method first checks if the byte vector exceeds the specified limit,
    /// then validates UTF-8 encoding if the size check passes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The byte vector to convert to text data
    /// * `limit` - Maximum allowed size in bytes
    ///
    /// # Returns
    ///
    /// * `Ok(TextData)` if the bytes are within the limit and valid UTF-8
    /// * `Err(TextDataError::TooLarge)` if the bytes exceed the size limit
    /// * `Err(TextDataError::InvalidUtf8)` if the bytes are not valid UTF-8
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let small_text = "Hi!".as_bytes().to_vec();
    /// let text = TextData::from_bytes_with_limit(small_text, 10).unwrap();
    /// assert_eq!(text.len_bytes(), 3);
    ///
    /// let large_text = "This is a very long string".as_bytes().to_vec();
    /// assert!(TextData::from_bytes_with_limit(large_text, 5).is_err());
    /// ```
    pub fn from_bytes_with_limit(bytes: Vec<u8>, limit: usize) -> Result<Self, RipressError> {
        if bytes.len() > limit {
            return Err(RipressError::from(TextDataError::TooLarge {
                size: bytes.len(),
                limit,
            }));
        }
        Self::from_bytes(bytes)
    }

    /// Creates `TextData` from raw bytes without UTF-8 validation.
    ///
    /// This method is useful when you need to store potentially invalid UTF-8 data
    /// or when working with text in non-UTF-8 encodings. The charset parameter
    /// helps track what encoding the data might be in.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The raw byte vector
    /// * `charset` - Optional charset information (e.g., "iso-8859-1", "cp1252")
    ///
    /// # Safety
    ///
    /// This method doesn't validate UTF-8, so calling UTF-8-specific methods like
    /// `as_str()` may fail. Use `is_valid_utf8()` to check or `as_str_lossy()`
    /// for safe string conversion.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// // Creating with unknown encoding
    /// let raw_bytes = vec![0xC4, 0xE9, 0xF1, 0xF2]; // Some encoding
    /// let text = TextData::from_raw_bytes(raw_bytes, Some("cp1252".to_string()));
    /// assert_eq!(text.charset(), Some("cp1252"));
    /// assert!(!text.is_valid_utf8());
    /// ```
    pub fn from_raw_bytes(bytes: Vec<u8>, charset: Option<String>) -> Self {
        Self {
            inner: bytes,
            charset,
        }
    }

    /// Returns the text as a string slice, validating UTF-8.
    ///
    /// This method performs UTF-8 validation each time it's called. For better
    /// performance with known-valid UTF-8 data, consider using methods that
    /// don't require validation.
    ///
    /// # Returns
    ///
    /// * `Ok(&str)` if the data is valid UTF-8
    /// * `Err(TextDataError::InvalidUtf8)` if the data contains invalid UTF-8 sequences
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let text = TextData::new("Hello!".to_string());
    /// assert_eq!(text.as_str().unwrap(), "Hello!");
    ///
    /// let invalid = TextData::from_raw_bytes(vec![0xFF], None);
    /// assert!(invalid.as_str().is_err());
    /// ```
    pub fn as_str(&self) -> Result<&str, RipressError> {
        std::str::from_utf8(&self.inner)
            .map_err(|e| RipressError::from(TextDataError::InvalidUtf8(e)))
    }

    /// Returns the text as a string slice, replacing invalid UTF-8 with replacement characters.
    ///
    /// This method never fails and always returns a valid string representation.
    /// Invalid UTF-8 sequences are replaced with the Unicode replacement character (�).
    ///
    /// # Returns
    ///
    /// A `Cow<str>` that borrows from the original data when possible, or owns
    /// a new string when replacement characters were needed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let valid = TextData::new("Hello!".to_string());
    /// assert_eq!(valid.as_str_lossy(), "Hello!");
    ///
    /// let invalid = TextData::from_raw_bytes(vec![b'H', b'i', 0xFF], None);
    /// assert_eq!(invalid.as_str_lossy(), "Hi�");
    /// ```
    pub fn as_str_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.inner)
    }

    /// Converts to a `String`, validating UTF-8.
    ///
    /// This method consumes the `TextData` and attempts to convert it to a `String`.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` if the data is valid UTF-8
    /// * `Err(TextDataError::InvalidUtf8)` if the data contains invalid UTF-8 sequences
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let text = TextData::new("Hello!".to_string());
    /// let string = text.into_string().unwrap();
    /// assert_eq!(string, "Hello!");
    /// ```
    pub fn into_string(self) -> Result<String, RipressError> {
        String::from_utf8(self.inner)
            .map_err(|e| RipressError::from(TextDataError::InvalidUtf8(e.utf8_error())))
    }

    /// Converts to a `String`, replacing invalid UTF-8 with replacement characters.
    ///
    /// This method never fails and always returns a valid string. It consumes the
    /// `TextData` and produces an owned `String`.
    ///
    /// # Returns
    ///
    /// A `String` with any invalid UTF-8 sequences replaced by the Unicode
    /// replacement character (�).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let invalid = TextData::from_raw_bytes(vec![b'H', b'i', 0xFF], None);
    /// assert_eq!(invalid.into_string_lossy(), "Hi�");
    /// ```
    pub fn into_string_lossy(self) -> String {
        String::from_utf8_lossy(&self.inner).into_owned()
    }

    /// Returns a reference to the underlying byte array.
    ///
    /// This provides direct access to the raw bytes without any encoding validation.
    /// Useful for binary operations or when you need to work with the data at the byte level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let text = TextData::new("Hello!".to_string());
    /// assert_eq!(text.as_bytes(), b"Hello!");
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Returns a mutable reference to the underlying byte array.
    ///
    /// # Safety
    ///
    /// This method is marked as `pub(crate)` because directly modifying the bytes
    /// can violate UTF-8 invariants. Only internal code should use this method,
    /// and it should ensure that any modifications maintain data integrity.
    ///
    /// After modifying bytes through this method, UTF-8 validation methods may
    /// start failing even if they previously succeeded.
    pub(crate) fn _as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }

    /// Converts to the underlying byte vector, consuming the `TextData`.
    ///
    /// This method transfers ownership of the internal byte vector to the caller.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let text = TextData::new("Hello!".to_string());
    /// let bytes = text.into_bytes();
    /// assert_eq!(bytes, b"Hello!");
    /// ```
    pub fn into_bytes(self) -> Vec<u8> {
        self.inner
    }

    /// Returns the length of the data in bytes.
    ///
    /// Note that for UTF-8 text, this may be different from the number of characters,
    /// as UTF-8 characters can be 1-4 bytes long.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let ascii = TextData::new("Hello".to_string());
    /// assert_eq!(ascii.len_bytes(), 5);
    ///
    /// let unicode = TextData::new("世界".to_string()); // Two Chinese characters
    /// assert_eq!(unicode.len_bytes(), 6); // 3 bytes each in UTF-8
    /// ```
    pub fn len_bytes(&self) -> usize {
        self.inner.len()
    }

    /// Returns the length of the text in Unicode scalar values (characters).
    ///
    /// This method validates UTF-8 and counts Unicode scalar values, not bytes.
    /// Each emoji, accented character, or multi-byte Unicode character counts as one.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` with the character count if the data is valid UTF-8
    /// * `Err(TextDataError::InvalidUtf8)` if the data contains invalid UTF-8 sequences
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let ascii = TextData::new("Hello".to_string());
    /// assert_eq!(ascii.len_chars().unwrap(), 5);
    ///
    /// let unicode = TextData::new("🦀Rust".to_string());
    /// assert_eq!(unicode.len_chars().unwrap(), 5); // 1 emoji + 4 ASCII chars
    /// assert_eq!(unicode.len_bytes(), 8); // But 8 bytes total
    /// ```
    pub fn len_chars(&self) -> Result<usize, RipressError> {
        Ok(self.as_str()?.chars().count())
    }

    /// Returns `true` if the text data contains no bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let empty = TextData::new(String::new());
    /// assert!(empty.is_empty());
    ///
    /// let not_empty = TextData::new("Hello".to_string());
    /// assert!(!not_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the charset information if available.
    ///
    /// The charset indicates the character encoding of the data. It's automatically
    /// set to "utf-8" for validated UTF-8 data, or can be manually specified for
    /// other encodings.
    ///
    /// # Returns
    ///
    /// * `Some(&str)` with the charset name
    /// * `None` if no charset information is available
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let utf8_text = TextData::new("Hello".to_string());
    /// assert_eq!(utf8_text.charset(), Some("utf-8"));
    ///
    /// let raw_text = TextData::from_raw_bytes(vec![0x48, 0x69], None);
    /// assert_eq!(raw_text.charset(), None);
    /// ```
    pub fn charset(&self) -> Option<&str> {
        self.charset.as_deref()
    }

    /// Sets the charset information.
    ///
    /// This method updates the charset metadata but doesn't validate that the data
    /// actually conforms to the specified encoding. It's primarily for bookkeeping.
    ///
    /// # Arguments
    ///
    /// * `charset` - The charset name (e.g., "utf-8", "iso-8859-1", "cp1252")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let mut text = TextData::from_raw_bytes(vec![0xE9], None);
    /// text.set_charset("iso-8859-1".to_string());
    /// assert_eq!(text.charset(), Some("iso-8859-1"));
    /// ```
    pub fn set_charset(&mut self, charset: String) {
        self.charset = Some(charset);
    }

    /// Returns `true` if the underlying bytes form a valid UTF-8 sequence.
    ///
    /// This method performs UTF-8 validation without consuming the data or
    /// creating string slices. It's useful for checking validity before
    /// attempting operations that require valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let valid = TextData::new("Hello, 世界!".to_string());
    /// assert!(valid.is_valid_utf8());
    ///
    /// let invalid = TextData::from_raw_bytes(vec![0xFF, 0xFE], None);
    /// assert!(!invalid.is_valid_utf8());
    /// ```
    pub fn is_valid_utf8(&self) -> bool {
        std::str::from_utf8(&self.inner).is_ok()
    }

    /// Returns an iterator over the lines in the text.
    ///
    /// Lines are split on `\n` characters. The iterator yields string slices that
    /// do not include the line terminator.
    ///
    /// # Returns
    ///
    /// * `Ok(Lines)` iterator if the data is valid UTF-8
    /// * `Err(TextDataError::InvalidUtf8)` if the data contains invalid UTF-8 sequences
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let text = TextData::new("Line 1\nLine 2\nLine 3".to_string());
    /// let lines: Vec<&str> = text.lines().unwrap().collect();
    /// assert_eq!(lines, vec!["Line 1", "Line 2", "Line 3"]);
    /// ```
    pub fn lines(&self) -> Result<std::str::Lines<'_>, RipressError> {
        Ok(self.as_str()?.lines())
    }

    /// Returns the text with leading and trailing whitespace removed.
    ///
    /// This method validates UTF-8 and returns a string slice with whitespace trimmed.
    ///
    /// # Returns
    ///
    /// * `Ok(&str)` with trimmed text if the data is valid UTF-8
    /// * `Err(TextDataError::InvalidUtf8)` if the data contains invalid UTF-8 sequences
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let text = TextData::new("  Hello, world!  ".to_string());
    /// assert_eq!(text.trim().unwrap(), "Hello, world!");
    /// ```
    pub fn trim(&self) -> Result<&str, RipressError> {
        Ok(self.as_str()?.trim())
    }

    /// Returns `true` if the text contains the specified substring.
    ///
    /// This method validates UTF-8 and performs a substring search.
    ///
    /// # Arguments
    ///
    /// * `needle` - The substring to search for
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` indicating whether the substring was found
    /// * `Err(TextDataError::InvalidUtf8)` if the data contains invalid UTF-8 sequences
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let text = TextData::new("Hello, world!".to_string());
    /// assert!(text.contains("world").unwrap());
    /// assert!(!text.contains("Rust").unwrap());
    /// ```
    pub fn contains(&self, needle: &str) -> Result<bool, RipressError> {
        Ok(self.as_str()?.contains(needle))
    }

    /// Returns an iterator over substrings split by the specified delimiter.
    ///
    /// This method validates UTF-8 and splits the text on occurrences of the delimiter.
    /// The delimiter itself is not included in the yielded substrings.
    ///
    /// # Arguments
    ///
    /// * `delimiter` - The string to split on
    ///
    /// # Returns
    ///
    /// * `Ok(Split)` iterator if the data is valid UTF-8
    /// * `Err(TextDataError::InvalidUtf8)` if the data contains invalid UTF-8 sequences
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let text = TextData::new("apple,banana,cherry".to_string());
    /// let parts: Vec<&str> = text.split(",").unwrap().collect();
    /// assert_eq!(parts, vec!["apple", "banana", "cherry"]);
    /// ```
    pub fn split<'a>(
        &'a self,
        delimiter: &'a str,
    ) -> Result<std::str::Split<'a, &'a str>, RipressError> {
        Ok(self.as_str()?.split(delimiter))
    }

    /// Truncates the data to the specified maximum byte length.
    ///
    /// If the current data is longer than `max_len` bytes, it will be truncated.
    /// This method ensures that truncation doesn't happen in the middle of a UTF-8
    /// character sequence by removing additional bytes until a valid UTF-8 boundary is found.
    ///
    /// # Arguments
    ///
    /// * `max_len` - Maximum allowed length in bytes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let mut text = TextData::new("Hello, world!".to_string());
    /// text.truncate_bytes(5);
    /// assert_eq!(text.as_str().unwrap(), "Hello");
    ///
    /// // Works safely with multi-byte UTF-8 characters
    /// let mut unicode_text = TextData::new("世界Hello".to_string());
    /// unicode_text.truncate_bytes(7); // Might cut in middle of character
    /// assert!(unicode_text.is_valid_utf8()); // But result is still valid
    /// ```
    pub fn truncate_bytes(&mut self, max_len: usize) {
        if self.inner.len() > max_len {
            self.inner.truncate(max_len);
            while !self.inner.is_empty() && !std::str::from_utf8(&self.inner).is_ok() {
                self.inner.pop();
            }
        }
    }

    /// Creates a truncated copy with the specified maximum byte length.
    ///
    /// This method returns a new `TextData` instance that contains at most `max_len` bytes.
    /// Like `truncate_bytes()`, it ensures UTF-8 character boundaries are respected.
    ///
    /// # Arguments
    ///
    /// * `max_len` - Maximum allowed length in bytes
    ///
    /// # Returns
    ///
    /// A new `TextData` instance with the truncated content.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let text = TextData::new("Hello, world!".to_string());
    /// let short = text.truncated_bytes(5);
    /// assert_eq!(short.as_str().unwrap(), "Hello");
    /// assert_eq!(text.as_str().unwrap(), "Hello, world!"); // Original unchanged
    /// ```
    pub fn truncated_bytes(&self, max_len: usize) -> Self {
        let mut copy = self.clone();
        copy.truncate_bytes(max_len);
        copy
    }
}

impl Display for TextData {
    /// Formats the text data for display.
    ///
    /// This implementation first attempts to display the data as valid UTF-8.
    /// If that fails, it falls back to lossy conversion, replacing invalid
    /// sequences with replacement characters.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{}", s),
            Err(_) => {
                write!(f, "{}", self.as_str_lossy())
            }
        }
    }
}

impl Debug for TextData {
    /// Formats the text data for debugging.
    ///
    /// The debug representation shows internal structure including:
    /// - Length in bytes
    /// - Charset information
    /// - UTF-8 validity status
    /// - A preview of the content (first 50 characters, lossy)
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
    /// Creates `TextData` from a `String`.
    ///
    /// This is equivalent to calling `TextData::new()` and guarantees valid UTF-8 content.
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for TextData {
    /// Creates `TextData` from a string slice.
    ///
    /// The string slice is converted to an owned `String` and then to `TextData`,
    /// guaranteeing valid UTF-8 content.
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl TryFrom<Vec<u8>> for TextData {
    type Error = RipressError;

    /// Attempts to create `TextData` from a byte vector with UTF-8 validation.
    ///
    /// This is equivalent to calling `TextData::from_bytes()`.
    ///
    /// # Errors
    ///
    /// Returns `TextDataError::InvalidUtf8` if the bytes are not valid UTF-8.
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_bytes(bytes)
    }
}

/// Provides direct access to the underlying bytes.
///
/// This implementation allows `TextData` to be used anywhere a `&[u8]` is expected,
/// enabling low-level byte operations without explicit conversion.
///
/// # Examples
///
/// ```rust
/// use ripress::req::body::text_data::TextData;
///
/// let text = TextData::new("Hello".to_string());
/// let bytes: &[u8] = &*text; // Deref coercion
/// assert_eq!(bytes, b"Hello");
/// ```
impl std::ops::Deref for TextData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
