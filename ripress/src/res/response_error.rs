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
pub enum HttpResponseError {
    /// An IO error occurred, typically when reading from or writing to a stream.
    IoError(std::io::Error),
    /// An expected HTTP header is missing. Contains the name of the missing header.
    MissingHeader(String),
    /// A generic or custom error with a static string message.
    _Other(&'static str),
}

impl From<std::io::Error> for HttpResponseError {
    fn from(err: std::io::Error) -> Self {
        HttpResponseError::IoError(err)
    }
}

impl std::error::Error for HttpResponseError {}

impl std::fmt::Display for HttpResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpResponseError::IoError(e) => write!(f, "IO error: {}", e),
            HttpResponseError::_Other(e) => write!(f, "Error: {}", e),
            HttpResponseError::MissingHeader(h) => write!(f, "Missing header: {}", h),
        }
    }
}
