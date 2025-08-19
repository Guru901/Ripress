#[cfg(test)]
mod tests {
    use crate::req::body::{FormData, TextData, TextDataError};

    #[test]
    fn test_new_from_string() {
        let text = TextData::new("Hello, world!".to_string());
        assert_eq!(text.as_str().unwrap(), "Hello, world!");
        assert_eq!(text.len_bytes(), 13);
        assert_eq!(text.charset(), Some("utf-8"));
    }

    #[test]
    fn test_from_bytes() {
        let bytes = "Hello, 世界!".as_bytes().to_vec();
        let text = TextData::from_bytes(bytes).unwrap();

        assert_eq!(text.as_str().unwrap(), "Hello, 世界!");
        assert_eq!(text.len_chars().unwrap(), 10);
    }

    #[test]
    fn test_invalid_utf8() {
        let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
        let result = TextData::from_bytes(invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_size_limit() {
        let large_text = "x".repeat(1000);
        let bytes = large_text.as_bytes().to_vec();
        let result = TextData::from_bytes_with_limit(bytes, 500);
        assert!(matches!(result, Err(TextDataError::TooLarge { .. })));
    }

    #[test]
    fn test_truncation() {
        let text = TextData::new("Hello, 世界!".to_string());
        let truncated = text.truncated_bytes(8);
        // Should truncate at valid UTF-8 boundary
        assert!(truncated.as_str().is_ok());
    }

    #[test]
    fn test_display() {
        let text = TextData::new("Test display".to_string());
        assert_eq!(format!("{}", text), "Test display");
    }

    #[test]
    fn test_append() {
        let mut form = FormData::new();
        form.append("tags", "rust");
        form.append("tags", "web");
        assert_eq!(form.get("tags"), Some("rust,web"));
    }

    #[test]
    fn test_query_string() {
        let mut form = FormData::new();
        form.insert("name", "John Doe");
        form.insert("age", "30");

        let query = form.to_query_string();
        let parsed = FormData::from_query_string(&query).unwrap();

        assert_eq!(parsed.get("name"), Some("John Doe"));
        assert_eq!(parsed.get("age"), Some("30"));
    }

    #[test]
    fn test_raw_form_data_preservation() {
        let mut form = FormData::new();
        form.insert("invalid", "%%form%data");

        // Raw data should be preserved in get()
        assert_eq!(form.get("invalid"), Some("%%form%data"));

        // But should be URL-encoded when converted to query string
        let query = form.to_query_string();
        assert!(query.contains("invalid=%25%25form%25data"));

        // And should decode back correctly
        let parsed = FormData::from_query_string(&query).unwrap();
        assert_eq!(parsed.get("invalid"), Some("%%form%data"));
    }

    #[test]
    fn test_url_encoding_edge_cases() {
        let mut form = FormData::new();
        form.insert("special", "hello world+&=");

        let query = form.to_query_string();
        let parsed = FormData::from_query_string(&query).unwrap();

        assert_eq!(parsed.get("special"), Some("hello world+&="));
    }
    #[test]
    fn test_basic_form_operations() {
        let mut form = FormData::new();
        assert!(form.is_empty());

        form.insert("key", "value");
        assert_eq!(form.get("key"), Some("value"));
        assert_eq!(form.len(), 1);
        assert!(!form.is_empty());

        assert_eq!(form.remove("key"), Some("value".to_string()));
        assert!(form.is_empty());
    }
}
