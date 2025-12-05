#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        error::{RipressError, RipressErrorKind},
        req::route_params::{ParamError, RouteParams},
    };

    #[test]
    fn test_basic_params_operations() {
        let mut params = RouteParams::new();
        params.insert("id", "123");
        params.insert("slug", "hello-world");

        assert_eq!(params.get("id"), Some("123"));
        assert_eq!(params.get("slug"), Some("hello-world"));
        assert_eq!(params.get("missing"), None);
        assert!(params.contains("id"));
        assert!(!params.contains("missing"));
    }

    #[test]
    fn test_type_parsing() {
        let mut params = RouteParams::new();
        params.insert("id", "123");
        params.insert("user_id", "456");
        params.insert("invalid", "not-a-number");

        assert_eq!(params.get_int("id").unwrap(), 123);
        assert!(params.get_int("invalid").is_err());
        assert!(params.get_uint("missing").is_err());
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
            Err(e) => {
                assert_eq!(e.kind, RipressErrorKind::NotFound)
            }
            _ => panic!("Expected error"),
        }

        let mut params = RouteParams::new();
        params.insert("invalid", "not-a-number");

        match params.get_int("invalid") {
            Err(e) => {
                assert_eq!(e.kind, RipressErrorKind::ParseError);
                assert_eq!(
                    e.message,
                    "Failed to parse route param 'invalid' from: i32 to: 'not-a-number'"
                )
            }
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn test_defaults() {
        let mut params = RouteParams::new();
        params.insert("valid", "10");

        assert_eq!(params.get_or_default("valid", 5), 10);
        assert_eq!(params.get_or_default("missing", 5), 5);
        assert_eq!(params.get_or_default("invalid", 5), 5);
        assert_eq!(
            params.get_or_parse_default("invalid", 5),
            Err(RipressError {
                kind: RipressErrorKind::NotFound,
                message: format!("Route Param 'invalid' not found"),
            })
        );
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

    #[test]
    fn test_display_not_found() {
        let err = ParamError::NotFound("user_id".into());
        let output = format!("{}", err);
        assert_eq!(output, "Route parameter 'user_id' not found");
    }

    #[test]
    fn test_display_parse_error() {
        let err = ParamError::ParseError {
            param: "age".into(),
            value: "abc".into(),
            target_type: "u32".into(),
        };
        let output = format!("{}", err);
        assert_eq!(
            output,
            "Failed to parse parameter 'age' with value 'abc' as u32"
        );
    }

    #[test]
    fn test_default_route_param() {
        let route_param = RouteParams::default();

        assert_eq!(
            route_param,
            RouteParams {
                params: HashMap::new()
            }
        );
    }

    fn make_route_params(pairs: Vec<(&str, &str)>) -> RouteParams {
        let mut map = HashMap::new();
        for (k, v) in pairs {
            map.insert(k.to_string(), v.to_string());
        }
        RouteParams { params: map }
    }

    #[test]
    fn test_display_empty_params() {
        let params = make_route_params(vec![]);
        assert_eq!(format!("{}", params), "");
    }

    #[test]
    fn test_display_single_param() {
        let params = make_route_params(vec![("user_id", "42")]);
        let output = format!("{}", params);
        assert!(output.contains("user_id=42"));
    }

    #[test]
    fn test_display_multiple_params() {
        let params = make_route_params(vec![("user_id", "42"), ("format", "json")]);
        let output = format!("{}", params);

        // Order not guaranteed, so check both substrings
        assert!(output.contains("user_id=42"));
        assert!(output.contains("format=json"));
        // Ensure they are separated by comma+space
        assert!(output.contains(", "));
    }

    #[test]
    fn test_index_existing_param() {
        let params = make_route_params(vec![("user_id", "42")]);
        assert_eq!(&params["user_id"], "42");
    }

    #[test]
    #[should_panic(expected = "Route parameter 'missing' not found")]
    fn test_index_missing_param_panics() {
        let params = make_route_params(vec![("user_id", "42")]);
        let _ = &params["missing"]; // should panic
    }

    #[test]
    fn test_from_hashmap() {
        let mut map = HashMap::new();
        map.insert("user_id".to_string(), "42".to_string());
        map.insert("format".to_string(), "json".to_string());

        let params: RouteParams = map.clone().into();
        // Should contain the same keys/values
        assert_eq!(&params["user_id"], "42");
        assert_eq!(&params["format"], "json");
    }

    #[test]
    fn test_into_hashmap() {
        let params = make_route_params(vec![("user_id", "42"), ("format", "json")]);
        let map: HashMap<String, String> = params.into();

        assert_eq!(map.get("user_id"), Some(&"42".to_string()));
        assert_eq!(map.get("format"), Some(&"json".to_string()));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_roundtrip_hashmap_conversion() {
        let mut original = HashMap::new();
        original.insert("a".to_string(), "1".to_string());
        original.insert("b".to_string(), "2".to_string());

        let params: RouteParams = original.clone().into();
        let back: HashMap<String, String> = params.into();

        assert_eq!(original, back);
    }

    #[test]
    fn test_everything_else() {
        let mut params = RouteParams::new();

        params.insert("id", "123");

        assert_eq!(params.is_empty(), false);
        assert_eq!(params.len(), 1);
        assert_eq!(params.names().count(), 1);

        let map = params.into_map();
        assert_eq!(map.get("id"), Some(&"123".to_string()));
        assert_eq!(map.len(), 1);
    }
}
