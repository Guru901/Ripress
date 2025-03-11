use actix_web::Responder;

pub struct HttpResponse {
    status_code: i32,
    body: String,
}

impl HttpResponse {
    pub fn new() -> Self {
        HttpResponse {
            status_code: 200,
            body: String::new(),
        }
    }

    pub fn status(mut self, code: i32) -> Self {
        self.status_code = code;
        return self;
    }

    pub fn json(mut self, json: serde_json::Value) -> Self {
        self.body = json.to_string();
        return self;
    }

    pub fn text(mut self, text: String) -> Self {
        self.body = text;
        return self;
    }

    pub fn to_responder(self) -> actix_web::HttpResponse {
        let body = self.body;
        actix_web::http::StatusCode::from_u16(self.status_code as u16)
            .map(|status| actix_web::HttpResponse::build(status).body(body))
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
