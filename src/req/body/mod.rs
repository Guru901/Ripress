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
/// - **Empty**: No body content with empty content type
///
/// # Design Philosophy
///
/// This struct enforces consistency between the content type and the actual data format,
/// preventing common mistakes like sending JSON data with a form content type. Each
/// constructor method automatically sets the appropriate content type header.
///
/// # Examples
///
/// ```rust
/// use ripress::req::body::{RequestBody, FormData, TextData};
///
/// use serde_json::json;
///
/// // JSON body
/// let json_body = RequestBody::new_json(json!({
///     "username": "alice",
///     "email": "alice@example.com"
/// }));
///
/// // Form data body
/// let mut form = FormData::new();
/// form.insert("username", "alice");
/// form.insert("password", "secret");
/// let form_body = RequestBody::new_form(form);
///
/// // Text body
/// let text_data = TextData::new(String::from("Hello, world!"));
/// let text_body = RequestBody::new_text(text_data);
///
/// // Usage in HTTP client
/// // client.post("https://api.example.com/users").body(json_body).send().await?;
/// ```
#[derive(Debug, Clone)]
pub struct RequestBody {
    /// The actual body content data
    content: RequestBodyContent,
    /// The MIME type representing the format of the content
    content_type: RequestBodyType,
}

/// Module containing form data structures and utilities.
///
/// This module provides the [`FormData`] type for handling HTML form submissions
/// and URL-encoded data, with support for parsing and generating query strings.
pub mod form_data;

/// Module containing text data structures and utilities.
///
/// This module provides the [`TextData`] type for handling plain text content
/// with validation and encoding support.
pub mod text_data;

// Re-export commonly used types for convenience
pub use form_data::FormData;
pub use text_data::{TextData, TextDataError};

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
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::{RequestBody, RequestBodyType};
    /// use ripress::req::body::text_data::TextData;
    ///
    /// let text_data = TextData::new(String::from("Hello, server!"));
    /// let body = RequestBody::new_text(text_data);
    ///
    /// assert_eq!(body.content_type, RequestBodyType::TEXT);
    /// // body.content will be RequestBodyContent::TEXT(text_data)
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Sending plain text messages
    /// - Uploading text files
    /// - Raw text data transmission
    /// - Log messages or debug information
    pub fn new_text(text: TextData) -> Self {
        RequestBody {
            content_type: RequestBodyType::TEXT,
            content: RequestBodyContent::TEXT(text),
        }
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
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::{RequestBody, RequestBodyType, FormData};
    ///
    /// let mut form = FormData::new();
    /// form.insert("username", "alice");
    /// form.insert("password", "secret123");
    /// form.insert("remember_me", "on");
    ///
    /// let body = RequestBody::new_form(form);
    /// assert_eq!(body.content_type, RequestBodyType::FORM);
    /// ```
    ///
    /// # Use Cases
    ///
    /// - HTML form submissions (login, registration, etc.)
    /// - Simple key-value data transmission
    /// - Traditional web form processing
    /// - URL parameter-style data in request body
    pub fn new_form(form_data: FormData) -> Self {
        RequestBody {
            content_type: RequestBodyType::FORM,
            content: RequestBodyContent::FORM(form_data),
        }
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
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::{RequestBody, RequestBodyType};
    /// use serde_json::json;
    /// use serde::Serialize;
    ///
    /// // Using json! macro
    /// let body1 = RequestBody::new_json(json!({
    ///     "name": "Alice",
    ///     "age": 30,
    ///     "active": true
    /// }));
    ///
    /// // Using a serializable struct
    /// #[derive(Serialize)]
    /// struct User {
    ///     id: u64,
    ///     email: String,
    /// }
    ///
    /// let user = User {
    ///     id: 123,
    ///     email: "user@example.com".to_string(),
    /// };
    /// let body2 = RequestBody::new_json(serde_json::to_value(user).unwrap());
    ///
    /// // Using primitive values
    /// let body3 = RequestBody::new_json("simple string");
    /// let body4 = RequestBody::new_json(vec![1, 2, 3, 4, 5]);
    ///
    /// assert_eq!(body1.content_type, RequestBodyType::JSON);
    /// ```
    ///
    /// # Use Cases
    ///
    /// - REST API requests
    /// - Structured data transmission
    /// - Complex nested data
    /// - Modern web application communication
    /// - Microservice communication
    pub fn new_json<T: Into<serde_json::Value>>(json: T) -> Self {
        RequestBody {
            content_type: RequestBodyType::JSON,
            content: RequestBodyContent::JSON(json.into()),
        }
    }
}

/// Enumeration of supported HTTP request body content types.
///
/// This enum represents the different MIME types that can be used for HTTP request bodies,
/// providing a type-safe way to specify content types and automatically generate the
/// appropriate HTTP headers.
///
/// # MIME Type Mapping
///
/// Each variant corresponds to a standard MIME type:
/// - `JSON` → `application/json`
/// - `TEXT` → `text/plain`
/// - `FORM` → `application/x-www-form-urlencoded`
/// - `EMPTY` → "" (empty string)
///
/// # Examples
///
/// ```rust
/// use ripress::req::body::RequestBodyType;
///
/// let json_type = RequestBodyType::JSON;
/// let form_type = RequestBodyType::FORM;
/// let text_type = RequestBodyType::TEXT;
/// let empty_type = RequestBodyType::EMPTY;
///
/// // Convert to MIME type strings
/// assert_eq!(json_type.to_string(), "application/json");
/// assert_eq!(form_type.to_string(), "application/x-www-form-urlencoded");
/// assert_eq!(text_type.to_string(), "text/plain");
/// assert_eq!(empty_type.to_string(), "");
///
/// // Types are copyable and comparable
/// let another_json = json_type; // Copy
/// assert_eq!(json_type, another_json); // PartialEq
/// ```
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum RequestBodyType {
    /// JSON content type (`application/json`).
    ///
    /// Used for structured data serialized as JSON. This is the preferred format
    /// for modern REST APIs and web services that need to transmit complex,
    /// hierarchical data structures.
    ///
    /// # Common Use Cases
    /// - REST API requests and responses
    /// - AJAX requests from web applications
    /// - Microservice communication
    /// - Configuration data transmission
    JSON,

    /// Plain text content type (`text/plain`).
    ///
    /// Used for simple text data without any specific structure or encoding
    /// beyond basic character encoding (typically UTF-8).
    ///
    /// # Common Use Cases
    /// - Log messages
    /// - Simple text file uploads
    /// - Plain text notifications
    /// - Debug or diagnostic information
    TEXT,

    /// Form data content type (`application/x-www-form-urlencoded`).
    ///
    /// Used for HTML form submissions where data is encoded as key-value pairs
    /// in the same format as URL query parameters. This is the default encoding
    /// type for HTML forms.
    ///
    /// # Common Use Cases
    /// - HTML form submissions (login, registration, contact forms)
    /// - Simple key-value data transmission
    /// - Legacy web application compatibility
    /// - OAuth token requests
    FORM,

    /// Empty content type (no body).
    ///
    /// Represents the absence of a request body, typically used for HTTP methods
    /// like GET, DELETE, or HEAD that don't usually carry body content.
    ///
    /// # Common Use Cases
    /// - GET requests
    /// - DELETE requests
    /// - HEAD requests
    /// - OPTIONS requests
    EMPTY,
}

/// Implements Copy trait for RequestBodyType.
///
/// This allows the enum to be copied rather than moved, which is efficient
/// since all variants are zero-sized and the enum is small.
impl ToString for RequestBodyType {
    /// Converts the request body type to its corresponding MIME type string.
    ///
    /// This method provides the standard MIME type string representation for each
    /// body type variant, suitable for use in HTTP Content-Type headers.
    ///
    /// # Returns
    ///
    /// The MIME type string corresponding to the enum variant:
    /// - `JSON` → `"application/json"`
    /// - `TEXT` → `"text/plain"`
    /// - `FORM` → `"application/x-www-form-urlencoded"`
    /// - `EMPTY` → `""` (empty string)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ripress::req::body::RequestBodyType;
    ///
    /// assert_eq!(RequestBodyType::JSON.to_string(), "application/json");
    /// assert_eq!(RequestBodyType::TEXT.to_string(), "text/plain");
    /// assert_eq!(RequestBodyType::FORM.to_string(), "application/x-www-form-urlencoded");
    /// assert_eq!(RequestBodyType::EMPTY.to_string(), "");
    ///
    /// // Usage in HTTP headers
    /// let content_type = RequestBodyType::JSON;
    /// let header_value = format!("Content-Type: {}", content_type.to_string());
    /// assert_eq!(header_value, "Content-Type: application/json");
    /// ```
    ///
    /// # HTTP Header Integration
    ///
    /// These strings are designed to be used directly in HTTP Content-Type headers:
    ///
    /// ```rust
    /// use ripress::req::body::RequestBodyType;
    ///
    /// fn set_content_type_header(body_type: RequestBodyType) -> String {
    ///     match body_type {
    ///         RequestBodyType::EMPTY => "".to_string(), // No Content-Type header
    ///         _ => format!("Content-Type: {}", body_type.to_string()),
    ///     }
    /// }
    /// ```
    fn to_string(&self) -> String {
        match self {
            RequestBodyType::JSON => "application/json".to_string(),
            RequestBodyType::TEXT => "text/plain".to_string(),
            RequestBodyType::FORM => "application/x-www-form-urlencoded".to_string(),
            RequestBodyType::EMPTY => "".to_string(),
        }
    }
}

/// Enumeration of the actual content data for HTTP request bodies.
///
/// This enum holds the actual data content corresponding to each supported body type,
/// providing a type-safe way to store different kinds of request body data in a
/// unified structure.
///
/// # Variant Correspondence
///
/// Each variant corresponds to a [`RequestBodyType`]:
/// - `TEXT(TextData)` ↔ `RequestBodyType::TEXT`
/// - `JSON(serde_json::Value)` ↔ `RequestBodyType::JSON`
/// - `FORM(FormData)` ↔ `RequestBodyType::FORM`
/// - `EMPTY` ↔ `RequestBodyType::EMPTY`
///
/// # Design Pattern
///
/// This enum follows the "tagged union" pattern where the tag ([`RequestBodyType`])
/// and the data ([`RequestBodyContent`]) are kept in sync by the [`RequestBody`]
/// constructor methods, ensuring type safety and preventing mismatched content
/// types and data.
///
/// # Examples
///
/// ```rust
/// use ripress::req::body::{RequestBodyContent, FormData, TextData};
/// use serde_json::json;
///
/// // The enum variants hold the actual data
/// let json_content = RequestBodyContent::JSON(json!({"key": "value"}));
///
/// let mut form = FormData::new();
/// form.insert("field", "value");
/// let form_content = RequestBodyContent::FORM(form);
///
/// let text_data = TextData::new(String::from("Hello"));
/// let text_content = RequestBodyContent::TEXT(text_data);
///
/// let empty_content = RequestBodyContent::EMPTY;
///
/// // Pattern matching for processing
/// match json_content {
///     RequestBodyContent::JSON(value) => {
///         println!("JSON data: {}", value);
///     }
///     RequestBodyContent::FORM(form) => {
///         println!("Form has {} fields", form.len());
///     }
///     RequestBodyContent::TEXT(text) => {
///         println!("Text content: {:?}", text.as_str());
///     }
///     RequestBodyContent::EMPTY => {
///         println!("No body content");
///     }
/// }
/// ```
///
/// # Memory Efficiency
///
/// The enum is designed to be memory-efficient by storing only the data that's
/// actually needed for each content type. Empty bodies consume minimal memory,
/// while complex JSON structures can use as much memory as needed.
///
/// # Serialization Considerations
///
/// While this enum is cloneable, be aware that:
/// - JSON content (via `serde_json::Value`) can represent large, deeply nested structures
/// - Form data contains owned strings for all keys and values
/// - Text data includes validation state and encoding information
/// - Cloning may be expensive for large payloads
#[derive(Debug, Clone)]
pub enum RequestBodyContent {
    /// Plain text content data.
    ///
    /// Contains a [`TextData`] instance that holds validated plain text content
    /// with encoding and validation information. Used for simple text payloads,
    /// log messages, or any unstructured text data.
    ///
    /// # Examples
    /// - Error messages
    /// - Log entries
    /// - Plain text file content
    /// - Simple string data
    TEXT(TextData),

    /// JSON content data.
    ///
    /// Contains a [`serde_json::Value`] that can represent any valid JSON structure
    /// including objects, arrays, strings, numbers, booleans, and null values.
    /// This provides maximum flexibility for structured data.
    ///
    /// # Examples
    /// - API request/response payloads
    /// - Configuration data
    /// - Complex nested data structures
    /// - User profiles, product catalogs, etc.
    JSON(serde_json::Value),

    /// Form-encoded content data.
    ///
    /// Contains a [`FormData`] instance that holds key-value pairs typically
    /// used in HTML form submissions. Data is URL-encoded when transmitted.
    ///
    /// # Examples
    /// - Login forms (username/password)
    /// - Registration forms
    /// - Contact forms
    /// - Simple key-value parameter sets
    FORM(FormData),

    /// No content (empty body).
    ///
    /// Represents the absence of body content, typically used for HTTP methods
    /// that don't carry a payload or when explicitly sending an empty body.
    ///
    /// # Examples
    /// - GET requests
    /// - DELETE requests without payload
    /// - HEAD requests
    /// - OPTIONS requests
    EMPTY,
}
