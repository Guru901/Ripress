#[derive(Debug, Clone)]
pub struct RequestBody {
    pub content: RequestBodyContent,
    pub content_type: RequestBodyType,
}

pub mod form_data;
pub mod text_data;

pub use form_data::FormData;
pub use text_data::{TextData, TextDataError};

impl RequestBody {
    pub fn new_text(text: TextData) -> Self {
        RequestBody {
            content_type: RequestBodyType::TEXT,
            content: RequestBodyContent::TEXT(text),
        }
    }

    pub fn new_form(form_data: FormData) -> Self {
        RequestBody {
            content_type: RequestBodyType::FORM,
            content: RequestBodyContent::FORM(form_data),
        }
    }

    pub fn new_json<T: Into<serde_json::Value>>(json: T) -> Self {
        RequestBody {
            content_type: RequestBodyType::JSON,
            content: RequestBodyContent::JSON(json.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestBodyType {
    JSON,
    TEXT,
    FORM,
    EMPTY,
}

impl Copy for RequestBodyType {}

impl ToString for RequestBodyType {
    fn to_string(&self) -> String {
        match self {
            RequestBodyType::JSON => "application/json".to_string(),
            RequestBodyType::TEXT => "text/plain".to_string(),
            RequestBodyType::FORM => "application/x-www-form-urlencoded".to_string(),
            RequestBodyType::EMPTY => "".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RequestBodyContent {
    TEXT(TextData),
    JSON(serde_json::Value),
    FORM(FormData),
    EMPTY,
}
