use std::fmt::Display;

/// A struct that represents the origin url of the request.
/// And it's methods.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Url {
    /// The url string
    pub url_string: &'static str,
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url_string)
    }
}

impl Url {
    pub(crate) fn new(url_string: &'static str) -> Self {
        Self { url_string }
    }

    pub(crate) fn from<T: Into<String>>(url_string: T) -> Self {
        let static_str: &'static str = Box::leak(url_string.into().into_boxed_str());
        Self::new(static_str)
    }
}
