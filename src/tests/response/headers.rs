#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{req::HttpRequest, res::response_headers::ResponseHeaders};

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

        let all_cookies = headers.get_all("set-cookie");
        assert_eq!(all_cookies.len(), 2);
        assert!(all_cookies.contains(&"session=abc123; HttpOnly"));
        assert!(all_cookies.contains(&"theme=dark; Path=/"));
    }
}
