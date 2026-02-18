#![warn(missing_docs)]

/// A structured representation of HTTP request body data.
///
/// `RequestBody` provides a type-safe wrapper around different types of request body content,
/// automatically managing content types and providing convenient constructors for common
/// body formats used in HTTP requests.
///
/// # Supported Body Types
///
/// - **JSON**: Structured data serialized as JSON with `application/json` content type
/// - **Form Data**: Key-value pairs for form submissions with `application/x-www-form-urlencoded` content type
/// - **Text**: Plain text content with `text/plain` content type
/// - **Binary**: Raw binary data with `application/octet-stream` content type
/// - **Empty**: No body content with empty content type
///
/// # Design Philosophy
///
/// This struct enforces consistency between the content type and the actual data format,
/// preventing common mistakes like sending JSON data with a form content type. Each
/// constructor method automatically sets the appropriate content type header.

#[derive(Debug, Clone, PartialEq)]
pub enum RequestBody {
    TEXT(TextData),
    JSON(serde_json::Value),
    FORM(FormData),
    BINARY(Bytes),
    BinaryWithFields(Bytes, FormData),
    EMPTY,
}

#[derive(PartialEq, Debug, Clone)]
pub enum RequestBodyType {
    TEXT,
    JSON,
    FORM,
    BINARY,
    EMPTY,
    MultipartForm,
}

impl Display for RequestBodyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestBodyType::TEXT => write!(f, "text/plain"),
            RequestBodyType::JSON => write!(f, "application/json"),
            RequestBodyType::FORM => write!(f, "application/x-www-form-urlencoded"),
            RequestBodyType::BINARY => write!(f, "application/octet-stream"),
            RequestBodyType::EMPTY => write!(f, ""),
            RequestBodyType::MultipartForm => write!(f, "multipart/form-data"),
        }
    }
}

impl RequestBody {
    /// Returns the length of the data in bytes.
    ///
    /// Note:
    /// - TEXT/HTML: returns `String::len()` (UTF-8 byte length)
    /// - JSON: returns the length of the compact serialized form
    /// - BINARY: returns `Bytes::len()`
    /// - FORM: returns the length of the query string
    ///

    pub fn len(&self) -> usize {
        match self {
            RequestBody::TEXT(text) => text.len(),
            RequestBody::JSON(json) => serde_json::to_vec(json).map(|v| v.len()).unwrap_or(0),
            RequestBody::BINARY(bytes) => bytes.len(),
            RequestBody::BinaryWithFields(bytes, _form_data) => bytes.len(),
            RequestBody::EMPTY => 0,
            RequestBody::FORM(form_data) => form_data.byte_len(),
        }
    }

    pub fn body_type(&self) -> RequestBodyType {
        match self {
            RequestBody::TEXT(_) => RequestBodyType::TEXT,
            RequestBody::JSON(_) => RequestBodyType::JSON,
            RequestBody::FORM(_) => RequestBodyType::FORM,
            RequestBody::BINARY(_) => RequestBodyType::BINARY,
            RequestBody::BinaryWithFields(_, _) => RequestBodyType::BINARY,
            RequestBody::EMPTY => RequestBodyType::EMPTY,
        }
    }
}

/// Module containing form data structures and utilities.
///
/// This module provides the [`FormData`] type for handling HTML form submissions
/// and URL-encoded data, with support for parsing and generating query strings.
pub mod form_data;

pub mod json_data;
/// Module containing text data structures and utilities.
///
/// This module provides the [`TextData`] type for handling plain text content
/// with validation and encoding support.
pub mod text_data;

use std::fmt::Display;

use bytes::Bytes;
pub use form_data::FormData;
pub use text_data::TextData;

impl RequestBody {
    /// Creates a new request body with plain text content.
    ///
    /// This constructor creates a request body containing plain text data with the
    /// appropriate `text/plain` content type. The text data is validated according
    /// to the rules defined in [`TextData`].
    ///
    /// # Arguments
    ///
    /// * `text` - The text data to include in the request body
    ///
    /// # Returns
    ///
    /// A new `RequestBody` instance with `TEXT` content type
    ///
    /// # Use Cases
    ///
    /// - Sending plain text messages
    /// - Uploading text files
    /// - Raw text data transmission
    /// - Log messages or debug information
    pub(crate) fn new_text(text: TextData) -> Self {
        RequestBody::TEXT(text)
    }

    /// Creates a new request body with binary content.
    ///
    /// This constructor creates a request body containing binary data with the
    /// appropriate `application/octet-stream` content type. This is used for
    /// transmitting raw bytes without any specific structure or encoding.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The binary data to include in the request body
    ///
    /// # Returns
    ///
    /// A new `RequestBody` instance with `BINARY` content type
    ///
    /// # Use Cases
    ///
    /// - File uploads (images, documents, etc.)
    /// - Raw binary data transmission
    /// - Protocol buffer or other binary format data
    /// - Transmitting non-textual data (e.g., protocol buffers, compressed files)
    /// - Any HTTP request requiring `application/octet-stream` content type
    pub(crate) fn new_binary<T: Into<Bytes>>(bytes: T) -> Self {
        RequestBody::BINARY(bytes.into())
    }

    pub(crate) fn _new_empty() -> Self {
        Self::EMPTY
    }
    /// Creates a new request body with binary content that also contains form fields.
    ///
    /// This constructor creates a request body containing binary data (typically multipart form data)
    /// while also preserving the form fields for direct access. This is useful when processing
    /// multipart forms with files where you want both the raw bytes for middleware processing
    /// and the text fields accessible via form_data().
    ///
    /// # Arguments
    ///
    /// * `bytes` - The binary data to include in the request body
    /// * `form_data` - The form fields extracted from the multipart data
    ///
    /// # Returns
    ///
    /// A new `RequestBody` instance with `BINARY` content type but form fields accessible
    ///
    /// # Use Cases
    ///
    /// - Multipart forms with files that need middleware processing
    /// - Preserving both binary data and form fields simultaneously
    /// - Ensuring form fields are accessible even when body is binary
    pub(crate) fn new_binary_with_form_fields(bytes: Bytes, form_data: FormData) -> Self {
        RequestBody::BinaryWithFields(bytes, form_data)
    }

    /// Creates a new request body with form data content.
    ///
    /// This constructor creates a request body containing URL-encoded form data with the
    /// appropriate `application/x-www-form-urlencoded` content type. This is the standard
    /// format for HTML form submissions.
    ///
    /// # Arguments
    ///
    /// * `form_data` - The form data to include in the request body
    ///
    /// # Returns
    ///
    /// A new `RequestBody` instance with `FORM` content type
    ///
    /// # Use Cases
    ///
    /// - HTML form submissions (login, registration, etc.)
    /// - Simple key-value data transmission
    /// - Traditional web form processing
    /// - URL parameter-style data in request body
    pub(crate) fn new_form(form_data: FormData) -> Self {
        RequestBody::FORM(form_data)
    }

    /// Creates a new request body with JSON content.
    ///
    /// This constructor creates a request body containing JSON data with the
    /// appropriate `application/json` content type. The input is converted to a
    /// `serde_json::Value` which can represent any valid JSON structure.
    ///
    /// # Arguments
    ///
    /// * `json` - Any value that can be converted to `serde_json::Value`
    ///
    /// # Returns
    ///
    /// A new `RequestBody` instance with `JSON` content type
    ///
    /// # Type Flexibility
    ///
    /// This method accepts any type that implements `Into<serde_json::Value>`, including:
    /// - `serde_json::Value` directly
    /// - Values created with the `json!` macro
    /// - Any serializable struct (with `#[derive(Serialize)]`)
    /// - Primitive types (numbers, strings, booleans)
    /// - Collections (Vec, HashMap, etc.)
    ///
    /// # Use Cases
    ///
    /// - REST API requests
    /// - Structured data transmission
    /// - Complex nested data
    /// - Modern web application communication
    /// - Microservice communication
    pub(crate) fn new_json<T: Into<serde_json::Value>>(json: T) -> Self {
        RequestBody::JSON(json.into())
    }
}
