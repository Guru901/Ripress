#[cfg(test)]
mod extractor_tests {
    use crate::helpers::{ExtractFromOwned, FromRequest};
    use crate::req::body::json_data::JsonBody;
    use crate::req::body::{RequestBody, RequestBodyContent, RequestBodyType};
    use crate::req::origin_url::Url;
    use crate::req::query_params::{FromQueryParam, QueryParam};
    use crate::req::request_headers::Headers;
    use crate::req::route_params::Params;
    use crate::req::HttpRequest;
    use hyper::header::{HeaderName, HeaderValue};
    use serde_json::json;

    // Test structures
    #[derive(Debug, PartialEq)]
    struct UserId {
        id: u32,
    }

    impl crate::req::route_params::FromParams for UserId {
        fn from_params(params: &crate::req::route_params::RouteParams) -> Result<Self, String> {
            let id = params.get("id").ok_or("Missing id parameter")?;
            let id: u32 = id.parse().map_err(|_| "Invalid id format")?;
            Ok(UserId { id })
        }
    }

    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct UserData {
        name: String,
        age: u32,
    }

    impl crate::req::body::json_data::FromJson for UserData {
        fn from_json(data: &crate::req::body::RequestBodyContent) -> Result<Self, String> {
            if let RequestBodyContent::JSON(json_val) = data {
                serde_json::from_value(json_val.clone()).map_err(|e| e.to_string())
            } else {
                Err("Expected JSON body".to_string())
            }
        }
    }

    // Helper functions
    fn create_request_with_json(json_value: serde_json::Value) -> HttpRequest {
        let mut req = HttpRequest::default();
        req.body = RequestBody {
            content: RequestBodyContent::JSON(json_value),
            content_type: RequestBodyType::JSON,
        };
        req
    }

    fn create_request_with_params(params: Vec<(&str, &str)>) -> HttpRequest {
        let mut req = HttpRequest::default();
        for (key, value) in params {
            req.set_param(key, value);
        }
        req
    }

    fn create_request_with_query(query: &str) -> HttpRequest {
        let mut req = HttpRequest::default();
        req.origin_url = Url::new(format!("http://localhost/test?{}", query).as_str());
        req
    }

    fn create_request_with_headers(headers: Vec<(&str, &str)>) -> HttpRequest {
        let mut req = HttpRequest::default();
        for (key, value) in headers {
            req.headers
                .insert(HeaderName::from_bytes(key.as_bytes()).unwrap(), value);
        }
        req
    }

    // JsonBody extractor tests
    #[test]
    fn test_json_body_extractor_success() {
        let json = json!({
            "name": "Alice",
            "age": 30
        });
        let req = create_request_with_json(json);

        let result = JsonBody::<UserData>::from_request(&req);
        assert!(result.is_ok());

        let body = result.unwrap();
        assert_eq!(body.name, "Alice");
        assert_eq!(body.age, 30);
    }

    #[test]
    fn test_json_body_extractor_invalid_json() {
        let json = json!({
            "name": "Bob"
            // age is missing
        });
        let req = create_request_with_json(json);

        let result = JsonBody::<UserData>::from_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_body_deref() {
        let json = json!({
            "name": "Charlie",
            "age": 25
        });
        let req = create_request_with_json(json);

        let body = JsonBody::<UserData>::from_request(&req).unwrap();

        // Test Deref access
        assert_eq!(body.name, "Charlie");
        assert_eq!(body.age, 25);
    }

    // Params extractor tests
    #[test]
    fn test_params_extractor_success() {
        let req = create_request_with_params(vec![("id", "42")]);

        let result = Params::<UserId>::from_request(&req);
        assert!(result.is_ok());

        let params = result.unwrap();
        assert_eq!(params.id, 42);
    }

    #[test]
    fn test_params_extractor_missing_param() {
        let req = HttpRequest::default();

        let result = Params::<UserId>::from_request(&req);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.contains("Missing id parameter"));
    }

    #[test]
    fn test_params_extractor_invalid_format() {
        let req = create_request_with_params(vec![("id", "not_a_number")]);

        let result = Params::<UserId>::from_request(&req);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.contains("Invalid id format"));
    }

    // QueryParam extractor tests
    #[test]
    fn test_query_param_string_success() {
        let req = create_request_with_query("search=rust");

        let result = QueryParam::<String>::from_request(&req);
        assert!(result.is_ok());
    }

    impl FromQueryParam for String {
        fn from_query_param(
            params: &crate::req::query_params::QueryParams,
        ) -> Result<Self, String> {
            Ok(String::new())
        }
    }

    #[test]
    fn test_query_param_number_success() {
        let req = create_request_with_query("page=5");

        let result = QueryParam::<i32>::from_request(&req);
        assert!(result.is_ok());
    }

    impl FromQueryParam for i32 {
        fn from_query_param(
            params: &crate::req::query_params::QueryParams,
        ) -> Result<Self, String> {
            Ok(0)
        }
    }

    // Headers extractor tests
    #[test]
    fn test_headers_extractor_success() {
        let req = create_request_with_headers(vec![
            ("content-type", "application/json"),
            ("authorization", "Bearer token123"),
        ]);

        let result = Headers::from_request(&req);
        assert!(result.is_ok());

        let headers = result.unwrap();
        assert!(headers.get("content-type").is_some());
        assert!(headers.get("authorization").is_some());
    }

    #[test]
    fn test_headers_extractor_empty() {
        let req = HttpRequest::default();

        let result = Headers::from_request(&req);
        assert!(result.is_ok());
    }

    // Tuple extractor tests (ExtractFromOwned)
    #[test]
    fn test_tuple_two_extractors() {
        let mut req = create_request_with_json(json!({
            "name": "David",
            "age": 35
        }));
        req.set_param("id", "123");

        let result = <(JsonBody<UserData>, Params<UserId>)>::extract_from_owned(req);
        assert!(result.is_ok());

        let (body, params) = result.unwrap();
        assert_eq!(body.name, "David");
        assert_eq!(params.id, 123);
    }

    #[test]
    fn test_tuple_three_extractors() {
        let mut req = create_request_with_json(json!({
            "name": "Eve",
            "age": 28
        }));
        req.set_param("id", "456");
        req.origin_url = Url::new("/users?sort=desc");

        let result =
            <(JsonBody<UserData>, Params<UserId>, QueryParam<String>)>::extract_from_owned(req);

        assert!(result.is_ok());
        let (body, params, _query) = result.unwrap();
        assert_eq!(body.name, "Eve");
        assert_eq!(params.id, 456);
    }

    #[test]
    fn test_tuple_extractor_first_fails() {
        let req = create_request_with_params(vec![("id", "789")]);
        // JSON body is missing

        let result = <(JsonBody<UserData>, Params<UserId>)>::extract_from_owned(req);
        assert!(result.is_err());
    }

    #[test]
    fn test_tuple_extractor_second_fails() {
        let req = create_request_with_json(json!({
            "name": "Frank",
            "age": 40
        }));
        // id param is missing

        let result = <(JsonBody<UserData>, Params<UserId>)>::extract_from_owned(req);
        assert!(result.is_err());
    }

    #[test]
    fn test_single_extractor_tuple() {
        let req = create_request_with_params(vec![("id", "999")]);

        let result = <(Params<UserId>,)>::extract_from_owned(req);
        assert!(result.is_ok());

        let (params,) = result.unwrap();
        assert_eq!(params.id, 999);
    }

    #[test]
    fn test_http_request_extractor() {
        let req = HttpRequest::default();

        let result = HttpRequest::extract_from_owned(req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tuple_with_headers_and_json() {
        let mut req = create_request_with_json(json!({
            "name": "Grace",
            "age": 32
        }));
        req.headers
            .insert(HeaderName::from_static("user-agent"), "test-client");

        let result = <(JsonBody<UserData>, Headers)>::extract_from_owned(req);
        assert!(result.is_ok());

        let (body, headers) = result.unwrap();
        assert_eq!(body.name, "Grace");
        assert!(headers.get("user-agent").is_some());
    }

    #[test]
    fn test_complex_four_extractors() {
        let mut req = create_request_with_json(json!({
            "name": "Henry",
            "age": 45
        }));
        req.set_param("id", "111");
        req.origin_url = Url::new("/api/users?filter=active");
        req.headers
            .insert(HeaderName::from_static("authorization"), "Bearer abc123");

        let result = <(
            JsonBody<UserData>,
            Params<UserId>,
            QueryParam<String>,
            Headers,
        )>::extract_from_owned(req);

        assert!(result.is_ok());
        let (body, params, _query, headers) = result.unwrap();
        assert_eq!(body.name, "Henry");
        assert_eq!(params.id, 111);
        assert!(headers.get("authorization").is_some());
    }
}
