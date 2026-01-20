#[cfg(test)]
mod tests {
    use hyper::{HeaderMap, header::HeaderValue};

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
        assert_eq!(headers.get("accept"), Some("text/html")); 
    }

    #[test]
    fn test_headers_convenience_methods() {
        let mut headers = RequestHeaders::new();
        headers.insert("Content-Type", "application/json");
        headers.insert("Accept", "application/json");
        headers.insert("host", "example.com");
        headers.insert("x-forwarded-for", "127.0.0.1");
        headers.insert("Authorization", "Bearer 123");
        headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36");

        assert_eq!(headers.content_type(), Some("application/json"));
        assert_eq!(headers.host(), Some("example.com"));
        assert!(headers.accepts_json());
        assert!(!headers.accepts_html());
        assert_eq!(headers.x_forwarded_for(), Some("127.0.0.1"));
        assert_eq!(headers.authorization(), Some("Bearer 123"));
        assert_eq!(
            headers.user_agent(),
            Some(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
            )
        );
    }

    #[test]
    fn test_headers_default() {
        let headers = RequestHeaders::default();
        assert_eq!(headers.is_empty(), true);
        assert_eq!(headers.len(), 0);
        assert_eq!(headers.iter().count(), 0);
        assert_eq!(headers.iter_all().count(), 0);
        assert_eq!(headers.as_header_map().is_empty(), true);
        assert_eq!(headers.keys().count(), 0);
        assert_eq!(headers.into_header_map().is_empty(), true);
    }

    #[test]
    fn test_headers_from_and_into_header_map() {
        let mut map = HeaderMap::new();
        map.insert("id", HeaderValue::from_static("123"));
        map.insert("name", HeaderValue::from_static("test"));

        let headers = RequestHeaders::from(map.clone());
        assert!(headers.contains_key("id"));
        assert!(headers.contains_key("name"));

        let map2 = HeaderMap::from(headers);
        assert_eq!(map2.get("id").and_then(|v| v.to_str().ok()), Some("123"));
        assert_eq!(map2.get("name").and_then(|v| v.to_str().ok()), Some("test"));
    }

    #[test]
    fn test_headers_index_and_fmt() {
        let mut headers = RequestHeaders::new();
        headers.insert("Content-Type", "application/json");

        assert_eq!(headers.to_string(), "content-type: \"application/json\"\n");
        assert_eq!(&headers["content-type"], "application/json");
    }
}
