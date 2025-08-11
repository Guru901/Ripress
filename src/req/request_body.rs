use crate::req::body::FormData;

#[derive(Debug, Clone)]
pub struct RequestBody {
    pub content: RequestBodyContent,
    pub content_type: RequestBodyType,
}

impl RequestBody {
    pub fn new_text<T: Into<String>>(text: T) -> Self {
        RequestBody {
            content_type: RequestBodyType::TEXT,
            content: RequestBodyContent::TEXT(text.into()),
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

#[derive(Debug, Clone)]
pub enum RequestBodyContent {
    TEXT(String),
    JSON(serde_json::Value),
    FORM(FormData),
    EMPTY,
}
