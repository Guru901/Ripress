/// Represents errors that can occur when generating an HTTP response.
///
/// This enum is used to encapsulate possible error types that may arise during
/// the process of constructing or streaming an HTTP response. It is primarily
/// used for error handling in streaming responses or when IO operations fail.
///
/// # Variants
///
/// - `IoError(std::io::Error)`: Represents an IO error that occurred, such as a failure
///   to read from or write to a stream.
/// - `_Other(&'static str)`: Represents a generic or custom error with a static string message.
#[derive(Debug)]
pub enum ResponseError {
    /// An IO error occurred.
    IoError(std::io::Error),
    /// A generic or custom error with a static string message.
    _Other(&'static str),
}

impl From<std::io::Error> for ResponseError {
    fn from(err: std::io::Error) -> Self {
        ResponseError::IoError(err)
    }
}

impl std::error::Error for ResponseError {}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseError::IoError(e) => write!(f, "IO error: {}", e),
            ResponseError::_Other(e) => write!(f, "Error: {}", e),
        }
    }
}
