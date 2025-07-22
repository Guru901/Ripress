use crate::types::{ResponseContentBody, ResponseContentType};

pub struct HttpResponse<'a> {
    body: ResponseContentBody<'a>,
    content_type: ResponseContentType,
}

impl<'a> HttpResponse<'a> {
    pub fn text<T: Into<&'a str>>(mut self, text: T) -> Self {
        self.body = ResponseContentBody::new_text(text);
        self.content_type = ResponseContentType::TEXT;
        return self;
    }
}
