#[cfg(test)]
mod tests {
    use crate::context::HttpRequest;
    use crate::types::HttpRequestError;
    use crate::types::{HttpMethods, RequestBodyType};
    use actix_web::FromRequest;
    use serde_json::json;

    fn determine_content_type(content_type: &str) -> RequestBodyType {
        if content_type == "application/json" {
            return RequestBodyType::JSON;
        } else if content_type == "application/x-www-form-urlencoded" {
            return RequestBodyType::FORM;
        } else {
            RequestBodyType::TEXT
        }
    }

    #[test]
    fn test_get_query() {
        let mut req = HttpRequest::new();
        req.set_query("q", "Ripress");

        assert_eq!(req.get_query("q"), Ok("Ripress"));

        assert_eq!(
            req.get_query("nonexistent"),
            Err(HttpRequestError::MissingQuery("nonexistent".to_string()))
        );
    }

    #[test]
    fn test_get_param() {
        let mut req = HttpRequest::new();
        req.set_param("q", "Ripress");

        assert_eq!(req.get_params("q"), Ok("Ripress"));

        assert_eq!(
            req.get_params("nonexistent"),
            Err(HttpRequestError::MissingParam("nonexistent".to_string()))
        );
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

        req.set_text("{invalid json}", RequestBodyType::JSON);

        assert!(req.json::<User>().is_err());
    }

    #[test]
    fn test_text_body() {
        // Test 1 - Everything Is Correct

        let mut req = HttpRequest::new();

        req.set_text("Ripress", RequestBodyType::TEXT);

        assert_eq!(req.text(), Ok("Ripress".to_string()));

        // Test 2 - Invalid Body Type

        req.set_text("", RequestBodyType::JSON);

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

        // Test 3 - Invalid Form Content

        req.set_json(json!({"key": "value"}), RequestBodyType::FORM);
        assert!(req.form_data().is_err());

        req.set_form("invalid", "%%form%data", RequestBodyType::FORM);
        assert_ne!(
            req.form_data().unwrap().get("invalid").unwrap(),
            "%%form%data"
        );
    }

    #[test]
    fn test_header() {
        let mut req = HttpRequest::new();
        req.set_header("key", "value");

        assert_eq!(req.get_header("key").unwrap(), "value");
        assert_eq!(
            req.get_header("nonexistent"),
            Err(HttpRequestError::MissingHeader("nonexistent".to_string()))
        );

        req.set_header("another_key", "another_value");
        let header = req.get_header("another_key").unwrap();
        assert_eq!(header, "another_value");
    }

    #[test]
    fn text_cookie() {
        let mut req = HttpRequest::new();
        req.set_cookie("key", "value");

        assert_eq!(req.get_cookie("key").unwrap(), "value");
        assert_eq!(
            req.get_cookie("nonexistent"),
            Err(HttpRequestError::MissingCookie("nonexistent".to_string()))
        );

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
    fn test_ip_method() {
        let mut req = HttpRequest::new();

        req.set_ip("127.0.0.1".to_string());
        assert_eq!(req.ip, "127.0.0.1");

        req.set_ip("127.0.0.2".to_string());
        assert_eq!(req.ip, "127.0.0.2");
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
        req.set_origin_url("/user/1".to_string());
        assert_eq!(req.origin_url, "/user/1");

        req.set_origin_url("/user/1?q=hello".to_string());
        assert_eq!(req.origin_url, "/user/1?q=hello");
    }

    #[test]
    fn test_content_type() {
        let content_type = determine_content_type("application/json");
        assert_eq!(content_type, RequestBodyType::JSON);

        let content_type = determine_content_type("");
        assert_eq!(content_type, RequestBodyType::TEXT);

        let content_type = determine_content_type("application/x-www-form-urlencoded");
        assert_eq!(content_type, RequestBodyType::FORM);
    }

    #[test]
    fn test_error_enum() {
        let err_1 = HttpRequestError::MissingParam("id".to_string());
        let err_2 = HttpRequestError::MissingQuery("id".to_string());
        let err_3 = HttpRequestError::MissingCookie("id".to_string());
        let err_4 = HttpRequestError::MissingHeader("id".to_string());

        assert_eq!(err_1.to_string(), "Param id doesn't exist");
        assert_eq!(err_2.to_string(), "Query id doesn't exist");
        assert_eq!(err_3.to_string(), "Cookie id doesn't exist");
        assert_eq!(err_4.to_string(), "Header id doesn't exist");
    }

    #[test]
    fn test_set_and_get_data() {
        let mut req = HttpRequest::new();
        req.set_data("id", "123");
        assert_eq!(req.get_data("id"), Some(&"123"));
        assert_eq!(req.get_data("nonexistent"), None);
    }

    #[tokio::test]
    async fn test_from_actix_request() {
        let request = actix_web::test::TestRequest::default().to_http_request();
        let mut payload = actix_web::dev::Payload::None;
        let web_payload = actix_web::web::Payload::from_request(&request, &mut payload)
            .await
            .unwrap();

        let _ = HttpRequest::from_actix_request(request, web_payload)
            .await
            .unwrap();
    }
}
