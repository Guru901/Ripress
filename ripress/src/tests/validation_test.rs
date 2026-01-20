#[cfg(feature = "validation")]
#[cfg(test)]
mod validation_tests {
    use crate::helpers::FromRequest;
    use crate::req::body::json_data::{FromJson, JsonBodyValidated};
    use crate::req::body::{RequestBody, RequestBodyContent, RequestBodyType, TextData};
    use crate::req::HttpRequest;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use validator::Validate;

    #[derive(Debug, Deserialize, Serialize, Validate)]
    struct User {
        #[validate(length(min = 3, max = 20))]
        username: String,
        #[validate(email)]
        email: String,
        #[validate(range(min = 18, max = 120))]
        age: u8,
    }

    impl FromJson for User {
        fn from_json(data: &RequestBodyContent) -> Result<Self, String> {
            if let RequestBodyContent::JSON(json_val) = data {
                serde_json::from_value::<Self>(json_val.clone()).map_err(|e| e.to_string())
            } else {
                Err("Expected JSON body".to_string())
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize, Validate)]
    struct Product {
        #[validate(length(min = 1, max = 100))]
        name: String,
        #[validate(range(min = 0.01))]
        price: f64,
        #[validate(length(max = 500))]
        description: Option<String>,
    }

    impl FromJson for Product {
        fn from_json(data: &RequestBodyContent) -> Result<Self, String> {
            if let RequestBodyContent::JSON(json_val) = data {
                serde_json::from_value::<Self>(json_val.clone()).map_err(|e| e.to_string())
            } else {
                Err("Expected JSON body".to_string())
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize, Validate)]
    struct LoginRequest {
        #[validate(email)]
        email: String,
        #[validate(length(min = 8))]
        password: String,
    }

    impl FromJson for LoginRequest {
        fn from_json(data: &RequestBodyContent) -> Result<Self, String> {
            if let RequestBodyContent::JSON(json_val) = data {
                serde_json::from_value::<Self>(json_val.clone()).map_err(|e| e.to_string())
            } else {
                Err("Expected JSON body".to_string())
            }
        }
    }

    fn create_json_request(json_value: serde_json::Value) -> HttpRequest {
        let mut req = HttpRequest::default();
        req.body = RequestBody {
            content: RequestBodyContent::JSON(json_value),
            content_type: RequestBodyType::JSON,
        };
        req
    }

    fn create_text_request(text: &str) -> HttpRequest {
        let mut req = HttpRequest::default();
        req.body = RequestBody {
            content: RequestBodyContent::TEXT(TextData::new(text.to_string())),
            content_type: RequestBodyType::TEXT,
        };
        req
    }

    #[test]
    fn test_valid_json_passes_validation() {
        let json = json!({
            "username": "john_doe",
            "email": "john@example.com",
            "age": 25
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<User>::from_request(&req);
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.username, "john_doe");
        assert_eq!(user.email, "john@example.com");
        assert_eq!(user.age, 25);
    }

    #[test]
    fn test_invalid_email_fails_validation() {
        let json = json!({
            "username": "john_doe",
            "email": "invalid-email",
            "age": 25
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<User>::from_request(&req);
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert!(error.contains("email"));
    }

    #[test]
    fn test_username_too_short_fails_validation() {
        let json = json!({
            "username": "ab",
            "email": "john@example.com",
            "age": 25
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<User>::from_request(&req);
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert!(error.contains("username"));
    }

    #[test]
    fn test_username_too_long_fails_validation() {
        let json = json!({
            "username": "this_is_a_very_long_username_that_exceeds_twenty_chars",
            "email": "john@example.com",
            "age": 25
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<User>::from_request(&req);
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert!(error.contains("username"));
    }

    #[test]
    fn test_age_below_minimum_fails_validation() {
        let json = json!({
            "username": "john_doe",
            "email": "john@example.com",
            "age": 17
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<User>::from_request(&req);
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert!(error.contains("age"));
    }

    #[test]
    fn test_age_above_maximum_fails_validation() {
        let json = json!({
            "username": "john_doe",
            "email": "john@example.com",
            "age": 121
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<User>::from_request(&req);
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert!(error.contains("age"));
    }

    #[test]
    fn test_multiple_validation_errors() {
        let json = json!({
            "username": "ab",
            "email": "invalid-email",
            "age": 17
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<User>::from_request(&req);
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert!(error.contains("username") || error.contains("email") || error.contains("age"));
    }

    #[test]
    fn test_missing_required_field_fails_parsing() {
        let json = json!({
            "username": "john_doe",
            "email": "john@example.com"
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<User>::from_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_json_body_returns_error() {
        let req = create_text_request("not a json body");

        let result = JsonBodyValidated::<User>::from_request(&req);
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert_eq!(error, "Invalid request body");
    }

    #[test]
    fn test_malformed_json_fails_parsing() {
        let json = json!({
            "username": "john_doe",
            "email": "john@example.com",
            "age": "not_a_number"
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<User>::from_request(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_product_validation_passes() {
        let json = json!({
            "name": "Gaming Laptop",
            "price": 999.99,
            "description": "High-performance gaming laptop"
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<Product>::from_request(&req);
        assert!(result.is_ok());

        let product = result.unwrap();
        assert_eq!(product.name, "Gaming Laptop");
        assert_eq!(product.price, 999.99);
    }

    #[test]
    fn test_product_with_optional_field_none() {
        let json = json!({
            "name": "Mouse",
            "price": 29.99,
            "description": null
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<Product>::from_request(&req);
        assert!(result.is_ok());

        let product = result.unwrap();
        assert!(product.description.is_none());
    }

    #[test]
    fn test_product_price_zero_fails_validation() {
        let json = json!({
            "name": "Free Item",
            "price": 0.0,
            "description": "This should fail"
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<Product>::from_request(&req);
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert!(error.contains("price"));
    }

    #[test]
    fn test_product_negative_price_fails_validation() {
        let json = json!({
            "name": "Invalid Product",
            "price": -10.0,
            "description": "Negative price"
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<Product>::from_request(&req);
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert!(error.contains("price"));
    }

    #[test]
    fn test_product_empty_name_fails_validation() {
        let json = json!({
            "name": "",
            "price": 99.99,
            "description": "Valid description"
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<Product>::from_request(&req);
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert!(error.contains("name"));
    }

    #[test]
    fn test_login_request_validation_passes() {
        let json = json!({
            "email": "user@example.com",
            "password": "securepassword123"
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<LoginRequest>::from_request(&req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_login_password_too_short_fails_validation() {
        let json = json!({
            "email": "user@example.com",
            "password": "short"
        });
        let req = create_json_request(json);

        let result = JsonBodyValidated::<LoginRequest>::from_request(&req);
        assert!(result.is_err());

        let error = result.err().unwrap();
        assert!(error.contains("password"));
    }

    #[test]
    fn test_deref_access_to_validated_data() {
        let json = json!({
            "username": "jane_doe",
            "email": "jane@example.com",
            "age": 30
        });
        let req = create_json_request(json);

        let validated = JsonBodyValidated::<User>::from_request(&req).unwrap();

        assert_eq!(validated.username, "jane_doe");
        assert_eq!(validated.email, "jane@example.com");
        assert_eq!(validated.age, 30);
    }
}
