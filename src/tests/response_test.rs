#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::error::Error;

    use crate::context::HttpRequest;
    use crate::req::body::RequestBodyType;
    use crate::req::body::TextData;
    use crate::req::determine_content_type;
    use crate::req::origin_url::Url;
    use crate::res::Cookie;
    use crate::res::CookieOptions;
    use crate::res::HttpResponse;
    use crate::res::ResponseError;
    use crate::res::response_headers::ResponseHeaders;
    use crate::res::response_status::StatusCode;
    use crate::types::HttpMethods;
    use crate::types::HttpRequestError;
    use futures::stream;
    use hyper::Method;
    use serde_json::json;

    #[test]
    fn test_httpmethods_display() {
        assert_eq!(HttpMethods::GET.to_string(), "GET");
        assert_eq!(HttpMethods::POST.to_string(), "POST");
        assert_eq!(HttpMethods::PUT.to_string(), "PUT");
        assert_eq!(HttpMethods::DELETE.to_string(), "DELETE");
        assert_eq!(HttpMethods::PATCH.to_string(), "PATCH");
        assert_eq!(HttpMethods::OPTIONS.to_string(), "OPTIONS");
        assert_eq!(HttpMethods::HEAD.to_string(), "HEAD");
    }

    #[test]
    fn test_httpmethods_from() {
        let method = HttpMethods::from(&Method::GET);
        assert_eq!(method, HttpMethods::GET);

        let method = HttpMethods::from(&Method::POST);
        assert_eq!(method, HttpMethods::POST);

        let method = HttpMethods::from(&Method::PUT);
        assert_eq!(method, HttpMethods::PUT);

        let method = HttpMethods::from(&Method::DELETE);
        assert_eq!(method, HttpMethods::DELETE);

        let method = HttpMethods::from(&Method::PATCH);
        assert_eq!(method, HttpMethods::PATCH);

        let method = HttpMethods::from(&Method::OPTIONS);
        assert_eq!(method, HttpMethods::OPTIONS);

        let method = HttpMethods::from(&Method::HEAD);
        assert_eq!(method, HttpMethods::HEAD);

        let method = HttpMethods::from(&Method::CONNECT);
        assert_eq!(method, HttpMethods::GET);

        let method = HttpMethods::from(&Method::TRACE);
        assert_eq!(method, HttpMethods::GET);
    }

    #[test]
    fn test_status_code_helpers() {
        let response = HttpResponse::new().accepted();
        assert_eq!(response.status_code.as_u16(), 202);
        assert_eq!(response.status_code.canonical_reason(), "Accepted");

        let response = HttpResponse::new().no_content();
        assert_eq!(response.status_code.as_u16(), 204);
        assert_eq!(response.status_code.canonical_reason(), "No Content");

        let response = HttpResponse::new().forbidden();
        assert_eq!(response.status_code.as_u16(), 403);
        assert_eq!(response.status_code.canonical_reason(), "Forbidden");

        let response = HttpResponse::new().method_not_allowed();
        assert_eq!(response.status_code.as_u16(), 405);
        assert_eq!(
            response.status_code.canonical_reason(),
            "Method Not Allowed"
        );

        let response = HttpResponse::new().conflict();
        assert_eq!(response.status_code.as_u16(), 409);
        assert_eq!(response.status_code.canonical_reason(), "Conflict");

        let response = HttpResponse::new().not_implemented();
        assert_eq!(response.status_code.as_u16(), 501);
        assert_eq!(response.status_code.canonical_reason(), "Not Implemented");

        let response = HttpResponse::new().bad_gateway();
        assert_eq!(response.status_code.as_u16(), 502);
        assert_eq!(response.status_code.canonical_reason(), "Bad Gateway");

        let response = HttpResponse::new().service_unavailable();
        assert_eq!(response.status_code.as_u16(), 503);
        assert_eq!(
            response.status_code.canonical_reason(),
            "Service Unavailable"
        );
    }

    #[test]
    fn test_get_query() {
        let mut req = HttpRequest::new();
        req.set_query("q", "Ripress");

        assert_eq!(req.query.get("q"), Some("Ripress"));

        assert_eq!(req.query.get("nonexistent"), None);
    }

    #[test]
    fn test_get_param() {
        let mut req = HttpRequest::new();
        req.set_param("q", "Ripress");

        assert_eq!(req.params.get("q"), Some("Ripress"));

        assert_eq!(req.params.get("nonexistent"), None);
    }

    #[test]
    fn test_json_body() {
        // Test 1 - Everything Is Correct
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct User {
            id: u32,
            name: String,
        }

        let mut req = HttpRequest::new();

        req.set_json(
            User {
                id: 1,
                name: "John Doe".to_string(),
            },
            RequestBodyType::JSON,
        );

        assert_eq!(
            req.json::<User>().unwrap(),
            User {
                id: 1,
                name: "John Doe".to_string()
            }
        );

        // Test 2 - Invalid Body Type

        req.set_json(
            User {
                id: 1,
                name: "John Doe".to_string(),
            },
            RequestBodyType::FORM,
        );

        assert!(req.json::<User>().is_err());

        // Test 3 - Invalid JSON Content

        req.set_text(
            TextData::new("{invalid json}".to_string()),
            RequestBodyType::JSON,
        );

        assert!(req.json::<User>().is_err());
    }

    #[test]
    fn test_text_body() {
        // Test 1 - Everything Is Correct

        let mut req = HttpRequest::new();

        req.set_text(TextData::new("Ripress".to_string()), RequestBodyType::TEXT);

        assert_eq!(req.text().unwrap().to_string(), "Ripress".to_string());

        // Test 2 - Invalid Body Type

        req.set_text(TextData::new("".to_string()), RequestBodyType::JSON);

        assert!(req.text().is_err());

        // Test 3 - Invalid Text Content

        req.set_json(json!({"key": "value"}), RequestBodyType::TEXT);

        assert!(req.text().is_err());
    }

    #[test]
    fn test_form_data() {
        // Test 1 - Everything Is Correct

        let mut req = HttpRequest::new();
        req.set_form("key", "value", RequestBodyType::FORM);

        assert_eq!(req.form_data().unwrap().get("key").unwrap(), "value");
        assert_eq!(req.form_data().unwrap().get("nonexistent"), None);

        // Test 2 - Invalid Body Type

        req.set_form("another_key", "another_value", RequestBodyType::JSON);
        assert!(req.form_data().is_err());

        // // Test 3 - Invalid Form Content

        req.set_json(json!({"key": "value"}), RequestBodyType::FORM);
        assert!(req.form_data().is_err());
    }

    #[test]
    fn test_header() {
        let mut req = HttpRequest::new();
        req.set_header("key", "value");

        assert_eq!(req.headers.get("key").unwrap(), "value");
        assert_eq!(req.headers.get("nonexistent"), None);

        req.set_header("another_key", "another_value");
        let header = req.headers.get("another_key").unwrap();
        assert_eq!(header, "another_value");
    }

    #[test]
    fn text_cookie() {
        let mut req = HttpRequest::new();
        req.set_cookie("key", "value");

        assert_eq!(req.get_cookie("key").unwrap(), "value");
        assert_eq!(req.get_cookie("nonexistent"), None);

        req.set_cookie("another_key", "another_value");
        let cookie = req.get_cookie("another_key").unwrap();
        assert_eq!(cookie, "another_value");
    }

    #[test]
    fn test_is_method() {
        let mut req = HttpRequest::new();

        req.set_content_type(RequestBodyType::JSON);
        assert!(req.is(RequestBodyType::JSON));

        req.set_content_type(RequestBodyType::FORM);
        assert!(req.is(RequestBodyType::FORM));

        req.set_content_type(RequestBodyType::TEXT);
        assert!(req.is(RequestBodyType::TEXT));

        req.set_content_type(RequestBodyType::TEXT);
        assert_ne!(req.is(RequestBodyType::FORM), true);
    }

    #[test]
    fn test_get_method() {
        let mut req = HttpRequest::new();

        req.set_method(HttpMethods::GET);
        assert_eq!(req.method, HttpMethods::GET);

        req.set_method(HttpMethods::POST);
        assert_eq!(req.method, HttpMethods::POST);

        req.set_method(HttpMethods::PUT);
        assert_eq!(req.method, HttpMethods::PUT);

        req.set_method(HttpMethods::DELETE);
        assert_eq!(req.method, HttpMethods::DELETE);

        req.set_method(HttpMethods::DELETE);
        assert_ne!(req.method, HttpMethods::GET);
    }

    #[test]
    fn test_get_path() {
        let mut req = HttpRequest::new();
        req.set_path("/user/1".to_string());

        assert_eq!(req.path, "/user/1");
    }

    #[test]
    fn test_origin_url() {
        let mut req = HttpRequest::new();

        req.set_origin_url(Url::new("value"));

        assert_eq!(req.origin_url, Url::new("value"));

        req.set_origin_url(Url::new("/user/1?q=hello"));
        assert_eq!(req.origin_url, Url::new("/user/1?q=hello"));
    }

    #[test]
    fn test_content_type() {
        let content_type = determine_content_type("application/json");
        assert_eq!(content_type, RequestBodyType::JSON);

        let content_type = determine_content_type("");
        assert_eq!(content_type, RequestBodyType::BINARY);

        let content_type = determine_content_type("application/x-www-form-urlencoded");
        assert_eq!(content_type, RequestBodyType::FORM);

        let content_type = determine_content_type("application/octet-stream");
        assert_eq!(content_type, RequestBodyType::BINARY);

        let content_type = determine_content_type("image/png");
        assert_eq!(content_type, RequestBodyType::BINARY);

        let content_type = determine_content_type("application/xml");
        assert_eq!(content_type, RequestBodyType::TEXT);
    }

    #[test]
    fn test_error_enum() {
        let err_1 = HttpRequestError::MissingParam("id".to_string());
        let err_2 = HttpRequestError::MissingQuery("id".to_string());
        let err_3 = HttpRequestError::MissingCookie("id".to_string());
        let err_4 = HttpRequestError::MissingHeader("id".to_string());
        let err_5 = HttpRequestError::InvalidJson("id".to_string());

        assert_eq!(err_1.to_string(), "Param id doesn't exist");
        assert_eq!(err_2.to_string(), "Query id doesn't exist");
        assert_eq!(err_3.to_string(), "Cookie id doesn't exist");
        assert_eq!(err_4.to_string(), "Header id doesn't exist");
        assert_eq!(err_5.to_string(), "JSON is invalid: id");
    }

    #[test]
    fn test_set_and_get_data() {
        let mut req = HttpRequest::new();
        req.set_data("id", "123");
        assert_eq!(req.get_data("id"), Some(String::from("123")));
        assert_eq!(req.get_data("nonexistent"), None);
    }

    #[test]
    fn test_basic_operations() {
        let mut headers = ResponseHeaders::new();
        headers.insert("Content-Type", "application/json");
        headers.insert("X-Custom", "test-value");

        assert_eq!(headers.get("content-type"), Some("application/json"));
        assert_eq!(headers.get("CONTENT-TYPE"), Some("application/json")); // case insensitive
        assert_eq!(headers.get("x-custom"), Some("test-value"));
        assert!(headers.contains_key("Content-Type"));
    }

    #[test]
    fn test_convenience_methods() {
        let mut headers = ResponseHeaders::new();
        headers.json();
        headers.content_length(1024);
        headers.location("https://example.com");

        assert_eq!(headers.get("content-type"), Some("application/json"));
        assert_eq!(headers.get("content-length"), Some("1024"));
        assert_eq!(headers.get("location"), Some("https://example.com"));
    }

    #[test]
    fn test_cors_headers() {
        let mut headers = ResponseHeaders::new();
        headers.cors_simple(Some("https://example.com"));

        assert_eq!(
            headers.get("access-control-allow-origin"),
            Some("https://example.com")
        );
        assert!(headers.get("access-control-allow-methods").is_some());
        assert!(headers.get("access-control-allow-headers").is_some());
    }

    #[test]
    fn test_security_headers() {
        let mut headers = ResponseHeaders::new();
        headers.security_headers();

        assert_eq!(headers.get("x-content-type-options"), Some("nosniff"));
        assert_eq!(headers.get("x-xss-protection"), Some("1; mode=block"));
        assert_eq!(headers.get("x-frame-options"), Some("DENY"));
        assert!(!headers.contains_key("x-powered-by"));
    }

    #[test]
    fn test_builder_pattern() {
        let headers = ResponseHeaders::new()
            .with_content_type("text/html")
            .with_header("X-Custom", "value")
            .with_cors(Some("*"))
            .with_security();

        assert_eq!(headers.get("content-type"), Some("text/html"));
        assert_eq!(headers.get("x-custom"), Some("value"));
        assert!(headers.contains_key("access-control-allow-origin"));
        assert!(headers.contains_key("x-content-type-options"));
    }

    #[test]
    fn test_dynamic_values() {
        let mut headers = ResponseHeaders::new();
        let dynamic_value = format!("session-{}", 12345);

        headers.insert("X-Session-ID", dynamic_value.clone());
        assert_eq!(headers.get("x-session-id"), Some(dynamic_value.as_str()));
    }

    #[test]
    fn test_multiple_values() {
        let mut headers = ResponseHeaders::new();
        headers.insert("Set-Cookie", "session=abc123; HttpOnly");
        headers.append("Set-Cookie", "theme=dark; Path=/");

        let all_cookies = headers.get_all("set-cookie").unwrap();
        assert_eq!(all_cookies.len(), 2);
        assert!(all_cookies.contains(&"session=abc123; HttpOnly".to_string()));
        assert!(all_cookies.contains(&"theme=dark; Path=/".to_string()));
    }

    #[test]
    fn test_canonical_reason_standard() {
        assert_eq!(StatusCode::Ok.canonical_reason(), "OK");
        assert_eq!(StatusCode::Created.canonical_reason(), "Created");
        assert_eq!(StatusCode::Accepted.canonical_reason(), "Accepted");
        assert_eq!(StatusCode::NoContent.canonical_reason(), "No Content");
        assert_eq!(StatusCode::Redirect.canonical_reason(), "Found");
        assert_eq!(
            StatusCode::PermanentRedirect.canonical_reason(),
            "Moved Permanently"
        );
        assert_eq!(StatusCode::BadRequest.canonical_reason(), "Bad Request");
        assert_eq!(StatusCode::Unauthorized.canonical_reason(), "Unauthorized");
        assert_eq!(StatusCode::Forbidden.canonical_reason(), "Forbidden");
        assert_eq!(StatusCode::NotFound.canonical_reason(), "Not Found");
        assert_eq!(
            StatusCode::MethodNotAllowed.canonical_reason(),
            "Method Not Allowed"
        );
        assert_eq!(StatusCode::Conflict.canonical_reason(), "Conflict");
        assert_eq!(
            StatusCode::InternalServerError.canonical_reason(),
            "Internal Server Error"
        );
        assert_eq!(
            StatusCode::NotImplemented.canonical_reason(),
            "Not Implemented"
        );
        assert_eq!(StatusCode::BadGateway.canonical_reason(), "Bad Gateway");
        assert_eq!(
            StatusCode::ServiceUnavailable.canonical_reason(),
            "Service Unavailable"
        );
        let custom = StatusCode::Custom(599);
        assert_eq!(custom.canonical_reason(), "Custom");
    }

    #[test]
    fn response_header_from() {
        let mut hash_map = HashMap::new();
        hash_map.insert("Content-Type", "application/json");
        let headers = ResponseHeaders::from(hash_map);
        assert_eq!(headers.get("content-type"), Some("application/json"));
    }

    #[test]
    fn response_header_index() {
        let mut hash_map = HashMap::new();
        hash_map.insert("Content-Type", "application/json");
        let headers = ResponseHeaders::from(hash_map);

        assert_eq!(&headers["content-type"], "application/json");
    }

    #[should_panic]
    #[test]
    fn response_header_index_should_panic() {
        let hash_map = HashMap::new();
        let headers = ResponseHeaders::from(hash_map);

        assert_eq!(&headers["content-type"], "application/json");
    }

    #[test]
    fn test_response_header_display() {
        let mut headers = ResponseHeaders::new();
        headers.insert("Content-Type", "application/json");

        assert_eq!(headers.to_string(), "content-type: application/json\n");
    }
    #[test]
    fn test_standard_status_codes_fmt() {
        let cases = vec![
            (StatusCode::Ok, "200 OK"),
            (StatusCode::Created, "201 Created"),
            (StatusCode::Accepted, "202 Accepted"),
            (StatusCode::NoContent, "204 No Content"),
            (StatusCode::Redirect, "302 Found"),
            (StatusCode::PermanentRedirect, "301 Moved Permanently"),
            (StatusCode::BadRequest, "400 Bad Request"),
            (StatusCode::Unauthorized, "401 Unauthorized"),
            (StatusCode::Forbidden, "403 Forbidden"),
            (StatusCode::NotFound, "404 Not Found"),
            (StatusCode::MethodNotAllowed, "405 Method Not Allowed"),
            (StatusCode::Conflict, "409 Conflict"),
            (StatusCode::InternalServerError, "500 Internal Server Error"),
            (StatusCode::NotImplemented, "501 Not Implemented"),
            (StatusCode::BadGateway, "502 Bad Gateway"),
            (StatusCode::ServiceUnavailable, "503 Service Unavailable"),
        ];

        for (status, expected) in cases {
            assert_eq!(format!("{}", status), expected);
        }
    }

    #[test]
    fn test_custom_status_code_fmt() {
        let custom = StatusCode::Custom(499);
        assert_eq!(format!("{}", custom), "499 Custom");

        let another_custom = StatusCode::Custom(600);
        assert_eq!(format!("{}", another_custom), "600 Custom");
    }
    #[test]
    fn test_standard_status_codes_from_u16() {
        let cases = vec![
            (200, StatusCode::Ok),
            (201, StatusCode::Created),
            (202, StatusCode::Accepted),
            (204, StatusCode::NoContent),
            (302, StatusCode::Redirect),
            (301, StatusCode::PermanentRedirect),
            (400, StatusCode::BadRequest),
            (401, StatusCode::Unauthorized),
            (403, StatusCode::Forbidden),
            (404, StatusCode::NotFound),
            (405, StatusCode::MethodNotAllowed),
            (409, StatusCode::Conflict),
            (500, StatusCode::InternalServerError),
            (501, StatusCode::NotImplemented),
            (502, StatusCode::BadGateway),
            (503, StatusCode::ServiceUnavailable),
        ];

        for (code, expected) in cases {
            assert_eq!(StatusCode::from_u16(code), expected, "failed for {}", code);
        }
    }

    #[test]
    fn test_custom_status_codes_from_u16() {
        let custom_codes = [199, 299, 450, 600, 999];

        for &code in &custom_codes {
            match StatusCode::from_u16(code) {
                StatusCode::Custom(inner) => assert_eq!(inner, code),
                other => panic!("expected Custom({}), got {:?}", code, other),
            }
        }
    }
    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let resp_err: ResponseError = io_err.into();

        match resp_err {
            ResponseError::IoError(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
                assert_eq!(e.to_string(), "file not found");
            }
            _ => panic!("Expected ResponseError::IoError"),
        }
    }

    #[test]
    fn test_display_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "no permission");
        let resp_err: ResponseError = io_err.into();

        let output = format!("{}", resp_err);
        assert_eq!(output, "IO error: no permission");
    }

    #[test]
    fn test_display_other_error() {
        let resp_err = ResponseError::_Other("something went wrong");

        let output = format!("{}", resp_err);
        assert_eq!(output, "Error: something went wrong");
    }

    #[test]
    fn test_error_trait_description() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "low-level failure");
        let resp_err: ResponseError = io_err.into();

        // std::error::Error gives us a source() method
        assert!(resp_err.source().is_none(), "Expected no source error");
    }

    fn sample_response() -> HttpResponse {
        let mut headers = ResponseHeaders::new();
        headers.insert("Content-Type", "application/json");

        let cookies = Cookie {
            name: "session",
            value: "abcd1234",
            options: CookieOptions::default(),
        };

        HttpResponse {
            status_code: StatusCode::Ok,
            body: crate::types::ResponseContentBody::new_binary(bytes::Bytes::from_static(
                b"hello world",
            )),
            content_type: crate::types::ResponseContentType::BINARY,
            cookies: vec![cookies],
            headers,
            remove_cookies: vec!["old_cookie".into()],
            is_stream: false,
            stream: Box::pin(stream::empty::<Result<bytes::Bytes, ResponseError>>()),
        }
    }

    #[test]
    fn test_debug_formatting() {
        let resp = sample_response();
        let debug_str = format!("{:?}", resp);

        println!("{:?}", debug_str);

        // Stream should be displayed as "<stream>"
        assert!(debug_str.contains("HttpResponse"));
        assert!(debug_str.contains("status_code: Ok"));
        assert!(debug_str.contains("body: BINARY(b\"hello world\")"));
        assert!(debug_str.contains("content_type: BINARY"));
        assert!(debug_str.contains("cookies"));
        assert!(debug_str.contains("headers"));
        assert!(debug_str.contains("remove_cookies"));
        assert!(debug_str.contains("is_stream: false"));
        assert!(debug_str.contains("stream: \"<stream>\""));
    }

    #[test]
    fn test_clone_response() {
        let resp = sample_response();
        let cloned = resp.clone();

        assert_eq!(resp.status_code, cloned.status_code);
        assert_eq!(resp.body, cloned.body);
        assert_eq!(resp.content_type, cloned.content_type);
        assert_eq!(resp.cookies, cloned.cookies);
        assert_eq!(resp.headers, cloned.headers);
        assert_eq!(resp.remove_cookies, cloned.remove_cookies);
        assert_eq!(resp.is_stream, cloned.is_stream);

        // Ensure cloned stream is not the same allocation as original
        // (both are empty, but new clone should have a fresh Box::pin(stream::empty()))
        let orig_debug = format!("{:?}", resp);
        let cloned_debug = format!("{:?}", cloned);
        assert_eq!(orig_debug, cloned_debug);
    }

    #[test]
    fn test_redirect_sets_status_and_location() {
        let res = sample_response().redirect("/home");

        assert_eq!(res.status_code, StatusCode::Redirect);
        assert_eq!(res.headers.get("Location"), Some("/home"));
    }

    #[test]
    fn test_permanent_redirect_sets_status_and_location() {
        let res = sample_response().permanent_redirect("https://example.com");

        assert_eq!(res.status_code, StatusCode::PermanentRedirect);
        assert_eq!(res.headers.get("Location"), Some("https://example.com"));
    }
}
