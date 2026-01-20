/// Represents possible errors that can occur when handling an HTTP request.
#[derive(Debug, PartialEq)]
pub enum HttpRequestError {
    /// Indicates that a required cookie is missing.
    /// The associated `String` is the name of the missing cookie.
    MissingCookie(String),
    /// Indicates that a required URL parameter is missing.
    /// The associated `String` is the name of the missing parameter.
    MissingParam(String),
    /// Indicates that a required HTTP header is missing.
    /// The associated `String` is the name of the missing header.
    MissingHeader(String),
    /// Indicates that a required query parameter is missing.
    /// The associated `String` is the name of the missing query parameter.
    MissingQuery(String),
    /// Indicates that the request body contains invalid JSON.
    /// The associated `String` provides details about the JSON error.
    InvalidJson(String),
}

impl std::fmt::Display for HttpRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpRequestError::MissingCookie(cookie) => write!(f, "Cookie {} doesn't exist", cookie),
            HttpRequestError::MissingParam(param) => write!(f, "Param {} doesn't exist", param),
            HttpRequestError::MissingHeader(header) => write!(f, "Header {} doesn't exist", header),
            HttpRequestError::MissingQuery(query) => write!(f, "Query {} doesn't exist", query),
            HttpRequestError::InvalidJson(json) => write!(f, "JSON is invalid: {}", json),
        }
    }
}
