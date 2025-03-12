use actix_web::Responder;
use serde::Serialize;

pub struct HttpResponse {
    status_code: i32,
    body: ContentBody,
    content_type: ContentType,
}

#[derive(PartialEq)]
enum ContentType {
    JSON,
    TEXT,
}

#[derive(Serialize)]
enum ContentBody {
    JSON(serde_json::Value),
    TEXT(String),
}

impl ContentBody {
    fn new_text<T: Into<String>>(text: T) -> Self {
        ContentBody::TEXT(text.into())
    }
}

impl HttpResponse {
    pub fn new() -> Self {
        HttpResponse {
            status_code: 200,
            body: ContentBody::TEXT(String::new()),
            content_type: ContentType::JSON,
        }
    }

    pub fn status(mut self, code: i32) -> Self {
        self.status_code = code;
        return self;
    }

    pub fn json(mut self, json: serde_json::Value) -> Self {
        self.body = ContentBody::JSON(json);
        self.content_type = ContentType::JSON;
        return self;
    }

    pub fn text<T: Into<String>>(mut self, text: T) -> Self {
        self.body = ContentBody::new_text(text);
        self.content_type = ContentType::TEXT;
        return self;
    }

    pub fn to_responder(self) -> actix_web::HttpResponse {
        let body = self.body;
        actix_web::http::StatusCode::from_u16(self.status_code as u16)
            .map(|status| match body {
                ContentBody::JSON(json) => actix_web::HttpResponse::build(status)
                    .content_type("application/json")
                    .json(json),
                ContentBody::TEXT(text) => actix_web::HttpResponse::build(status)
                    .content_type("text/plain")
                    .body(text),
            })
            .unwrap_or_else(|_| {
                actix_web::HttpResponse::InternalServerError().body("Invalid status code")
            })
    }
}

impl Responder for HttpResponse {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse {
        self.to_responder()
    }
}
