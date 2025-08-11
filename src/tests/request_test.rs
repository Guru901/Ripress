#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::context::HttpResponse;
    use crate::req::query_params::{QueryParams, SortDirection};
    use crate::req::request_headers::RequestHeaders;
    use crate::req::route_params::{ParamError, RouteParams};
    use crate::res::CookieOptions;
    use crate::types::{HttpResponseError, ResponseContentBody, ResponseContentType};
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
            .to_responder()
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
            .to_responder()
            .unwrap();

        assert_eq!(response.status(), 200);
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
            .set_cookie("another_cookie", "123", CookieOptions::default())
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
        let hyper_response = response.to_responder().unwrap();
        assert_eq!(hyper_response.status(), 200);

        let response = HttpResponse::new().internal_server_error().text("Invalid");
        let hyper_response = response.to_responder().unwrap();

        assert_eq!(hyper_response.status(), 500);
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
    fn test_case_insensitive() {
        let mut headers = RequestHeaders::new();
        headers.insert("Content-Type", "application/json");

        assert_eq!(headers.get("content-type"), Some("application/json"));
        assert_eq!(headers.get("CONTENT-TYPE"), Some("application/json"));
        assert_eq!(headers.get("Content-Type"), Some("application/json"));
    }

    #[test]
    fn test_multiple_values() {
        let mut headers = RequestHeaders::new();
        headers.insert("Accept", "text/html");
        headers.append("Accept", "application/json");

        let all_values = headers.get_all("accept").unwrap();
        assert_eq!(all_values.len(), 2);
        assert_eq!(headers.get("accept"), Some("text/html")); // First value
    }

    #[test]
    fn test_headers_convenience_methods() {
        let mut headers = RequestHeaders::new();
        headers.insert("Content-Type", "application/json");
        headers.insert("Accept", "application/json");

        assert_eq!(headers.content_type(), Some("application/json"));
        assert!(headers.accepts_json());
        assert!(!headers.accepts_html());
    }

    #[test]
    fn test_basic_operations() {
        let mut params = RouteParams::new();
        params.insert("id", "123");
        params.insert("slug", "hello-world");

        assert_eq!(params.get("id"), Some("123"));
        assert_eq!(params.get("slug"), Some("hello-world"));
        assert_eq!(params.get("missing"), None);
    }

    #[test]
    fn test_type_parsing() {
        let mut params = RouteParams::new();
        params.insert("id", "123");
        params.insert("user_id", "456");
        params.insert("invalid", "not-a-number");

        assert_eq!(params.get_int("id").unwrap(), 123);
        assert!(params.get_int("invalid").is_err());
        assert!(params.get_int("missing").is_err());
    }

    #[test]
    fn test_param_convenience_methods() {
        let mut params = RouteParams::new();
        params.insert("id", "42");
        params.insert("slug", "test-post");
        params.insert("user_id", "100");

        assert_eq!(params.id().unwrap(), 42);
        assert_eq!(params.slug(), Some("test-post"));
    }

    #[test]
    fn test_error_types() {
        let params = RouteParams::new();

        match params.get_int("missing") {
            Err(ParamError::NotFound(name)) => assert_eq!(name, "missing"),
            _ => panic!("Expected NotFound error"),
        }

        let mut params = RouteParams::new();
        params.insert("invalid", "not-a-number");

        match params.get_int("invalid") {
            Err(ParamError::ParseError { param, value, .. }) => {
                assert_eq!(param, "invalid");
                assert_eq!(value, "not-a-number");
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_defaults() {
        let mut params = RouteParams::new();
        params.insert("valid", "10");

        assert_eq!(params.get_or_default("valid", 5), 10);
        assert_eq!(params.get_or_default("missing", 5), 5);
        assert_eq!(params.get_or_default("invalid", 5), 5);
    }

    #[test]
    fn test_from_map() {
        let mut map = HashMap::new();
        map.insert("id".to_string(), "123".to_string());
        map.insert("name".to_string(), "test".to_string());

        let params = RouteParams::from_map(map);
        assert_eq!(params.get("id"), Some("123"));
        assert_eq!(params.get("name"), Some("test"));
    }

    // Example of using the extract_params macro
    #[test]
    fn test_extract_macro() {
        let mut params = RouteParams::new();
        params.insert("id", "123");
        params.insert("user_id", "456");

        // This would be used in a real handler like:
        // let (id, user_id) = extract_params!(params, { id: i32, user_id: i32 })?;
    }

    #[test]
    fn test_from_query_string() {
        let query =
            QueryParams::from_query_string("page=2&limit=10&tags=rust&tags=web&active=true");

        assert_eq!(query.get("page"), Some("2"));
        assert_eq!(query.get("limit"), Some("10"));
        assert_eq!(query.get("active"), Some("true"));

        let tags = query.get_all("tags").unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"web".to_string()));
    }

    #[test]
    fn test_query_type_parsing() {
        let query = QueryParams::from_query_string("page=2&limit=10&active=true&price=19.99");

        assert_eq!(query.get_int("page").unwrap(), 2);
        assert_eq!(query.get_uint("limit").unwrap(), 10);
        assert_eq!(query.get_bool("active").unwrap(), true);
        assert_eq!(query.get_float("price").unwrap(), 19.99);
    }

    #[test]
    fn test_query_multiple_values() {
        let query = QueryParams::from_query_string("tags=rust&tags=web&tags=backend");

        let tags = query.get_all_parsed::<String>("tags").unwrap();
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"web".to_string()));
        assert!(tags.contains(&"backend".to_string()));
    }

    #[test]
    fn test_convenience_methods() {
        let query =
            QueryParams::from_query_string("page=3&limit=25&q=search+term&sort=name&order=desc");

        assert_eq!(query.page(), 3);
        assert_eq!(query.limit(), 25);
        assert_eq!(query.search_query(), Some("search+term"));
        assert_eq!(query.sort(), Some("name"));
        assert_eq!(query.sort_direction(), SortDirection::Desc);
    }

    #[test]
    fn test_boolean_parsing() {
        let query = QueryParams::from_query_string(
            "active=true&debug=1&verbose=yes&feature&disabled=false",
        );

        assert_eq!(query.get_bool("active").unwrap(), true);
        assert_eq!(query.get_bool("debug").unwrap(), true);
        assert_eq!(query.get_bool("verbose").unwrap(), true);
        assert_eq!(query.get_bool("disabled").unwrap(), false);
        assert!(query.is_truthy("feature")); // Parameter exists without value
    }

    #[test]
    fn test_filters() {
        let query = QueryParams::from_query_string(
            "filter[status]=active&filter[type]=user&filter[role]=admin",
        );

        let filters = query.filters();
        assert_eq!(filters.get("status").unwrap()[0], "active");
        assert_eq!(filters.get("type").unwrap()[0], "user");
        assert_eq!(filters.get("role").unwrap()[0], "admin");
    }

    #[test]
    fn test_query_defaults() {
        let query = QueryParams::from_query_string("existing=42");

        assert_eq!(query.get_or_default("existing", 0), 42);
        assert_eq!(query.get_or_default("missing", 100), 100);
        assert_eq!(query.page(), 1); // Default page
        assert_eq!(query.limit(), 20); // Default limit
    }

    #[test]
    fn test_headers_from_map() {
        let mut map = HashMap::new();
        map.insert("id".to_string(), "123".to_string());
        map.insert("name".to_string(), "test".to_string());

        let headers = RequestHeaders::_from_map(map);
        assert_eq!(headers.get("id"), Some("123"));
        assert_eq!(headers.get("name"), Some("test"));
    }

    #[test]
    fn test_headers_remove() {
        let mut map = HashMap::new();
        map.insert("id".to_string(), "123".to_string());
        map.insert("name".to_string(), "test".to_string());

        let mut headers = RequestHeaders::_from_map(map);

        headers.remove("id");

        assert_eq!(headers.get("id"), None);
    }

    #[test]
    fn test_headers_contains_key() {
        let mut map = HashMap::new();
        map.insert("id".to_string(), "123".to_string());
        map.insert("name".to_string(), "test".to_string());

        let headers = RequestHeaders::_from_map(map);

        assert_eq!(headers.contains_key("id"), true);
        assert_eq!(headers.contains_key("name"), true);
        assert_eq!(headers.contains_key("non-existent"), false);
    }
}
