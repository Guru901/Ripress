#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::req::route_params::{ParamError, RouteParams};

    #[test]
    fn test_basic_params_operations() {
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
    }
}
