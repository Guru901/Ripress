use serde::Serialize;

pub enum ResponseContentBody<'a> {
    TEXT(&'a str),
    JSON(serde_json::Value),
}

impl<'a> ResponseContentBody<'a> {
    pub fn new_text<T: Into<&'a str>>(text: T) -> Self {
        ResponseContentBody::TEXT(text.into())
    }

    pub fn new_json<T: Serialize>(json: T) -> Self {
        let value = serde_json::to_value(json).expect("Failed to serialize to JSON");
        ResponseContentBody::JSON(value)
    }
}
pub enum ResponseContentType {
    TEXT,
    JSON,
}
