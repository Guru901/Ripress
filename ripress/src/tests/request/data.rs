#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::req::{HttpRequest, request_data::RequestData};

    #[test]
    fn test_iter_request_data() {
        let mut req = HttpRequest::new();
        req.set_data("data_key", "data_value");

        req.data.into_iter().for_each(|(key, value)| {
            assert_eq!(key, b"data_key");
            assert_eq!(value, b"data_value");
        });
    }

    #[test]
    fn test_shrink_to_fit_request_data() {
        let mut req = HttpRequest::new();
        req.set_data("data_key", "data_value");

        assert!(req.data.contains_key("data_key"));

        req.data.shrink_to_fit();
        assert_eq!(req.data.byte_size(), 162);

        req.data.remove("data_key");
        assert!(!req.data.contains_key("data_key"));
    }

    #[test]
    fn test_from_map_request_data() {
        let mut data = HashMap::new();
        data.insert("data_key", "data_value");

        let request_data = RequestData::from_map(data);

        assert_eq!(request_data.get("data_key"), Some("data_value".to_string()));
    }

    fn make_request_data(pairs: Vec<(&str, Vec<u8>)>) -> RequestData {
        let mut data = RequestData::new();
        for (k, v) in pairs {
            data.insert(k.to_string(), v);
        }
        data
    }

    #[test]
    fn test_display_empty_request_data() {
        let data = make_request_data(vec![]);
        assert_eq!(format!("{}", data), "RequestData {  }");
    }

    #[test]
    fn test_display_utf8_value() {
        let data = make_request_data(vec![("username", b"gurvinder".to_vec())]);
        let output = format!("{}", data);
        assert!(output.contains("username: gurvinder"));
    }

    #[test]
    fn test_display_non_utf8_value() {
        let data = make_request_data(vec![("bin", vec![0xff, 0xfe, 0xfd])]);
        let output = format!("{}", data);
        assert!(output.contains("bin: [255, 254, 253]"));
    }

    #[test]
    fn test_display_multiple_pairs() {
        let data = make_request_data(vec![("a", b"1".to_vec()), ("b", b"2".to_vec())]);
        let output = format!("{}", data);

        assert!(output.contains("a: 1"));
        assert!(output.contains("b: 2"));
        assert!(output.starts_with("RequestData {"));
        assert!(output.ends_with("}"));
        assert!(output.contains(", "));
    }
}
