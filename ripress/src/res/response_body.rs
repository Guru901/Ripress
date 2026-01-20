use bytes::Bytes;
use mime_guess::MimeGuess;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ResponseBodyContent {
    TEXT(String),
    HTML(String),
    JSON(serde_json::Value),
    BINARY(Bytes),
}

impl ResponseBodyContent {
    /// Returns the content length in bytes for the current variant.
    /// Note:
    /// - TEXT/HTML: returns `String::len()` (UTF-8 byte length)
    /// - JSON: returns the length of the compact serialized form
    /// - BINARY: returns `Bytes::len()`

    #[cfg(feature = "logger")]
    pub fn len(&self) -> usize {
        match self {
            ResponseBodyContent::TEXT(text) => text.len(),
            ResponseBodyContent::HTML(html) => html.len(),
            ResponseBodyContent::JSON(json) => {
                serde_json::to_vec(json).map(|v| v.len()).unwrap_or(0)
            }
            ResponseBodyContent::BINARY(bytes) => bytes.len(),
        }
    }

    pub(crate) fn new_text<T: Into<String>>(text: T) -> Self {
        ResponseBodyContent::TEXT(text.into())
    }

    pub(crate) fn new_json<T: Serialize>(json: T) -> Self {
        Self::try_new_json(json).expect("Failed to serialize to JSON")
    }

    pub(crate) fn try_new_json<T: Serialize>(json: T) -> Result<Self, serde_json::Error> {
        serde_json::to_value(json).map(ResponseBodyContent::JSON)
    }

    pub(crate) fn new_html<T: Into<String>>(html: T) -> Self {
        ResponseBodyContent::HTML(html.into())
    }

    pub(crate) fn new_binary<T: Into<Bytes>>(bytes: T) -> Self {
        ResponseBodyContent::BINARY(bytes.into())
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub(crate) enum ResponseBodyType {
    TEXT,
    JSON,
    HTML,
    BINARY,
}

impl From<MimeGuess> for ResponseBodyType {
    fn from(guess: MimeGuess) -> Self {
        let mime = guess.first_or_octet_stream();

        match (mime.type_(), mime.subtype()) {
            (mime::TEXT, mime::HTML) => ResponseBodyType::HTML,
            (mime::TEXT, _) => ResponseBodyType::TEXT,
            (mime::APPLICATION, mime::JSON) => ResponseBodyType::JSON,
            _ => ResponseBodyType::BINARY,
        }
    }
}

impl ResponseBodyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResponseBodyType::TEXT => "text/plain",
            ResponseBodyType::JSON => "application/json",
            ResponseBodyType::HTML => "text/html",
            ResponseBodyType::BINARY => "application/octet-stream",
        }
    }
}
