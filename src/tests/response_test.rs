#[cfg(test)]
mod tests {
    use crate::response::{ContentBody, ContentType, HttpResponse};
    use serde_json::json;

    #[test]
    fn test_default_response() {
        let response = HttpResponse::new();
        assert_eq!(response.get_status_code(), 200);
        assert_eq!(response.get_content_type(), ContentType::JSON);
    }

    #[test]
    fn test_status_code() {
        let response = HttpResponse::new().status(404);
        assert_eq!(response.get_status_code(), 404);
    }

    #[test]
    fn test_json_response() {
        let json_body = json!({"key": "value"});
        let response = HttpResponse::new().json(json_body.clone());
        assert_eq!(response.get_content_type(), ContentType::JSON);
        if let ContentBody::JSON(body) = response.get_body() {
            assert_eq!(body, json_body);
        } else {
            panic!("Expected JSON body");
        }
    }

    #[test]
    fn test_text_response() {
        let text_body = "Hello, World!";
        let response = HttpResponse::new().text(text_body);
        assert_eq!(response.get_content_type(), ContentType::TEXT);
        if let ContentBody::TEXT(body) = response.get_body() {
            assert_eq!(body, text_body);
        } else {
            panic!("Expected TEXT body");
        }
    }

    #[test]
    fn test_to_responder() {
        let response = HttpResponse::new().ok().text("OK");
        let actix_response = response.to_responder();
        assert_eq!(actix_response.status(), actix_web::http::StatusCode::OK);
    }
}
