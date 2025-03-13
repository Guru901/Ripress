use serde::Serialize;

// HttpRequest types

#[derive(Debug, Clone, PartialEq)]
pub enum RequestBodyType {
    JSON,
    TEXT,
    FORM,
}

impl Copy for RequestBodyType {}

#[derive(Debug, Clone)]
pub enum RequestBodyContent {
    TEXT(String),
    JSON(serde_json::Value),
    FORM(String),
}

// HttpResponse types

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum ResponseContentType {
    JSON,
    TEXT,
}

#[derive(Serialize)]
pub(crate) enum ResponseContentBody {
    JSON(serde_json::Value),
    TEXT(String),
}

impl ResponseContentBody {
    pub fn new_text<T: Into<String>>(text: T) -> Self {
        ResponseContentBody::TEXT(text.into())
    }
}
