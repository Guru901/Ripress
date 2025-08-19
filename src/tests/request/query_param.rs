#[cfg(test)]
mod tests {
    use crate::req::query_params::{QueryParamError, QueryParams, SortDirection};
    use std::collections::HashMap;

    #[test]
    fn test_query_params_display_empty_value() {
        let mut qp = QueryParams::new();
        qp.insert("flag", "");

        assert_eq!(qp.to_string(), "flag");
    }

    #[test]
    fn test_query_params_default_is_empty() {
        let qp = QueryParams::default();
        assert_eq!(qp.to_string(), "");
        assert!(qp.inner.is_empty());
    }

    #[test]
    fn test_query_params_index_existing() {
        let mut qp = QueryParams::new();
        qp.insert("page", "42");

        assert_eq!(&qp["page"], "42");
    }

    #[test]
    #[should_panic(expected = "Query parameter 'missing' not found")]
    fn test_query_params_index_missing_panics() {
        let qp = QueryParams::new();
        let _ = &qp["missing"];
    }

    #[test]
    fn test_query_params_from_hashmap() {
        let mut map = HashMap::new();
        map.insert("page".to_string(), "7".to_string());
        map.insert("q".to_string(), "rust".to_string());

        let qp: QueryParams = map.into();
        let output = qp.to_string();

        assert!(output.contains("page=7"));
        assert!(output.contains("q=rust"));
    }

    #[test]
    fn test_query_params_multiple_values_same_key() {
        let mut qp = QueryParams::new();
        qp.insert("tag", "rust");
        qp.insert("tag", "web");

        let output = qp.to_string();
        println!("{}", output);
        // order is not guaranteed, so just check both substrings
        assert!(output.contains("tag=rust"));
        assert!(output.contains("tag=web"));
    }

    #[test]
    fn test_sort_direction_display_asc() {
        let dir = SortDirection::Asc;
        assert_eq!(dir.to_string(), "asc");
        assert_eq!(format!("{}", dir), "asc");
    }

    #[test]
    fn test_sort_direction_display_desc() {
        let dir = SortDirection::Desc;
        assert_eq!(dir.to_string(), "desc");
        assert_eq!(format!("{}", dir), "desc");
    }

    #[test]
    fn test_display_not_found() {
        let err = QueryParamError::NotFound("page".into());
        let output = format!("{}", err);
        assert_eq!(output, "Query parameter 'page' not found");
    }

    #[test]
    fn test_display_parse_error() {
        let err = QueryParamError::ParseError {
            param: "limit".into(),
            value: "abc".into(),
            target_type: "u32".into(),
        };
        let output = format!("{}", err);
        assert_eq!(
            output,
            "Failed to parse parameter 'limit' with value 'abc' as u32"
        );
    }

    #[test]
    fn test_display_multiple_values() {
        let err = QueryParamError::MultipleValues {
            param: "tag".into(),
            values: vec!["rust".into(), "web".into()],
        };
        let output = format!("{}", err);
        assert_eq!(
            output,
            "Multiple values found for parameter 'tag': [\"rust\", \"web\"]"
        );
    }

    #[test]
    fn test_from_query_string() {
        let query =
            QueryParams::from_query_string("page=2&limit=10&tags=rust&tags=web&active=true");

        assert_eq!(query.get("page"), Some("2"));
        assert_eq!(query.get("limit"), Some("10"));
        assert_eq!(query.get("active"), Some("true"));

        let tags = query.get_all("tags").unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"web".to_string()));
    }

    #[test]
    fn test_query_type_parsing() {
        let query = QueryParams::from_query_string("page=2&limit=10&active=true&price=19.99");

        assert_eq!(query.get_int("page").unwrap(), 2);
        assert_eq!(query.get_uint("limit").unwrap(), 10);
        assert_eq!(query.get_bool("active").unwrap(), true);
        assert_eq!(query.get_float("price").unwrap(), 19.99);
    }

    #[test]
    fn test_query_multiple_values() {
        let query = QueryParams::from_query_string("tags=rust&tags=web&tags=backend");

        let tags = query.get_all_parsed::<String>("tags").unwrap();
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"web".to_string()));
        assert!(tags.contains(&"backend".to_string()));
    }

    #[test]
    fn test_convenience_methods() {
        let query =
            QueryParams::from_query_string("page=3&limit=25&q=search+term&sort=name&order=desc");

        assert_eq!(query.page(), 3);
        assert_eq!(query.limit(), 25);
        assert_eq!(query.search_query(), Some("search+term"));
        assert_eq!(query.sort(), Some("name"));
        assert_eq!(query.sort_direction(), SortDirection::Desc);
    }

    #[test]
    fn test_boolean_parsing() {
        let query = QueryParams::from_query_string(
            "active=true&debug=1&verbose=yes&feature&disabled=false",
        );

        assert_eq!(query.get_bool("active").unwrap(), true);
        assert_eq!(query.get_bool("debug").unwrap(), true);
        assert_eq!(query.get_bool("verbose").unwrap(), true);
        assert_eq!(query.get_bool("disabled").unwrap(), false);
        assert!(query.is_truthy("feature")); // Parameter exists without value
    }

    #[test]
    fn test_filters() {
        let query = QueryParams::from_query_string(
            "filter[status]=active&filter[type]=user&filter[role]=admin",
        );

        let filters = query.filters();
        assert_eq!(filters.get("status").unwrap()[0], "active");
        assert_eq!(filters.get("type").unwrap()[0], "user");
        assert_eq!(filters.get("role").unwrap()[0], "admin");
    }

    #[test]
    fn test_query_defaults() {
        let query = QueryParams::from_query_string("existing=42");

        assert_eq!(query.get_or_default("existing", 0), 42);
        assert_eq!(query.get_or_default("missing", 100), 100);
        assert_eq!(query.page(), 1); // Default page
        assert_eq!(query.limit(), 20); // Default limit
    }
}
