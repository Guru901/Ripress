#[cfg(test)]
mod tests {
    use crate::response::{ContentBody, ContentType, HttpResponse};
    use serde_json::json;

    #[test]
    fn test_default_response() {
        let response = HttpResponse::new();
        assert_eq!(response.get_status_code(), 200);
        assert_eq!(response.get_content_type(), ContentType::JSON);

        // Edge case: Check default body content
        if let ContentBody::TEXT(body) = response.get_body() {
            assert_eq!(body, "");
        } else {
            panic!("Expected TEXT body");
        }
    }

    #[test]
    fn test_status_code() {
        let response = HttpResponse::new().status(404);
        assert_eq!(response.get_status_code(), 404);

        // Edge case: Invalid status code
        let response = HttpResponse::new().status(999);
        assert_eq!(response.get_status_code(), 999); // Assuming the implementation allows any integer
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

        // Edge case: Empty JSON object
        let empty_json = json!({});
        let response = HttpResponse::new().json(empty_json.clone());
        if let ContentBody::JSON(body) = response.get_body() {
            assert_eq!(body, empty_json);
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

        // Edge case: Empty text body
        let response = HttpResponse::new().text("");
        if let ContentBody::TEXT(body) = response.get_body() {
            assert_eq!(body, "");
        } else {
            panic!("Expected TEXT body");
        }
    }

    #[test]
    fn test_to_responder() {
        let response = HttpResponse::new().ok().text("OK");
        let actix_response = response.to_responder();
        assert_eq!(actix_response.status(), actix_web::http::StatusCode::OK);

        let response = HttpResponse::new().internal_server_error().text("Invalid");
        let actix_response = response.to_responder();
        assert_eq!(
            actix_response.status(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
