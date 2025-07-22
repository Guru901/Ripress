pub struct HttpRequest {}

impl HttpRequest {
    pub fn from_actix_request(_req: actix_web::HttpRequest) -> Self {
        HttpRequest {}
    }
}
