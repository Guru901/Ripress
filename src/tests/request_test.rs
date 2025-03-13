#[cfg(test)]
mod tests {
    use crate::{
        request::HttpRequest,
        types::{HttpMethods, RequestBodyType},
    };

    #[test]
    fn test_get_query() {
        let mut req = HttpRequest::new();
        req.set_query("q", "Ripress");

        assert_eq!(req.get_query("q"), Some("Ripress".to_string()));

        assert_eq!(req.get_query("nonexistent"), None);
    }

    #[test]
    fn test_get_param() {
        let mut req = HttpRequest::new();
        req.set_param("q", "Ripress");

        assert_eq!(req.get_params("q"), Some("Ripress".to_string()));

        assert_eq!(req.get_params("nonexistent"), None);
    }

    #[test]
    fn test_json_body() {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct User {
            id: u32,
            name: String,
        }

        let mut req = HttpRequest::new();
        req.set_json(User {
            id: 1,
            name: "John Doe".to_string(),
        });

        assert_eq!(
            req.json::<User>().unwrap(),
            User {
                id: 1,
                name: "John Doe".to_string()
            }
        );

        assert!(req.json::<String>().is_err());
    }

    #[test]
    fn test_text_body() {
        let mut req = HttpRequest::new();
        req.set_text("Ripress");

        assert_eq!(req.text(), Ok("Ripress".to_string()));

        req.set_text("");
        assert_eq!(req.text(), Ok("".to_string()));
    }

    #[test]
    fn test_form_data() {
        let mut req = HttpRequest::new();
        req.set_form("key", "value");

        assert_eq!(req.form_data().unwrap().get("key").unwrap(), "value");
        assert_eq!(req.form_data().unwrap().get("nonexistent"), None);

        req.set_form("another_key", "another_value");
        let form_data = req.form_data().unwrap();
        dbg!(&form_data);
        assert_eq!(form_data.get("key").unwrap(), "value");
        assert_eq!(form_data.get("another_key").unwrap(), "another_value");
    }

    #[test]
    fn test_header() {
        let mut req = HttpRequest::new();
        req.set_header("key", "value");

        assert_eq!(req.get_header("key").unwrap(), "value");
        assert_eq!(req.get_header("non-existent"), None);

        req.set_header("another_key", "another_value");
        let header = req.get_header("another_key").unwrap();
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
        assert_eq!(req.get_method(), &HttpMethods::GET);

        req.set_method(HttpMethods::POST);
        assert_eq!(req.get_method(), &HttpMethods::POST);

        req.set_method(HttpMethods::PUT);
        assert_eq!(req.get_method(), &HttpMethods::PUT);

        req.set_method(HttpMethods::DELETE);
        assert_eq!(req.get_method(), &HttpMethods::DELETE);

        req.set_method(HttpMethods::DELETE);
        assert_ne!(req.get_method(), &HttpMethods::GET);
    }

    #[test]
    fn test_ip_method() {
        let mut req = HttpRequest::new();

        req.set_ip("127.0.0.1".to_string());
        assert_eq!(req.ip().unwrap(), "127.0.0.1");

        req.set_ip("127.0.0.2".to_string());
        assert_eq!(req.ip().unwrap(), "127.0.0.2");
    }

    #[test]
    fn test_get_path() {
        let mut req = HttpRequest::new();
        req.set_path("/user/1".to_string());

        assert_eq!(req.get_path().unwrap(), "/user/1");
    }

    #[test]
    fn test_get_origin_url() {
        let mut req = HttpRequest::new();
        req.set_origin_url("/user/1".to_string());
        assert_eq!(req.get_origin_url().unwrap(), "/user/1");

        req.set_origin_url("/user/1?q=hello".to_string());
        assert_eq!(req.get_origin_url().unwrap(), "/user/1?q=hello");
    }
}
