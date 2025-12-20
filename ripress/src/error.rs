use std::fmt::Display;

use crate::req::{
    body::text_data::TextDataError, query_params::QueryParamError, route_params::ParamError,
};

/// Represents the category or type of error that can occurr in the Ripress.
///
/// This enum is used to classify errors for easier handling and reporting.
/// Each variant corresponds to a broad class of errors that may arise during
/// request processing, parsing, or application logic.
///
/// # Variants
///
/// - `IO`: An input/output error, typically from file or network operations.
/// - `ParseError`: An error occurred while parsing data (e.g., query params, body).
/// - `InvalidInput`: The input provided was invalid or malformed.
/// - `NotFound`: The requested resource or parameter was not found.
#[derive(Debug, PartialEq, Eq)]
pub enum RipressErrorKind {
    /// An input/output error, such as file or network failure.
    IO,
    /// An error occurred while parsing data.
    ParseError,
    /// The input provided was invalid or malformed.
    InvalidInput,
    /// The requested resource or parameter was not found.
    NotFound,
}

impl Display for RipressErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RipressErrorKind::IO => write!(f, "IO error"),
            RipressErrorKind::ParseError => write!(f, "Parse error"),
            RipressErrorKind::InvalidInput => write!(f, "Invalid input"),
            RipressErrorKind::NotFound => write!(f, "Not found"),
        }
    }
}

/// A structured error type for representing errors in the Ripress framework.
///
/// `RipressError` encapsulates both the kind of error (using [`RipressErrorKind`])
/// and a human-readable message describing the error. This allows for consistent
/// error handling and reporting throughout the framework.
///
/// # Fields
///
/// - `kind`: The broad category of the error (see [`RipressErrorKind`]).
/// - `message`: A detailed, human-readable description of the error.
///
/// # Examples
///
/// ```rust
/// use ripress::error::{RipressError, RipressErrorKind};
///
/// let err = RipressError {
///     kind: RipressErrorKind::ParseError,
///     message: "Failed to parse integer".to_string(),
/// };
/// assert_eq!(err.kind, RipressErrorKind::ParseError);
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct RipressError {
    /// The category or type of error.
    pub kind: RipressErrorKind,
    /// A human-readable error message.
    pub message: String,
}

impl RipressError {
    /// Creates a new `RipressError` with the specified kind and message.
    ///
    /// # Arguments
    ///
    /// * `kind` - The category or type of error (see [`RipressErrorKind`]).
    /// * `message` - A human-readable error message describing the error.
    ///
    /// # Example
    ///
    /// ```
    /// use ripress::error::{RipressError, RipressErrorKind};
    ///
    /// let err = RipressError::new(RipressErrorKind::InvalidInput, "Invalid user ID".to_string());
    /// assert_eq!(err.kind, RipressErrorKind::InvalidInput);
    /// assert_eq!(err.message, "Invalid user ID");
    /// ```
    pub fn new(kind: RipressErrorKind, message: String) -> Self {
        Self { kind, message }
    }

    /// Returns a reference to the error message.
    ///
    /// # Returns
    ///
    /// A string slice containing the human-readable error message associated with this error.
    ///
    /// # Example
    ///
    /// ```
    /// use ripress::error::{RipressError, RipressErrorKind};
    /// let err = RipressError::new(RipressErrorKind::InvalidInput, "Invalid user ID".to_string());
    /// assert_eq!(err.message(), "Invalid user ID");
    /// ```
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns a reference to the error kind.
    ///
    /// # Returns
    ///
    /// A reference to the [`RipressErrorKind`] indicating the category or type of this error.
    ///
    /// # Example
    ///
    /// ```
    /// use ripress::error::{RipressError, RipressErrorKind};
    /// let err = RipressError::new(RipressErrorKind::ParseError, "Failed to parse".to_string());
    /// assert_eq!(*err.kind(), RipressErrorKind::ParseError);
    /// ```
    pub fn kind(&self) -> &RipressErrorKind {
        &self.kind
    }
}

impl Display for RipressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RipressError: {{ message: {}, kind: {} }}",
            self.message, self.kind
        )
    }
}

impl From<std::io::Error> for RipressError {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: RipressErrorKind::IO,
            message: err.to_string(),
        }
    }
}

impl From<std::string::FromUtf8Error> for RipressError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self {
            kind: RipressErrorKind::ParseError,
            message: err.to_string(),
        }
    }
}

impl From<QueryParamError> for RipressError {
    fn from(value: QueryParamError) -> Self {
        match value {
            QueryParamError::NotFound(param) => Self {
                kind: RipressErrorKind::NotFound,
                message: format!("Query Param '{}' not found", param),
            },
            QueryParamError::ParseError {
                param,
                value,
                target_type,
            } => Self {
                kind: RipressErrorKind::ParseError,
                message: format!(
                    "Failed to parse '{}' from: {} to:'{}'",
                    param, target_type, value
                ),
            },
        }
    }
}

impl From<ParamError> for RipressError {
    fn from(value: ParamError) -> Self {
        match value {
            ParamError::NotFound(param) => Self {
                kind: RipressErrorKind::NotFound,
                message: format!("Route Param '{}' not found", param),
            },
            ParamError::ParseError {
                param,
                value,
                target_type,
            } => Self {
                kind: RipressErrorKind::ParseError,
                message: format!(
                    "Failed to parse route param '{}' from: {} to: '{}'",
                    param, target_type, value
                ),
            },
        }
    }
}

impl From<TextDataError> for RipressError {
    fn from(value: TextDataError) -> Self {
        match value {
            TextDataError::InvalidUtf8(utf8_error) => Self {
                kind: RipressErrorKind::ParseError,
                message: utf8_error.to_string(),
            },
            TextDataError::TooLarge { size, limit } => Self {
                kind: RipressErrorKind::InvalidInput,
                message: format!("Text too large: {} bytes (limit: {} bytes)", size, limit),
            },
        }
    }
}
