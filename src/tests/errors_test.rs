#[cfg(test)]
mod tests {
    use crate::{
        error::{RipressError, RipressErrorKind},
        req::{
            body::text_data::TextDataError, query_params::QueryParamError, route_params::ParamError,
        },
    };

    #[test]
    fn test_ripress_error_kind_display() {
        assert_eq!(RipressErrorKind::IO.to_string(), "IO error");
        assert_eq!(RipressErrorKind::ParseError.to_string(), "Parse error");
        assert_eq!(RipressErrorKind::InvalidInput.to_string(), "Invalid input");
        assert_eq!(RipressErrorKind::NotFound.to_string(), "Not found");
    }

    #[test]
    fn test_ripress_error_new_and_accessors() {
        let err = RipressError::new(RipressErrorKind::ParseError, "bad parse".to_string());
        assert_eq!(err.kind, RipressErrorKind::ParseError);
        assert_eq!(err.kind(), &RipressErrorKind::ParseError);
        assert_eq!(err.message, "bad parse");
        assert_eq!(err.message(), "bad parse");
    }

    #[test]
    fn test_ripress_error_display() {
        let err = RipressError::new(RipressErrorKind::InvalidInput, "Oops".into());
        let printed = format!("{}", err);
        assert!(
            printed.contains("Oops") && printed.contains("Invalid input"),
            "Display impl should have message and kind"
        );
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "disk fail");
        let err = RipressError::from(io_err);
        assert_eq!(err.kind, RipressErrorKind::IO);
        assert_eq!(err.message, "disk fail");
    }

    #[test]
    fn test_from_utf8_error() {
        // Invalid UTF-8: [0xff, 0xff] is not valid
        let bad = vec![0xff, 0xff];
        let result = String::from_utf8(bad.clone());
        if let Err(e) = result {
            let err = RipressError::from(e);
            assert_eq!(err.kind, RipressErrorKind::ParseError);
            assert!(
                err.message.contains("invalid utf-8"),
                "Should contain utf-8 error"
            );
        }
    }

    #[test]
    fn test_from_query_param_error_not_found() {
        let qpe = QueryParamError::NotFound("foo".into());
        let err = RipressError::from(qpe);
        assert_eq!(err.kind, RipressErrorKind::NotFound);
        assert_eq!(err.message, "Query Param 'foo' not found");
    }

    #[test]
    fn test_from_query_param_error_parse() {
        let qpe = QueryParamError::ParseError {
            param: "bar".to_string(),
            value: "baz".to_string(),
            target_type: "u32".to_string(),
        };
        let err = RipressError::from(qpe);
        assert_eq!(err.kind, RipressErrorKind::ParseError);
        assert!(
            err.message
                .contains("Failed to parse 'bar' from: u32 to:'baz'")
        );
    }

    #[test]
    fn test_from_param_error_not_found() {
        let pe = ParamError::NotFound("id".to_string());
        let err = RipressError::from(pe);
        assert_eq!(err.kind, RipressErrorKind::NotFound);
        assert_eq!(err.message, "Route Param 'id' not found");
    }

    #[test]
    fn test_from_param_error_parse() {
        let pe = ParamError::ParseError {
            param: "page".to_string(),
            value: "abc".to_string(),
            target_type: "u32".to_string(),
        };
        let err = RipressError::from(pe);
        assert_eq!(err.kind, RipressErrorKind::ParseError);
        assert!(
            err.message
                .contains("Failed to parse route param 'page' from: u32 to: 'abc'")
        );
    }

    #[test]
    fn test_from_text_data_error_invalid_utf8() {
        // build a dummy error
        use std::str::from_utf8;
        let bytes = &[0xff, 0xff];
        let utf8res = from_utf8(bytes);
        if let Err(e) = utf8res {
            let td = TextDataError::InvalidUtf8(e);
            let err = RipressError::from(td);
            assert_eq!(err.kind, RipressErrorKind::ParseError);
            assert!(err.message.contains("invalid utf-8"));
        }
    }

    #[test]
    fn test_from_text_data_error_too_large() {
        let td = TextDataError::TooLarge {
            size: 1234,
            limit: 999,
        };
        let err = RipressError::from(td);
        assert_eq!(err.kind, RipressErrorKind::InvalidInput);
        assert_eq!(err.message, "Text too large: 1234 bytes (limit: 999 bytes)");
    }
}
