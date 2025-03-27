#[cfg(test)]
mod tests {
    use crate::response::HttpResponse;
    use crate::types::HttpResponseError::MissingHeader;
    use crate::types::{HttpResponseError, ResponseContentBody, ResponseContentType};
    use actix_web::Responder;
    use bytes::Bytes;
    use futures::{stream, StreamExt};
    use serde_json::json;

    #[test]
    fn test_default_response() {
        let response = HttpResponse::new();
        assert_eq!(response.get_status_code(), 200);
        assert_eq!(response.get_content_type(), ResponseContentType::JSON);

        // Edge case: Check default body content
        if let ResponseContentBody::TEXT(body) = response.get_body() {
            assert_eq!(body, "");
        } else {
            panic!("Expected TEXT body");
        }
    }

    #[test]
    fn test_status_code() {
        let response = HttpResponse::new().status(200);
        assert_eq!(response.get_status_code(), 200);

        let response = HttpResponse::new().status(999);
        assert_eq!(response.get_status_code(), 999);
    }

    #[tokio::test]
    async fn test_redirect_response() {
        let redirect_url = "https://example.com";
        let response = HttpResponse::new().redirect(redirect_url);

        // Test that status code isx 302 (Found/Redirect)
        assert_eq!(response.get_status_code(), 302);

        assert_eq!(response.get_header("Location").unwrap(), redirect_url);
    }

    #[tokio::test]
    async fn test_stream() {
        let stream = stream::iter(0..5)
            .map(|n| Ok::<Bytes, std::io::Error>(Bytes::from(format!("Number: {}\n", n))));

        let response = HttpResponse::new().write(stream);

        assert_eq!(response.get_status_code(), 200);
        assert_eq!(response.is_stream, true);
    }

    #[tokio::test]
    async fn test_redirect_with_chaining() {
        let redirect_url = "https://example.com";
        let response = HttpResponse::new()
            .set_header("X-Custom", "test")
            .redirect(redirect_url);

        assert_eq!(response.get_status_code(), 302);

        assert_eq!(response.get_header("Location").unwrap(), redirect_url);

        assert_eq!(response.get_header("X-Custom").unwrap(), "test");
    }

    #[test]
    fn test_status_code_helpers() {
        let response = HttpResponse::new();
        assert_eq!(response.ok().get_status_code(), 200);

        let response = HttpResponse::new();
        assert_eq!(response.bad_request().get_status_code(), 400);

        let response = HttpResponse::new();
        assert_eq!(response.internal_server_error().get_status_code(), 500);

        let response = HttpResponse::new();
        assert_eq!(response.not_found().get_status_code(), 404);

        let response = HttpResponse::new();
        assert_eq!(response.unauthorized().get_status_code(), 401);
    }

    #[test]
    fn test_json_response() {
        let json_body = json!({"key": "value"});
        let response = HttpResponse::new().json(json_body.clone());
        assert_eq!(response.get_content_type(), ResponseContentType::JSON);
        if let ResponseContentBody::JSON(body) = response.get_body() {
            assert_eq!(body, json_body);
        } else {
            panic!("Expected JSON body");
        }

        // Edge case: Empty JSON object
        let empty_json = json!({});
        let response = HttpResponse::new().json(empty_json.clone());
        if let ResponseContentBody::JSON(body) = response.get_body() {
            assert_eq!(body, empty_json);
        } else {
            panic!("Expected JSON body");
        }
    }

    #[test]
    fn test_text_response() {
        let text_body = "Hello, World!";
        let response = HttpResponse::new().text(text_body);
        assert_eq!(response.get_content_type(), ResponseContentType::TEXT);
        if let ResponseContentBody::TEXT(body) = response.get_body() {
            assert_eq!(body, text_body);
        } else {
            panic!("Expected TEXT body");
        }

        // Edge case: Empty text body
        let response = HttpResponse::new().text("");
        if let ResponseContentBody::TEXT(body) = response.get_body() {
            assert_eq!(body, "");
        } else {
            panic!("Expected TEXT body");
        }
    }

    #[test]
    fn test_html_response() {
        let text_body = "<h1>Hello, World!</h1>";
        let response = HttpResponse::new().html(text_body);
        assert_eq!(response.get_content_type(), ResponseContentType::HTML);
        if let ResponseContentBody::HTML(body) = response.get_body() {
            assert_eq!(body, text_body);
        } else {
            panic!("Expected TEXT body");
        }

        // Edge case: Empty text body
        let response = HttpResponse::new().html("");
        if let ResponseContentBody::HTML(body) = response.get_body() {
            assert_eq!(body, "");
        } else {
            panic!("Expected TEXT body");
        }
    }

    #[test]
    fn test_cookies() {
        let response = HttpResponse::new();
        let response = response.set_cookie("key", "value");
        assert_eq!(response.get_cookie("key".to_string()).unwrap(), "value");
    }

    #[test]
    fn test_headers() {
        let response = HttpResponse::new();
        let response = response.set_header("key", "value");
        assert_eq!(response.get_header("key").unwrap(), "value");
        assert_eq!(
            response.get_header("nonexistent"),
            Err(MissingHeader("nonexistent".to_string()))
        );
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

    #[test]
    fn test_set_content_type() {
        let response = HttpResponse::new();
        let response = response.set_content_type(ResponseContentType::JSON);
        assert_eq!(response.get_content_type(), ResponseContentType::JSON);
    }
    #[test]
    fn test_clear_cookie() {
        let response = HttpResponse::new();
        let response = response.set_cookie("session", "abc123");

        assert_eq!(
            response.get_cookie("session".to_string()).unwrap(),
            "abc123"
        );
        let response = HttpResponse::new();
        let response = response.set_cookie("session", "abc123");
        let response = response.clear_cookie("session");

        // Verify cookie is removed
        assert_eq!(response.get_cookie("session".to_string()), None);

        let response = HttpResponse::new();

        let response = response.set_cookie("session", "abc123");

        let response = response.clear_cookie("non-existent");

        assert_eq!(response.get_cookie("non-existent".to_string()), None);
    }

    #[test]
    fn test_response_error() {
        let err_1 = HttpResponseError::MissingHeader("id".to_string());

        assert_eq!(err_1.to_string(), "Header id doesnt exist");
    }

    #[test]
    fn test_respond_to() {
        let response = HttpResponse::new().ok().text("OK");
        let acitx_req = actix_web::test::TestRequest::default().to_http_request();
        let responder = response.respond_to(&acitx_req);

        assert_eq!(responder.status(), 200);

        let response = HttpResponse::new()
            .internal_server_error()
            .text("internal server error");
        let acitx_req = actix_web::test::TestRequest::default().to_http_request();
        let responder = response.respond_to(&acitx_req);

        assert_eq!(responder.status(), 500);

        let response = HttpResponse::new().unauthorized();
        let acitx_req = actix_web::test::TestRequest::default().to_http_request();
        let responder = response.respond_to(&acitx_req);

        assert_eq!(responder.status(), 401);
    }
}
