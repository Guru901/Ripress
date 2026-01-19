mod cookies_test;
mod headers;
mod methods;
mod redirects_test;
mod status_code;
mod streaming_test;

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::context::HttpRequest;
    use crate::helpers::determine_content_type_request;
    use crate::req::body::RequestBodyType;
    use crate::req::body::TextData;
    use crate::req::origin_url::Url;
    use crate::res::response_cookie::{Cookie, CookieOptions};
    use crate::res::response_headers::ResponseHeaders;
    use crate::res::response_status::StatusCode;
    use crate::res::HttpResponse;
    use crate::res::ResponseError;
    use crate::types::HttpRequestError;
    use futures::stream;
    use serde_json::json;

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
    fn test_binary_body() {
        // Test 1 - Everything Is Correct

        let mut req = HttpRequest::new();

        req.set_binary(vec![1, 2, 3, 4, 5], RequestBodyType::BINARY);

        assert_eq!(req.bytes().unwrap(), vec![1, 2, 3, 4, 5]);

        // Test 2 - Invalid Body Type

        req.set_binary(vec![1, 2, 3, 4, 5], RequestBodyType::FORM);

        assert!(req.bytes().is_err());

        // Test 3 - Invalid Text Content

        req.set_binary(vec![1, 2, 3, 4, 5], RequestBodyType::TEXT);
        assert!(req.bytes().is_err());
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
        let content_type = determine_content_type_request("application/json");
        assert_eq!(content_type, RequestBodyType::JSON);

        let content_type = determine_content_type_request("");
        assert_eq!(content_type, RequestBodyType::BINARY);

        let content_type = determine_content_type_request("application/x-www-form-urlencoded");
        assert_eq!(content_type, RequestBodyType::FORM);

        let content_type = determine_content_type_request("application/octet-stream");
        assert_eq!(content_type, RequestBodyType::BINARY);

        let content_type = determine_content_type_request("image/png");
        assert_eq!(content_type, RequestBodyType::BINARY);

        let content_type = determine_content_type_request("application/xml");
        assert_eq!(content_type, RequestBodyType::TEXT);

        // Test multipart form detection
        let content_type = determine_content_type_request(
            "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW",
        );
        assert_eq!(content_type, RequestBodyType::MultipartForm);
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
            content_type: crate::types::ResponseBodyType::BINARY,
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
