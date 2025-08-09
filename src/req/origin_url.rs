use std::fmt::Display;

/// A struct that represents the origin url of the request.
/// And it's methods.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Url {
    /// The url string
    pub url_string: String,
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
}
