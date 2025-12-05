#[cfg(test)]
mod tests {
    use crate::{
        helpers::{
            extract_boundary, find_subsequence, get_all_query, parse_multipart_form, path_matches,
        },
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

    #[test]
    fn extracts_normal() {
        let ct = "multipart/form-data; boundary=abcde1234";
        assert_eq!(extract_boundary(ct), Some("abcde1234".to_string()));
    }

    #[test]
    fn extracts_with_quotes() {
        let ct = "multipart/form-data; boundary=\"abcde1234\"";
        assert_eq!(extract_boundary(ct), Some("abcde1234".to_string()));
    }

    #[test]
    fn extracts_with_whitespace() {
        let ct = "multipart/form-data ; boundary = abcde1234 ";
        assert_eq!(extract_boundary(ct), Some("abcde1234".to_string()));
    }

    #[test]
    fn extracts_with_other_params() {
        let ct = "multipart/form-data; charset=UTF-8; boundary=alpha";
        assert_eq!(extract_boundary(ct), Some("alpha".to_string()));
    }

    #[test]
    fn extracts_with_other_order() {
        let ct = "multipart/form-data; boundary=foo; charset=UTF-8";
        assert_eq!(extract_boundary(ct), Some("foo".to_string()));
    }

    #[test]
    fn extracts_with_uppercase_key() {
        let ct = "multipart/form-data; BOUNDARY=foobar";
        assert_eq!(extract_boundary(ct), Some("foobar".to_string()));
    }

    #[test]
    fn returns_none_missing() {
        let ct = "application/json";
        assert_eq!(extract_boundary(ct), None);
    }

    #[test]
    fn returns_none_empty_boundary() {
        let ct = "multipart/form-data; boundary=";
        assert_eq!(extract_boundary(ct), None);
    }

    #[test]
    fn extracts_nonstandard_no_mime() {
        let ct = "something-else; boundary=yo";
        assert_eq!(extract_boundary(ct), Some("yo".to_string()));
    }

    #[test]
    fn finds_basic_match() {
        let haystack = b"abcdefg";
        let needle = b"cde";
        assert_eq!(find_subsequence(haystack, needle), Some(2));
    }

    #[test]
    fn finds_at_start() {
        let haystack = b"abc";
        let needle = b"ab";
        assert_eq!(find_subsequence(haystack, needle), Some(0));
    }

    #[test]
    fn finds_at_end() {
        let haystack = b"xyzabc";
        let needle = b"abc";
        assert_eq!(find_subsequence(haystack, needle), Some(3));
    }

    #[test]
    fn returns_none_if_no_match() {
        let haystack = b"abcdef";
        let needle = b"gh";
        assert_eq!(find_subsequence(haystack, needle), None);
    }

    #[test]
    fn returns_zero_on_empty_needle() {
        let haystack = b"abcdef";
        let needle: &[u8] = b"";
        assert_eq!(find_subsequence(haystack, needle), Some(0));
    }

    #[test]
    fn handles_empty_haystack() {
        let haystack: &[u8] = b"";
        let needle = b"abc";
        assert_eq!(find_subsequence(haystack, needle), None);
    }

    #[test]
    fn handles_both_empty() {
        let haystack: &[u8] = b"";
        let needle: &[u8] = b"";
        assert_eq!(find_subsequence(haystack, needle), Some(0));
    }

    #[test]
    fn finds_middle_multiple() {
        let haystack = b"aaabaaabaaa";
        let needle = b"ba";
        assert_eq!(find_subsequence(haystack, needle), Some(3));
    }

    fn make_body(parts: &[(&str, &str, Option<&[u8]>)], boundary: &str) -> Vec<u8> {
        let mut body = Vec::new();
        for (i, (field, value, file_bytes)) in parts.iter().enumerate() {
            if i != 0 {
                body.extend_from_slice(b"\r\n");
            }
            body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            if let Some(bytes) = file_bytes {
                body.extend_from_slice(
                    format!(
                        "Content-Disposition: form-data; name=\"{}\"; filename=\"file.bin\"\r\n\r\n",
                        field
                    )
                    .as_bytes(),
                );
                body.extend_from_slice(bytes);
            } else {
                body.extend_from_slice(
                    format!(
                        "Content-Disposition: form-data; name=\"{}\"\r\n\r\n{}",
                        field, value
                    )
                    .as_bytes(),
                );
            }
        }
        body.extend_from_slice(format!("\r\n--{}--", boundary).as_bytes());
        body
    }

    #[test]
    fn parses_simple_field() {
        let boundary = "AaB03x";
        let body = format!(
            "--AaB03x\r\nContent-Disposition: form-data; name=\"submit-name\"\r\n\r\nLarry\r\n--AaB03x--"
        );
        let (fields, files) = parse_multipart_form(body.as_bytes(), &boundary.to_string());
        assert_eq!(fields, vec![("submit-name", "Larry")]);
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn parses_multiple_fields() {
        let boundary = "xyz";
        let body = format!(
            "--xyz\r\nContent-Disposition: form-data; name=\"f1\"\r\n\r\nv1\r\n--xyz\r\nContent-Disposition: form-data; name=\"f2\"\r\n\r\nv2\r\n--xyz--"
        );
        let (fields, files) = parse_multipart_form(body.as_bytes(), &boundary.to_string());
        assert_eq!(fields, vec![("f1", "v1"), ("f2", "v2")]);
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn parses_file_and_field() {
        let boundary = "b";
        let file_content = b"\xDE\xAD\xBE\xEF";
        let body = make_body(
            &[("desc", "mydesc", None), ("upload", "", Some(file_content))],
            boundary,
        );
        let (fields, files) = parse_multipart_form(&body, &boundary.to_string());
        assert!(fields.contains(&("desc", "mydesc")));
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, file_content);
        assert_eq!(files[0].1, Some("upload"));
    }

    #[test]
    fn parses_multiple_files_and_fields() {
        let boundary = "b7";
        let content1 = b"filecontent1";
        let content2 = b"filecontent2";
        let body = make_body(
            &[
                ("n1", "v1", None),
                ("file1", "", Some(content1)),
                ("n2", "v2", None),
                ("file2", "", Some(content2)),
            ],
            boundary,
        );
        let (fields, files) = parse_multipart_form(&body, &boundary.to_string());
        assert!(fields.contains(&("n1", "v1")));
        assert!(fields.contains(&("n2", "v2")));
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].0, content1);
        assert_eq!(files[1].0, content2);
        assert_eq!(files[0].1, Some("file1"));
        assert_eq!(files[1].1, Some("file2"));
    }

    #[test]
    fn handles_file_name_variants() {
        // filename*
        let boundary = "multistar";
        let body = format!(
            "--multistar\r\nContent-Disposition: form-data; name=\"file\"; filename*=\"myfile.txt\"\r\n\r\nabc\r\n--multistar--"
        );
        let (fields, files) = parse_multipart_form(body.as_bytes(), &boundary.to_string());
        assert_eq!(fields.len(), 0);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].1, Some("file"));
        assert_eq!(files[0].0, b"abc");
    }

    #[test]
    fn trims_crlf_on_field() {
        let boundary = "wxc";
        let value = "a_line\r\n";
        let body = format!(
            "--wxc\r\nContent-Disposition: form-data; name=\"nm\"\r\n\r\n{}--wxc--",
            value
        );
        let (fields, files) = parse_multipart_form(body.as_bytes(), &boundary.to_string());
        assert_eq!(fields, vec![("nm", "a_line")]);
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn returns_empty_for_missing_boundary() {
        let boundary = "abs";
        let body = b"--xxx\r\nContent-Disposition: form-data; name=\"nm\"\r\n\r\nvv\r\n--xxx--";
        let (fields, files) = parse_multipart_form(body, &boundary.to_string());
        assert_eq!(fields.len(), 0);
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn handles_non_utf8_file_content() {
        let boundary = "binary";
        let file_content = b"\xF0\x90\x80\x80\xFF";
        let body = make_body(&[("file", "", Some(file_content))], boundary);
        let (_, files) = parse_multipart_form(&body, &boundary.to_string());
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, file_content);
    }

    #[test]
    fn handles_no_crlf_after_last_field() {
        let boundary = "plain";
        let body = b"--plain\r\nContent-Disposition: form-data; name=\"foo\"\r\n\r\nbar--plain--";
        let (fields, files) = parse_multipart_form(body, &boundary.to_string());
        assert_eq!(fields, vec![("foo", "bar")]);
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn trims_crlf_from_file_part() {
        let boundary = "def";
        let file_content = b"abc\r\n";
        let body = format!(
            "--def\r\nContent-Disposition: form-data; name=\"up\"; filename=\"f.txt\"\r\n\r\n{}--def--",
            std::str::from_utf8(file_content).unwrap()
        );
        let (fields, files) = parse_multipart_form(body.as_bytes(), &boundary.to_string());
        assert_eq!(fields.len(), 0);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, b"abc");
    }
}
