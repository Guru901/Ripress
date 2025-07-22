pub enum ResponseContentBody<'a> {
    TEXT(&'a str),
}

impl<'a> ResponseContentBody<'a> {
    pub fn new_text<T: Into<&'a str>>(text: T) -> Self {
        ResponseContentBody::TEXT(text.into())
    }
}
pub enum ResponseContentType {
    TEXT,
}
