#[cfg(test)]
mod tests {
    use crate::{
        helpers::{get_all_query, path_matches},
        req::query_params::QueryParams,
    };

    #[test]
    fn test_exact_match() {
        assert!(path_matches("/api", "/api"));
        assert!(path_matches("", ""));
        assert!(path_matches("/", "/"));
    }

    #[test]
    fn test_prefix_with_slash() {
        assert!(path_matches("/api", "/api/v1"));
        assert!(path_matches("/foo", "/foo/bar/baz"));
        assert!(path_matches("/", "/something"));
    }

    #[test]
    fn test_no_match() {
        assert!(!path_matches("/api", "/apix"));
        assert!(!path_matches("/foo", "/foobar"));
        assert!(!path_matches("/foo", "/fo"));
        assert!(!path_matches("/foo", "/bar/foo"));
    }

    #[test]
    fn test_prefix_is_empty() {
        assert!(path_matches("", ""));
        assert!(path_matches("", "/anything"));
    }

    #[test]
    fn test_path_is_empty() {
        assert!(!path_matches("/api", ""));
        assert!(path_matches("", ""));
    }

    #[test]
    fn test_trailing_slash_in_prefix() {
        // "/api/" as prefix should match "/api/" and "/api/foo"
        assert!(path_matches("/api/", "/api/"));
        assert!(path_matches("/api/", "/api/foo"));
        assert!(!path_matches("/api/", "/api")); // "/api" does not start with "/api//"
    }

    #[test]
    fn test_get_all_query_empty() {
        let queries = QueryParams::new();
        let result = get_all_query(&queries);
        assert_eq!(result, "");
    }

    #[test]
    fn test_get_all_query_single() {
        let mut queries = QueryParams::new();
        queries.insert("key", "value");
        let result = get_all_query(&queries);
        assert_eq!(result, "key=value");
    }

    #[test]
    fn test_get_all_query_multiple() {
        let mut queries = QueryParams::new();
        queries.insert("foo", "bar");
        queries.insert("baz", "qux");
        let result = get_all_query(&queries);
        // Order is not guaranteed, so check both possibilities
        let expected1 = "foo=bar&baz=qux";
        let expected2 = "baz=qux&foo=bar";
        assert!(result == expected1 || result == expected2);
    }

    #[test]
    fn test_get_all_query_url_encoding() {
        let mut queries = QueryParams::new();
        queries.insert("sp ce", "v@lue+1");
        let result = get_all_query(&queries);
        // "sp ce" -> "sp+ce", "v@lue+1" -> "v%40lue%2B1"
        assert!(result.contains("sp+ce="));
        assert!(result.contains("v%40lue%2B1"));
    }
}
