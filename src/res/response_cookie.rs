/// Options for the SameSite attribute of cookies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookieSameSiteOptions {
    /// Sets the SameSite attribute to Strict
    Strict,

    /// Sets the SameSite attribute to Lax
    Lax,

    /// Sets the SameSite attribute to None
    None,
}

/// Options for setting cookies
#[derive(Debug, Clone, PartialEq)]
pub struct CookieOptions {
    /// Sets the HttpOnly attribute
    pub http_only: bool,

    /// Sets the Secure attribute
    pub secure: bool,

    /// Sets the SameSite attribute
    pub same_site: CookieSameSiteOptions,

    /// Sets the Path attribute
    pub path: Option<&'static str>,

    /// Sets the Domain attribute
    pub domain: Option<&'static str>,

    /// Sets the Max-Age attribute (in seconds)
    pub max_age: Option<i64>,

    /// Sets the Expires attribute as a UNIX timestamp in seconds
    pub expires: Option<i64>,
}

impl Default for CookieOptions {
    fn default() -> Self {
        Self {
            http_only: true,
            secure: true,
            same_site: CookieSameSiteOptions::None,
            path: Some("/"),
            domain: None,
            max_age: None,
            expires: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Cookie {
    pub name: &'static str,
    pub value: &'static str,
    pub(crate) options: CookieOptions,
}
