use actix_web::Responder;
use serde::Serialize;

pub struct HttpResponse {
    status_code: i32,
    body: ContentBody,
    content_type: ContentType,
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum ContentType {
    JSON,
    TEXT,
}

#[derive(Serialize)]
pub(crate) enum ContentBody {
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

    pub fn ok(mut self) -> Self {
        self.status_code = 200;
        return self;
    }

    pub fn bad_request(mut self) -> Self {
        self.status_code = 400;
        return self;
    }

    pub fn not_found(mut self) -> Self {
        self.status_code = 401;
        return self;
    }

    pub fn internal_server_error(mut self) -> Self {
        self.status_code = 500;
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

#[cfg(test)]
impl HttpResponse {
    pub fn get_status_code(&self) -> i32 {
        self.status_code
    }

    pub fn get_content_type(&self) -> ContentType {
        self.content_type.clone()
    }

    pub fn get_body(self) -> ContentBody {
        self.body
    }
}
