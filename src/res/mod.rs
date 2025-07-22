use crate::types::{ResponseContentBody, ResponseContentType};
use actix_web::Responder;
use serde::Serialize;

pub struct HttpResponse {
    body: ResponseContentBody,
    content_type: ResponseContentType,
    status_code: u16,
}

impl HttpResponse {
    pub fn new() -> Self {
        Self {
            status_code: 200,
            body: ResponseContentBody::TEXT(String::new()),
            content_type: ResponseContentType::TEXT,
        }
    }
    pub fn text<T: Into<String>>(mut self, text: T) -> Self {
        self.body = ResponseContentBody::new_text(text);
        self.content_type = ResponseContentType::TEXT;
        return self;
    }

    pub fn json<T: Serialize>(mut self, json: T) -> Self {
        self.body = ResponseContentBody::new_json(json);
        self.content_type = ResponseContentType::JSON;
        return self;
    }

    pub fn status(mut self, code: u16) -> Self {
        self.status_code = code;
        self
    }

    pub fn to_responder(self) -> actix_web::HttpResponse {
        let actix_res = actix_web::http::StatusCode::from_u16(self.status_code as u16)
            .map(|status| match self.body {
                ResponseContentBody::JSON(json) => actix_web::HttpResponse::build(status)
                    .content_type("application/json")
                    .json(json),
                ResponseContentBody::TEXT(text) => actix_web::HttpResponse::build(status)
                    .content_type("text/plain")
                    .body(text),
            })
            .unwrap_or_else(|_| {
                actix_web::HttpResponse::InternalServerError().body("Invalid status code")
            });

        return actix_res;
    }
}

impl Responder for HttpResponse {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse {
        self.to_responder()
    }
}
