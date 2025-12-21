#[cfg(test)]
mod form_data_tests {
    use crate::req::body::form_data::FormData;
    use ahash::AHashMap;

    #[test]
    fn test_form_data_new() {
        let form = FormData::new();
        assert!(form.is_empty());
        assert_eq!(form.len(), 0);
    }

    #[test]
    fn test_form_data_with_capacity() {
        let form = FormData::with_capacity(10);
        assert!(form.is_empty());
    }

    #[test]
    fn test_form_data_insert() {
        let mut form = FormData::new();
        assert_eq!(form.insert("username", "alice"), None);
        assert_eq!(form.insert("username", "bob"), Some("alice".to_string()));
    }

    #[test]
    fn test_form_data_get() {
        let mut form = FormData::new();
        form.insert("email", "test@example.com");

        assert_eq!(form.get("email"), Some("test@example.com"));
        assert_eq!(form.get("missing"), None);
    }

    #[test]
    fn test_form_data_contains_key() {
        let mut form = FormData::new();
        form.insert("password", "secret");

        assert!(form.contains_key("password"));
        assert!(!form.contains_key("username"));
    }

    #[test]
    fn test_form_data_remove() {
        let mut form = FormData::new();
        form.insert("token", "abc123");

        assert_eq!(form.remove("token"), Some("abc123".to_string()));
        assert_eq!(form.remove("token"), None);
    }

    #[test]
    fn test_form_data_len() {
        let mut form = FormData::new();
        assert_eq!(form.len(), 0);

        form.insert("field1", "value1");
        assert_eq!(form.len(), 1);

        form.insert("field2", "value2");
        assert_eq!(form.len(), 2);

        form.remove("field1");
        assert_eq!(form.len(), 1);
    }

    #[test]
    fn test_form_data_is_empty() {
        let mut form = FormData::new();
        assert!(form.is_empty());

        form.insert("key", "value");
        assert!(!form.is_empty());
    }

    #[test]
    fn test_form_data_clear() {
        let mut form = FormData::new();
        form.insert("a", "1");
        form.insert("b", "2");

        form.clear();
        assert!(form.is_empty());
        assert_eq!(form.len(), 0);
    }

    #[test]
    fn test_form_data_iter() {
        let mut form = FormData::new();
        form.insert("key1", "value1");
        form.insert("key2", "value2");

        let mut count = 0;
        for (key, value) in form.iter() {
            assert!(key == "key1" || key == "key2");
            assert!(value == "value1" || value == "value2");
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_form_data_keys() {
        let mut form = FormData::new();
        form.insert("username", "alice");
        form.insert("email", "alice@example.com");

        let keys: Vec<&str> = form.keys().collect();

        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"username"));
        assert!(keys.contains(&"email"));
    }

    #[test]
    fn test_form_data_values() {
        let mut form = FormData::new();
        form.insert("field1", "value1");
        form.insert("field2", "value2");

        let values: Vec<&str> = form.values().collect();

        assert_eq!(values.len(), 2);

        assert!(values.contains(&&"value1"));
        assert!(values.contains(&&"value2"));
    }

    #[test]
    fn test_form_data_from_hashmap() {
        let mut map = AHashMap::new();
        map.insert("name".to_string(), "John".to_string());
        map.insert("age".to_string(), "30".to_string());

        let form = FormData::from(map);
        assert_eq!(form.len(), 2);
        assert_eq!(form.get("name"), Some("John"));
        assert_eq!(form.get("age"), Some("30"));
    }

    #[test]
    fn test_form_data_index() {
        let mut form = FormData::new();
        form.insert("title", "Hello");

        assert_eq!(&form["title"], "Hello");
    }

    #[test]
    #[should_panic]
    fn test_form_data_index_missing() {
        let form = FormData::new();
        let _ = &form["nonexistent"];
    }

    #[test]
    fn test_form_data_special_characters() {
        let mut form = FormData::new();
        form.insert("special_chars", "hello world!@#$%");
        form.insert("unicode", "こんにちは");

        assert_eq!(form.get("special_chars"), Some("hello world!@#$%"));
        assert_eq!(form.get("unicode"), Some("こんにちは"));
    }

    #[test]
    fn test_form_data_empty_values() {
        let mut form = FormData::new();
        form.insert("empty", "");
        form.insert("whitespace", "   ");

        assert_eq!(form.get("empty"), Some(""));
        assert_eq!(form.get("whitespace"), Some("   "));
    }

    #[test]
    fn test_form_data_long_values() {
        let mut form = FormData::new();
        let long_value = "x".repeat(10000);
        form.insert("long", &long_value);

        assert_eq!(form.get("long"), Some(long_value.as_str()));
    }

    #[test]
    fn test_form_data_multiple_operations() {
        let mut form = FormData::new();

        // Insert multiple fields
        form.insert("field1", "value1");
        form.insert("field2", "value2");
        form.insert("field3", "value3");

        // Update one
        form.insert("field2", "updated");

        // Remove one
        form.remove("field3");

        assert_eq!(form.len(), 2);
        assert_eq!(form.get("field1"), Some("value1"));
        assert_eq!(form.get("field2"), Some("updated"));
        assert_eq!(form.get("field3"), None);
    }
}
