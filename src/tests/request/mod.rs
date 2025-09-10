pub mod body;
pub mod data;
pub mod headers;
pub mod query_param;
pub mod route_params;

#[cfg(test)]
mod tests {
    use crate::{
        req::origin_url::Url,
        res::{CookieOptions, HttpResponse},
        types::{_HttpResponseError, ResponseContentBody, ResponseContentType},
    };
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
            .set_cookie("session", "123", Some(CookieOptions::default()))
            .created()
            .json(&data)
            .to_hyper_response()
            .unwrap();

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
    fn test_binary_response() {
        let bytes = vec![1, 2, 3, 4, 5];
        let response = HttpResponse::new().bytes(bytes.clone());
        assert_eq!(response.get_content_type(), &ResponseContentType::BINARY);
        if let ResponseContentBody::BINARY(body) = response.get_body() {
            assert_eq!(body, bytes);
        } else {
            panic!("Expected BINARY body");
        }

        // Edge case: Empty BINARY object
        let empty_bytes = vec![];
        let response = HttpResponse::new().bytes(empty_bytes.clone());
        if let ResponseContentBody::BINARY(body) = response.get_body() {
            assert_eq!(body, empty_bytes);
        } else {
            panic!("Expected BINARY body");
        }

        let data = vec![1, 2, 3, 4, 5];
        let response = HttpResponse::new()
            .set_header("X-Custom", "value")
            .set_cookie("session", "123", Some(CookieOptions::default()))
            .ok()
            .bytes(data)
            .to_hyper_response()
            .unwrap();

        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("X-Custom").unwrap(), "value");
        assert_eq!(
            response.headers().get("Set-Cookie").unwrap(),
            "session=123; HttpOnly; SameSite=None; Secure; Path=/; Max-Age=0"
        );
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/octet-stream"
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

        assert_eq!(response.get_status_code(), 200);
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
            .to_hyper_response()
            .unwrap();

        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
        assert_eq!(response.headers().get("x-custom").unwrap(), "value");
    }

    #[test]
    fn test_cookies() {
        let response = HttpResponse::new();
        let response = response.set_cookie("key", "value", Some(CookieOptions::default()));
        assert_eq!(response.get_cookie("key").unwrap(), "value");

        let response = HttpResponse::new()
            .set_cookie("session", "123", Some(CookieOptions::default()))
            .set_cookie("another_cookie", "123", Some(CookieOptions::default()))
            .clear_cookie("old_session")
            .ok()
            .text("test");

        let cookies: Vec<_> = response.cookies;
        let remove_cookies: Vec<_> = response.remove_cookies;
        assert_eq!(cookies.len(), 2);

        let session_cookie = cookies.iter().find(|c| c.name == "session").unwrap();
        assert_eq!(session_cookie.value, "123");

        let cleared_cookie = remove_cookies
            .iter()
            .find(|c| c == &&"old_session")
            .unwrap();

        assert_eq!(*cleared_cookie, "old_session");
    }

    #[test]
    fn test_to_responder() {
        let response = HttpResponse::new().ok().text("OK");
        let hyper_response = response.to_hyper_response().unwrap();
        assert_eq!(hyper_response.status(), 200);

        let response = HttpResponse::new().internal_server_error().text("Invalid");
        let hyper_response = response.to_hyper_response().unwrap();

        assert_eq!(hyper_response.status(), 500);
    }

    #[test]
    fn test_clear_cookie() {
        let response = HttpResponse::new();
        let response = response.set_cookie("session", "abc123", None);

        assert_eq!(response.get_cookie("session").unwrap(), "abc123");
        let response = HttpResponse::new();
        let response = response.set_cookie("session", "abc123", None);
        let response = response.clear_cookie("session");

        // Verify cookie is removed
        assert_eq!(response.get_cookie("session"), None);

        let response = HttpResponse::new();

        let response = response.set_cookie("session", "abc123", None);

        let response = response.clear_cookie("non-existent");

        assert_eq!(response.get_cookie("non-existent"), None);
    }

    #[test]
    fn test_response_error() {
        let err_1 = _HttpResponseError::MissingHeader("id".to_string());
        assert_eq!(err_1.to_string(), "Header id doesn't exist");
    }

    #[test]
    fn test_new_and_as_str() {
        let url = Url::new("https://example.com/path");
        assert_eq!(url.as_str(), "https://example.com/path");
    }

    #[test]
    fn test_value() {
        let url = Url::new("https://example.com/abc");
        assert_eq!(url.value(), &"https://example.com/abc".to_string());
    }

    #[test]
    fn test_display_trait() {
        let url = Url::new("https://display.com");
        let s = format!("{}", url);
        assert_eq!(s, "https://display.com");
    }

    #[test]
    fn test_debug_trait() {
        let url = Url::new("https://debug.com");
        let s = format!("{:?}", url);
        assert!(s.contains("Url"));
        assert!(s.contains("https://debug.com"));
    }

    #[test]
    fn test_clone_and_eq() {
        let url1 = Url::new("https://clone.com");
        let url2 = url1.clone();
        assert_eq!(url1, url2);
    }

    #[test]
    fn test_serde_serialize_deserialize() {
        let url = Url::new("https://serde.com");
        let serialized = serde_json::to_string(&url).unwrap();
        assert!(serialized.contains("serde.com"));

        let deserialized: Url = serde_json::from_str(&serialized).unwrap();
        assert_eq!(url, deserialized);
    }
}
