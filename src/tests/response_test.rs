#[cfg(test)]
mod tests {
    use crate::context::HttpResponse;
    use crate::response::BoxError;
    use crate::types::{HttpResponseError, ResponseContentBody, ResponseContentType};
    use bytes::Bytes;
    use futures::{stream, StreamExt};
    use hyper::{Body, StatusCode};
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

        let data = json!({"message": "test"});
        let response = HttpResponse::new()
            .set_header("X-Custom", "value")
            .set_cookie("session", "123")
            .status(201)
            .json(&data)
            .to_responder()
            .unwrap();

        assert_eq!(response.status(), 201);
        assert_eq!(response.headers().get("X-Custom").unwrap(), "value");
        assert_eq!(response.headers().get("Set-Cookie").unwrap(), "session=123");
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_text_response() {
        let text_body = "Hello, World!";
        let mut response = HttpResponse::new().text(text_body);
        response = response.set_header("x-custom", "value");

        assert_eq!(response.get_content_type(), ResponseContentType::TEXT);
        let response_2 = HttpResponse::new().text(text_body);

        if let ResponseContentBody::TEXT(body) = response_2.get_body() {
            assert_eq!(body, text_body);
        } else {
            panic!("Expected TEXT body");
        }

        assert_eq!(response.get_status_code(), 200);
        assert_eq!(response.get_content_type(), ResponseContentType::TEXT);
        assert_eq!(response.get_header("x-custom").unwrap(), "value");

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

        let response = HttpResponse::new()
            .set_header("X-Custom", "value")
            .html("<h1>Hello</h1>")
            .to_responder()
            .unwrap();

        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
        assert_eq!(response.headers().get("x-custom").unwrap(), "value");
    }

    #[test]
    fn test_headers() {
        let response = HttpResponse::new();
        let response = response.set_header("key", "value");
        assert_eq!(response.get_header("key").unwrap(), "value");
        assert_eq!(
            response.get_header("nonexistent"),
            Err(HttpResponseError::MissingHeader("nonexistent".to_string()))
        );
    }

    #[test]
    fn test_to_responder() {
        let response = HttpResponse::new().ok().text("OK");
        let hyper_res = response.to_responder();
        assert_eq!(hyper_res.unwrap().status(), StatusCode::OK);

        let response = HttpResponse::new().internal_server_error().text("Invalid");
        let hyper_res = response.to_responder();

        assert_eq!(
            hyper_res.unwrap().status(),
            StatusCode::INTERNAL_SERVER_ERROR
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

    #[tokio::test]
    async fn test_stream_response() {
        let stream = stream::iter(vec![Ok::<_, BoxError>(Bytes::from("test data"))]);
        let response = HttpResponse::new()
            .set_header("X-Custom", "value")
            .set_cookie("session", "123")
            .write(stream)
            .to_responder();

        assert_eq!(response.unwrap().status(), StatusCode::OK);

        let stream = stream::iter(vec![Ok::<_, BoxError>(Bytes::from("test data"))]);
        let response = HttpResponse::new()
            .set_header("X-Custom", "value")
            .set_cookie("session", "123")
            .write(stream)
            .to_responder();

        assert_eq!(
            response.unwrap().headers().get("content-type").unwrap(),
            "text/event-stream"
        );

        let stream = stream::iter(vec![Ok::<_, BoxError>(Bytes::from("test data"))]);
        let response = HttpResponse::new()
            .set_header("X-Custom", "value")
            .set_cookie("session", "123")
            .write(stream)
            .to_responder();
        assert_eq!(
            response.unwrap().headers().get("x-custom").unwrap(),
            "value"
        );

        let stream = stream::iter(vec![Ok::<_, BoxError>(Bytes::from("test data"))]);
        let response = HttpResponse::new()
            .set_header("X-Custom", "value")
            .set_cookie("session", "123")
            .write(stream)
            .to_responder();
        assert_eq!(
            response.unwrap().headers().get("connection").unwrap(),
            "keep-alive"
        );
    }
}
