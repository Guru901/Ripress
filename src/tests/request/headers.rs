#[cfg(test)]
mod tests {
    use hyper::HeaderMap;

    use crate::req::request_headers::RequestHeaders;

    #[test]
    fn test_headers_from_map() {
        let mut map = HeaderMap::new();
        map.insert("id", "123".parse().unwrap());
        map.insert("name", "test".parse().unwrap());

        let headers = RequestHeaders::from_header_map(map);
        assert_eq!(headers.get("id"), Some("123"));
        assert_eq!(headers.get("name"), Some("test"));
    }

    #[test]
    fn test_headers_remove() {
        let mut map = HeaderMap::new();
        map.insert("id", "123".parse().unwrap());
        map.insert("name", "test".parse().unwrap());

        let mut headers = RequestHeaders::from_header_map(map);

        headers.remove("id");

        assert_eq!(headers.get("id"), None);
    }

    #[test]
    fn test_headers_contains_key() {
        let mut map = HeaderMap::new();
        map.insert("id", "123".parse().unwrap());
        map.insert("name", "test".parse().unwrap());

        let headers = RequestHeaders::from_header_map(map);

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

        let all_values = headers.get_all("accept");
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
