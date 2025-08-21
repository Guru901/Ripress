#[cfg(test)]
mod tests {
    use crate::res::response_status::StatusCode;

    #[test]
    fn test_canonical_reason_standard() {
        assert_eq!(StatusCode::Ok.canonical_reason(), "OK");
        assert_eq!(StatusCode::Created.canonical_reason(), "Created");
        assert_eq!(StatusCode::Accepted.canonical_reason(), "Accepted");
        assert_eq!(StatusCode::NoContent.canonical_reason(), "No Content");
        assert_eq!(StatusCode::Redirect.canonical_reason(), "Found");
        assert_eq!(
            StatusCode::PermanentRedirect.canonical_reason(),
            "Moved Permanently"
        );
        assert_eq!(StatusCode::BadRequest.canonical_reason(), "Bad Request");
        assert_eq!(StatusCode::Unauthorized.canonical_reason(), "Unauthorized");
        assert_eq!(StatusCode::Forbidden.canonical_reason(), "Forbidden");
        assert_eq!(StatusCode::NotFound.canonical_reason(), "Not Found");
        assert_eq!(
            StatusCode::MethodNotAllowed.canonical_reason(),
            "Method Not Allowed"
        );
        assert_eq!(StatusCode::Conflict.canonical_reason(), "Conflict");
        assert_eq!(
            StatusCode::TooManyRequests.canonical_reason(),
            "Too Many Requests"
        );
        assert_eq!(
            StatusCode::InternalServerError.canonical_reason(),
            "Internal Server Error"
        );
        assert_eq!(
            StatusCode::NotImplemented.canonical_reason(),
            "Not Implemented"
        );
        assert_eq!(StatusCode::BadGateway.canonical_reason(), "Bad Gateway");
        assert_eq!(
            StatusCode::ServiceUnavailable.canonical_reason(),
            "Service Unavailable"
        );
        let custom = StatusCode::Custom(599);
        assert_eq!(custom.canonical_reason(), "Custom");
    }

    #[test]
    fn test_standard_status_codes_fmt() {
        let cases = vec![
            (StatusCode::Ok, "200 OK"),
            (StatusCode::Created, "201 Created"),
            (StatusCode::Accepted, "202 Accepted"),
            (StatusCode::NoContent, "204 No Content"),
            (StatusCode::Redirect, "302 Found"),
            (StatusCode::PermanentRedirect, "301 Moved Permanently"),
            (StatusCode::BadRequest, "400 Bad Request"),
            (StatusCode::Unauthorized, "401 Unauthorized"),
            (StatusCode::Forbidden, "403 Forbidden"),
            (StatusCode::NotFound, "404 Not Found"),
            (StatusCode::MethodNotAllowed, "405 Method Not Allowed"),
            (StatusCode::Conflict, "409 Conflict"),
            (StatusCode::TooManyRequests, "429 Too Many Requests"),
            (StatusCode::InternalServerError, "500 Internal Server Error"),
            (StatusCode::NotImplemented, "501 Not Implemented"),
            (StatusCode::BadGateway, "502 Bad Gateway"),
            (StatusCode::ServiceUnavailable, "503 Service Unavailable"),
        ];

        for (status, expected) in cases {
            assert_eq!(format!("{}", status), expected);
        }
    }

    #[test]
    fn test_custom_status_code_fmt() {
        let custom = StatusCode::Custom(499);
        assert_eq!(format!("{}", custom), "499 Custom");

        let another_custom = StatusCode::Custom(600);
        assert_eq!(format!("{}", another_custom), "600 Custom");
    }
    #[test]
    fn test_standard_status_codes_from_u16() {
        let cases = vec![
            (200, StatusCode::Ok),
            (201, StatusCode::Created),
            (202, StatusCode::Accepted),
            (204, StatusCode::NoContent),
            (302, StatusCode::Redirect),
            (301, StatusCode::PermanentRedirect),
            (400, StatusCode::BadRequest),
            (401, StatusCode::Unauthorized),
            (403, StatusCode::Forbidden),
            (404, StatusCode::NotFound),
            (405, StatusCode::MethodNotAllowed),
            (409, StatusCode::Conflict),
            (429, StatusCode::TooManyRequests),
            (500, StatusCode::InternalServerError),
            (501, StatusCode::NotImplemented),
            (502, StatusCode::BadGateway),
            (503, StatusCode::ServiceUnavailable),
        ];

        for (code, expected) in cases {
            assert_eq!(StatusCode::from_u16(code), expected, "failed for {}", code);
        }
    }

    #[test]
    fn test_custom_status_codes_from_u16() {
        let custom_codes = [199, 299, 450, 600, 999];

        for &code in &custom_codes {
            match StatusCode::from_u16(code) {
                StatusCode::Custom(inner) => assert_eq!(inner, code),
                other => panic!("expected Custom({}), got {:?}", code, other),
            }
        }
    }

    #[test]
    fn payload_too_large_roundtrip() {
        assert_eq!(StatusCode::PayloadTooLarge.as_u16(), 413);
        assert_eq!(StatusCode::from_u16(413), StatusCode::PayloadTooLarge);
    }

    #[test]
    fn payload_too_large_texts() {
        assert_eq!(
            StatusCode::PayloadTooLarge.canonical_reason(),
            "Payload Too Large"
        );
        assert_eq!(
            format!("{}", StatusCode::PayloadTooLarge),
            "413 Payload Too Large"
        );
    }
}
