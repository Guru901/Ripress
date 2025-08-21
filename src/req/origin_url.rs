#![warn(missing_docs)]
use std::fmt::Display;

/// A struct that represents the origin url of the request.
/// And it's methods.
#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Url {
    /// The url string
    pub(crate) url_string: String,
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url_string)
    }
}

impl Url {
    pub(crate) fn new<T: Into<String>>(url_string: T) -> Self {
        Self {
            url_string: url_string.into(),
        }
    }

    /// Returns the url string as a string slice.

    pub fn as_str(&self) -> &str {
        self.url_string.as_str()
    }

    /// Returns the value of the url string;

    pub fn value(&self) -> &String {
        &self.url_string
    }
}
