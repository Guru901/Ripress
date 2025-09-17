#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use serde_json::json;
    use std::collections::hash_map::HashMap;

    use crate::{
        error::{RipressError, RipressErrorKind},
        req::body::{
            FormData, RequestBody, RequestBodyContent, RequestBodyType, TextData,
            text_data::TextDataError,
        },
    };

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
        let ripress_error = RipressError::from(TextDataError::TooLarge {
            size: 1000,
            limit: 500,
        });

        assert_eq!(result, Err(ripress_error));
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

    #[test]
    fn test_new_text() {
        let text_data = TextData::new(String::from("Hello, world!"));
        let body = RequestBody::new_text(text_data.clone());

        assert_eq!(body.content_type, RequestBodyType::TEXT);
        match body.content {
            RequestBodyContent::TEXT(ref data) => {
                assert_eq!(data.as_str().unwrap(), "Hello, world!");
            }
            _ => panic!("Expected TEXT content"),
        }
    }

    #[test]
    fn test_new_text_empty() {
        let text_data = TextData::new(String::new());
        let body = RequestBody::new_text(text_data);

        assert_eq!(body.content_type, RequestBodyType::TEXT);
        match body.content {
            RequestBodyContent::TEXT(ref data) => {
                assert_eq!(data.as_str().unwrap(), "");
            }
            _ => panic!("Expected TEXT content"),
        }
    }

    #[test]
    fn test_new_text_multiline() {
        let text_content = "Line 1\nLine 2\nLine 3";
        let text_data = TextData::new(String::from(text_content));
        let body = RequestBody::new_text(text_data);

        assert_eq!(body.content_type, RequestBodyType::TEXT);
        match body.content {
            RequestBodyContent::TEXT(ref data) => {
                assert_eq!(data.as_str().unwrap(), text_content);
            }
            _ => panic!("Expected TEXT content"),
        }
    }

    #[test]
    fn test_new_binary() {
        let test_bytes = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello" in bytes
        let bytes = Bytes::from(test_bytes.clone());
        let body = RequestBody::new_binary(bytes.clone());

        assert_eq!(body.content_type, RequestBodyType::BINARY);
        match body.content {
            RequestBodyContent::BINARY(ref data) => {
                assert_eq!(data.as_ref(), test_bytes.as_slice());
            }
            _ => panic!("Expected BINARY content"),
        }
    }

    #[test]
    fn test_new_binary_empty() {
        let bytes = Bytes::new();
        let body = RequestBody::new_binary(bytes);

        assert_eq!(body.content_type, RequestBodyType::BINARY);
        match body.content {
            RequestBodyContent::BINARY(ref data) => {
                assert!(data.is_empty());
            }
            _ => panic!("Expected BINARY content"),
        }
    }

    #[test]
    fn test_new_form() {
        let mut form_data = FormData::new();
        form_data.insert("username", "alice");
        form_data.insert("password", "secret123");
        form_data.insert("remember_me", "on");

        let body = RequestBody::new_form(form_data.clone());

        assert_eq!(body.content_type, RequestBodyType::FORM);
        match body.content {
            RequestBodyContent::FORM(ref form) => {
                assert_eq!(form.get("username"), Some("alice"));
                assert_eq!(form.get("password"), Some("secret123"));
                assert_eq!(form.get("remember_me"), Some("on"));
                assert_eq!(form.len(), 3);
            }
            _ => panic!("Expected FORM content"),
        }
    }

    #[test]
    fn test_new_form_empty() {
        let form_data = FormData::new();
        let body = RequestBody::new_form(form_data);

        assert_eq!(body.content_type, RequestBodyType::FORM);
        match body.content {
            RequestBodyContent::FORM(ref form) => {
                assert_eq!(form.len(), 0);
            }
            _ => panic!("Expected FORM content"),
        }
    }

    #[test]
    fn test_new_json_with_json_macro() {
        let json_value = json!({
            "username": "alice",
            "email": "alice@example.com",
            "age": 30,
            "active": true,
            "preferences": {
                "theme": "dark",
                "notifications": true
            }
        });

        let body = RequestBody::new_json(json_value.clone());

        assert_eq!(body.content_type, RequestBodyType::JSON);
        match body.content {
            RequestBodyContent::JSON(ref value) => {
                assert_eq!(value["username"], "alice");
                assert_eq!(value["email"], "alice@example.com");
                assert_eq!(value["age"], 30);
                assert_eq!(value["active"], true);
                assert_eq!(value["preferences"]["theme"], "dark");
                assert_eq!(value["preferences"]["notifications"], true);
            }
            _ => panic!("Expected JSON content"),
        }
    }

    #[test]
    fn test_new_json_with_primitive_string() {
        let body = RequestBody::new_json("simple string");

        assert_eq!(body.content_type, RequestBodyType::JSON);
        match body.content {
            RequestBodyContent::JSON(ref value) => {
                assert_eq!(value, "simple string");
            }
            _ => panic!("Expected JSON content"),
        }
    }

    #[test]
    fn test_new_json_with_number() {
        let body = RequestBody::new_json(42);

        assert_eq!(body.content_type, RequestBodyType::JSON);
        match body.content {
            RequestBodyContent::JSON(ref value) => {
                assert_eq!(value, &json!(42));
            }
            _ => panic!("Expected JSON content"),
        }
    }

    #[test]
    fn test_new_json_with_array() {
        let array = vec![1, 2, 3, 4, 5];
        let json_value = serde_json::to_value(array.clone()).unwrap();
        let body = RequestBody::new_json(json_value);

        assert_eq!(body.content_type, RequestBodyType::JSON);
        match body.content {
            RequestBodyContent::JSON(ref value) => {
                assert_eq!(value, &json!([1, 2, 3, 4, 5]));
            }
            _ => panic!("Expected JSON content"),
        }
    }

    #[test]
    fn test_new_json_with_null() {
        let body = RequestBody::new_json(serde_json::Value::Null);

        assert_eq!(body.content_type, RequestBodyType::JSON);
        match body.content {
            RequestBodyContent::JSON(ref value) => {
                assert!(value.is_null());
            }
            _ => panic!("Expected JSON content"),
        }
    }

    #[test]
    fn test_new_json_with_boolean() {
        let body_true = RequestBody::new_json(true);
        let body_false = RequestBody::new_json(false);

        assert_eq!(body_true.content_type, RequestBodyType::JSON);
        assert_eq!(body_false.content_type, RequestBodyType::JSON);

        match body_true.content {
            RequestBodyContent::JSON(ref value) => {
                assert_eq!(value, &json!(true));
            }
            _ => panic!("Expected JSON content"),
        }

        match body_false.content {
            RequestBodyContent::JSON(ref value) => {
                assert_eq!(value, &json!(false));
            }
            _ => panic!("Expected JSON content"),
        }
    }

    // RequestBodyType ToString tests

    #[test]
    fn test_request_body_type_to_string_json() {
        let body_type = RequestBodyType::JSON;
        assert_eq!(body_type.to_string(), "application/json");
    }

    #[test]
    fn test_request_body_type_to_string_text() {
        let body_type = RequestBodyType::TEXT;
        assert_eq!(body_type.to_string(), "text/plain");
    }

    #[test]
    fn test_request_body_type_to_string_form() {
        let body_type = RequestBodyType::FORM;
        assert_eq!(body_type.to_string(), "application/x-www-form-urlencoded");
    }

    #[test]
    fn test_request_body_type_to_string_binary() {
        let body_type = RequestBodyType::BINARY;
        assert_eq!(body_type.to_string(), "application/octet-stream");
    }

    #[test]
    fn test_request_body_type_to_string_empty() {
        let body_type = RequestBodyType::EMPTY;
        assert_eq!(body_type.to_string(), "");
    }

    #[test]
    fn test_all_request_body_types_to_string() {
        let types_and_expected = vec![
            (RequestBodyType::JSON, "application/json"),
            (RequestBodyType::TEXT, "text/plain"),
            (RequestBodyType::FORM, "application/x-www-form-urlencoded"),
            (RequestBodyType::BINARY, "application/octet-stream"),
            (RequestBodyType::EMPTY, ""),
        ];

        for (body_type, expected) in types_and_expected {
            assert_eq!(body_type.to_string(), expected);
        }
    }

    // Integration tests combining new_* methods with content type checking

    #[test]
    fn test_content_type_consistency_text() {
        let text_data = TextData::new(String::from("Test content"));
        let body = RequestBody::new_text(text_data);

        assert_eq!(body.content_type, RequestBodyType::TEXT);
        assert_eq!(body.content_type.to_string(), "text/plain");

        match body.content {
            RequestBodyContent::TEXT(_) => {} // Expected
            _ => panic!("Content type and content variant mismatch"),
        }
    }

    #[test]
    fn test_content_type_consistency_binary() {
        let bytes = Bytes::from_static(b"binary data");
        let body = RequestBody::new_binary(bytes);

        assert_eq!(body.content_type, RequestBodyType::BINARY);
        assert_eq!(body.content_type.to_string(), "application/octet-stream");

        match body.content {
            RequestBodyContent::BINARY(_) => {} // Expected
            _ => panic!("Content type and content variant mismatch"),
        }
    }

    #[test]
    fn test_content_type_consistency_form() {
        let mut form_data = FormData::new();
        form_data.insert("key", "value");
        let body = RequestBody::new_form(form_data);

        assert_eq!(body.content_type, RequestBodyType::FORM);
        assert_eq!(
            body.content_type.to_string(),
            "application/x-www-form-urlencoded"
        );

        match body.content {
            RequestBodyContent::FORM(_) => {} // Expected
            _ => panic!("Content type and content variant mismatch"),
        }
    }

    #[test]
    fn test_content_type_consistency_json() {
        let json_data = json!({"test": "data"});
        let body = RequestBody::new_json(json_data);

        assert_eq!(body.content_type, RequestBodyType::JSON);
        assert_eq!(body.content_type.to_string(), "application/json");

        match body.content {
            RequestBodyContent::JSON(_) => {} // Expected
            _ => panic!("Content type and content variant mismatch"),
        }
    }

    // Clone and Debug trait tests

    #[test]
    fn test_request_body_clone() {
        let original_body = RequestBody::new_json(json!({"key": "value"}));
        let cloned_body = original_body.clone();

        assert_eq!(original_body.content_type, cloned_body.content_type);

        match (&original_body.content, &cloned_body.content) {
            (RequestBodyContent::JSON(orig), RequestBodyContent::JSON(cloned)) => {
                assert_eq!(orig, cloned);
            }
            _ => panic!("Clone failed to preserve content"),
        }
    }

    #[test]
    fn test_request_body_type_clone_and_copy() {
        let original = RequestBodyType::JSON;
        let copied = original; // Copy
        let cloned = original.clone(); // Clone

        assert_eq!(original, copied);
        assert_eq!(original, cloned);
        assert_eq!(copied, cloned);
    }

    #[test]
    fn test_request_body_type_partial_eq() {
        assert_eq!(RequestBodyType::JSON, RequestBodyType::JSON);
        assert_eq!(RequestBodyType::TEXT, RequestBodyType::TEXT);
        assert_eq!(RequestBodyType::FORM, RequestBodyType::FORM);
        assert_eq!(RequestBodyType::BINARY, RequestBodyType::BINARY);
        assert_eq!(RequestBodyType::EMPTY, RequestBodyType::EMPTY);

        assert_ne!(RequestBodyType::JSON, RequestBodyType::TEXT);
        assert_ne!(RequestBodyType::FORM, RequestBodyType::BINARY);
        assert_ne!(RequestBodyType::EMPTY, RequestBodyType::JSON);
    }

    #[test]
    fn test_debug_formatting() {
        let body = RequestBody::new_json(json!({"debug": "test"}));
        let debug_str = format!("{:?}", body);

        // Just verify it doesn't panic and contains expected parts
        assert!(debug_str.contains("RequestBody"));
        assert!(debug_str.contains("content_type"));
        assert!(debug_str.contains("content"));
    }

    // Edge case tests

    #[test]
    fn test_json_with_special_characters() {
        let json_with_unicode = json!({
            "emoji": "🚀",
            "chinese": "你好",
            "escaped": "\"quotes\" and \\backslashes\\",
            "newlines": "line1\nline2\nline3"
        });

        let body = RequestBody::new_json(json_with_unicode.clone());

        match body.content {
            RequestBodyContent::JSON(ref value) => {
                assert_eq!(value["emoji"], "🚀");
                assert_eq!(value["chinese"], "你好");
                assert_eq!(value["escaped"], "\"quotes\" and \\backslashes\\");
                assert_eq!(value["newlines"], "line1\nline2\nline3");
            }
            _ => panic!("Expected JSON content"),
        }
    }

    #[test]
    fn test_form_with_special_characters() {
        let mut form_data = FormData::new();
        form_data.insert("special chars", "value with spaces & symbols!");
        form_data.insert("unicode", "🌟 unicode value");
        form_data.insert("empty_value", "");

        let body = RequestBody::new_form(form_data);

        match body.content {
            RequestBodyContent::FORM(ref form) => {
                assert_eq!(
                    form.get("special chars"),
                    Some("value with spaces & symbols!")
                );
                assert_eq!(form.get("unicode"), Some("🌟 unicode value"));
                assert_eq!(form.get("empty_value"), Some(""));
            }
            _ => panic!("Expected FORM content"),
        }
    }

    #[test]
    fn test_text_data_error_display_invalid_utf8() {
        // Create a FromUtf8Error artificially
        let bytes = vec![0, 159]; // invalid UTF-8 sequence
        let err = String::from_utf8(bytes).unwrap_err().utf8_error();

        let error = TextDataError::InvalidUtf8(err);
        let msg = format!("{}", error);
        assert!(msg.starts_with("Invalid UTF-8:"));
    }

    #[test]
    fn test_text_data_error_display_too_large() {
        let error = TextDataError::TooLarge {
            size: 2048,
            limit: 1024,
        };
        let msg = format!("{}", error);
        assert_eq!(msg, "Text too large: 2048 bytes (limit: 1024 bytes)");
    }

    #[test]
    fn test_text_data_error_display_empty() {
        let error = TextDataError::Empty;
        let msg = format!("{}", error);
        assert_eq!(msg, "Text data is empty");
    }

    #[test]
    fn test_from_string() {
        let s = String::from("Hello");
        let text = TextData::from(s.clone());
        assert_eq!(text.as_str_lossy(), s);
    }

    #[test]
    fn test_from_str() {
        let s = "Hello, world!";
        let text = TextData::from(s);
        assert_eq!(text.as_str_lossy(), s);
    }

    #[test]
    fn test_try_from_vec_u8_valid_utf8() {
        let bytes = b"valid utf8".to_vec();
        let text = TextData::try_from(bytes).unwrap();
        assert_eq!(text.as_str_lossy(), "valid utf8");
    }

    #[test]
    fn test_try_from_vec_u8_invalid_utf8() {
        let bytes = vec![0xff, 0xfe, 0xfd]; // invalid UTF-8
        let err = TextData::try_from(bytes).unwrap_err();
        if err.kind != RipressErrorKind::ParseError {
            panic!("expected InvalidInput error");
        }
    }

    #[test]
    fn test_try_from_textdata_to_string_valid() {
        let text = TextData::from("hello");
        let s: String = String::try_from(text).unwrap();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_try_from_textdata_to_string_invalid() {
        // Construct TextData from invalid bytes (simulate via from_bytes)
        let bytes = vec![0xff, 0xfe];
        let text = TextData::try_from(bytes.clone()).unwrap_err();
        assert!(matches!(
            text,
            RipressError {
                kind: RipressErrorKind::ParseError,
                ..
            }
        ));
    }

    #[test]
    fn test_deref_to_bytes() {
        let text = TextData::from("hello");
        let bytes: &[u8] = &*text; // deref coercion
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn test_debug_output_contains_expected_fields() {
        let text = TextData::from("abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        let debug_str = format!("{:?}", text);

        assert!(debug_str.contains("TextData"));
        assert!(debug_str.contains("len_bytes"));
        assert!(debug_str.contains("charset"));
        assert!(debug_str.contains("is_valid_utf8"));
        assert!(debug_str.contains("preview"));
    }

    #[test]
    fn test_default_and_display() {
        let mut form = FormData::default();
        form.insert("name", "Alice");
        form.insert("age", "30");

        let s = format!("{}", form);
        assert!(s.contains("name=Alice"));
        assert!(s.contains("age=30"));
    }

    #[test]
    fn test_from_hashmap_and_into_hashmap() {
        let mut map = HashMap::new();
        map.insert("k1".to_string(), "v1".to_string());
        map.insert("k2".to_string(), "v2".to_string());

        let form: FormData = map.clone().into();
        let back: HashMap<String, String> = form.into();
        assert_eq!(map, back);
    }

    #[test]
    fn test_from_iterator() {
        let pairs = vec![("x", "1"), ("y", "2")];
        let form: FormData = pairs.into_iter().collect();
        assert_eq!(form.len(), 2);
        assert_eq!(form.get("x").unwrap(), "1");
        assert_eq!(form.get("y").unwrap(), "2");
    }

    #[test]
    fn test_extend() {
        let mut form = FormData::new();
        form.insert("a", "10");
        form.extend(vec![("b", "20"), ("c", "30")]);

        assert_eq!(form.len(), 3);
        assert_eq!(form.get("b").unwrap(), "20");
    }

    #[test]
    fn test_index_access() {
        let mut form = FormData::new();
        form.insert("username", "alice");
        assert_eq!(&form["username"], "alice");
    }

    #[test]
    #[should_panic(expected = "FormData parameter 'missing' not found")]
    fn test_index_panics_for_missing_key() {
        let form = FormData::new();
        let _ = &form["missing"];
    }

    #[test]
    fn test_into_iterator_owned() {
        let mut form = FormData::new();
        form.insert("a", "1");
        form.insert("b", "2");

        let mut items: Vec<(String, String)> = form.into_iter().collect();
        items.sort();
        assert_eq!(
            items,
            vec![
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string()),
            ]
        );
    }

    #[test]
    fn test_into_iterator_ref() {
        let mut form = FormData::new();
        form.insert("a", "1");
        form.insert("b", "2");

        let mut items: Vec<(&String, &String)> = (&form).into_iter().collect();
        items.sort_by(|a, b| a.0.cmp(b.0));
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|(k, v)| *k == "a" && *v == "1"));
        assert!(items.iter().any(|(k, v)| *k == "b" && *v == "2"));
    }

    #[test]
    fn test_into_iterator_mut() {
        let mut form = FormData::new();
        form.insert("a", "1");
        form.insert("b", "2");

        for (_, v) in &mut form {
            v.push_str("_updated");
        }

        assert_eq!(form.get("a").unwrap(), "1_updated");
        assert_eq!(form.get("b").unwrap(), "2_updated");
    }
}
