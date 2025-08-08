#[cfg(test)]
mod tests {
    use crate::context::HttpResponse;
    use crate::res::CookieOptions;
    use crate::types::{HttpResponseError, ResponseContentBody, ResponseContentType};
    use actix_web::Responder;
    use serde_json::json;

    #[test]
    fn test_default_response() {
        let response = HttpResponse::new();
        assert_eq!(
            response.status_code,
            crate::res::response_status::StatusCode::Ok
        );

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
        assert_eq!(response.get_content_type(), &ResponseContentType::JSON);
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

        let data = json!({"message": "test"});
        let response = HttpResponse::new()
            .set_header("X-Custom", "value")
            .set_cookie("session", "123", CookieOptions::default())
            .created()
            .json(&data)
            .to_responder();

        assert_eq!(response.status(), 201);
        assert_eq!(response.headers().get("X-Custom").unwrap(), "value");
        assert_eq!(
            response.headers().get("Set-Cookie").unwrap(),
            "session=123; HttpOnly; SameSite=None; Secure; Path=/; Max-Age=0"
        );
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_text_response() {
        let text_body = "Hello, World!";
        let response = HttpResponse::new().text(text_body);

        assert_eq!(response.get_content_type(), &ResponseContentType::TEXT);
        let response_2 = HttpResponse::new().text(text_body);

        if let ResponseContentBody::TEXT(body) = response_2.get_body() {
            assert_eq!(body, text_body);
        } else {
            panic!("Expected TEXT body");
        }

        assert_eq!(response.get_status_code(), actix_web::http::StatusCode::OK);
        assert_eq!(response.get_content_type(), &ResponseContentType::TEXT);

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
        assert_eq!(response.get_content_type(), &ResponseContentType::HTML);
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

        let response = HttpResponse::new()
            .set_header("X-Custom", "value")
            .html("<h1>Hello</h1>")
            .to_responder();

        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
        assert_eq!(response.headers().get("x-custom").unwrap(), "value");
    }

    #[test]
    fn test_cookies() {
        let response = HttpResponse::new();
        let response = response.set_cookie("key", "value", CookieOptions::default());
        assert_eq!(response.get_cookie("key").unwrap(), "value");

        let response = HttpResponse::new()
            .set_cookie("session", "123", CookieOptions::default())
            .clear_cookie("old_session")
            .ok()
            .text("test")
            .to_responder();

        let cookies: Vec<_> = response.cookies().collect();
        assert_eq!(cookies.len(), 2);

        let session_cookie = cookies.iter().find(|c| c.name() == "session").unwrap();
        assert_eq!(session_cookie.value(), "123");

        let cleared_cookie = cookies.iter().find(|c| c.name() == "old_session").unwrap();
        assert_eq!(cleared_cookie.value(), "");
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
    fn test_clear_cookie() {
        let response = HttpResponse::new();
        let response = response.set_cookie("session", "abc123", CookieOptions::default());

        assert_eq!(response.get_cookie("session").unwrap(), "abc123");
        let response = HttpResponse::new();
        let response = response.set_cookie("session", "abc123", CookieOptions::default());
        let response = response.clear_cookie("session");

        // Verify cookie is removed
        assert_eq!(response.get_cookie("session"), None);

        let response = HttpResponse::new();

        let response = response.set_cookie("session", "abc123", CookieOptions::default());

        let response = response.clear_cookie("non-existent");

        assert_eq!(response.get_cookie("non-existent"), None);
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
