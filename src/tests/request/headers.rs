#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::req::request_headers::RequestHeaders;

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
}
