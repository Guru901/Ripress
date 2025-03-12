#[cfg(test)]
mod tests {
    use crate::request::HttpRequest;

    #[test]
    fn test_get_query() {
        let mut req = HttpRequest::new();
        req.set_query("q", "Ripress");

        assert_eq!(req.get_query("q"), Some("Ripress".to_string()));

        assert_eq!(req.get_query("nonexistent"), None);
    }

    #[test]
    fn test_get_param() {
        let mut req = HttpRequest::new();
        req.set_param("q", "Ripress");

        assert_eq!(req.get_params("q"), Some("Ripress".to_string()));

        assert_eq!(req.get_params("nonexistent"), None);
    }

    #[test]
    fn test_json_body() {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct User {
            id: u32,
            name: String,
        }

        let mut req = HttpRequest::new();
        req.set_json(User {
            id: 1,
            name: "John Doe".to_string(),
        });

        assert_eq!(
            req.json::<User>().unwrap(),
            User {
                id: 1,
                name: "John Doe".to_string()
            }
        );

        assert!(req.json::<String>().is_err());
    }

    #[test]
    fn test_text_body() {
        let mut req = HttpRequest::new();
        req.set_text("Ripress");

        assert_eq!(req.text(), Ok("Ripress".to_string()));

        req.set_text("");
        assert_eq!(req.text(), Ok("".to_string()));
    }

    #[test]
    fn test_form_data() {
        let mut req = HttpRequest::new();
        req.set_form("key", "value");

        assert_eq!(req.form_data().unwrap().get("key").unwrap(), "value");
        assert_eq!(req.form_data().unwrap().get("nonexistent"), None);

        req.set_form("another_key", "another_value");
        let form_data = req.form_data().unwrap();
        dbg!(&form_data);
        assert_eq!(form_data.get("key").unwrap(), "value");
        assert_eq!(form_data.get("another_key").unwrap(), "another_value");
    }
}
