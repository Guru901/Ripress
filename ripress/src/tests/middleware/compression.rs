#[cfg(test)]
mod test {
    #[cfg(feature = "compression")]
    use crate::context::HttpResponse;
    #[cfg(feature = "compression")]
    use crate::middlewares::compression::{
        CompressionConfig, accepts_gzip_encoding, compress_data, compression,
        get_response_body_bytes, set_response_body, should_compress_content_type,
    };
    #[cfg(feature = "compression")]
    use crate::req::HttpRequest;
    #[cfg(feature = "compression")]
    use crate::types::{ResponseContentBody, ResponseContentType};

    #[cfg(feature = "compression")]
    fn make_response_with_body(body: ResponseContentBody, content_type: &str) -> HttpResponse {
        let mut res = HttpResponse::new();
        res.body = body;
        res.content_type = match content_type {
            "text/plain" => ResponseContentType::TEXT,
            "application/json" => ResponseContentType::JSON,
            "text/html" => ResponseContentType::HTML,
            "application/javascript" => ResponseContentType::TEXT,
            "application/octet-stream" => ResponseContentType::BINARY,
            _ => ResponseContentType::TEXT,
        };
        res
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_should_compress_content_type() {
        assert!(should_compress_content_type("text/plain"));
        assert!(should_compress_content_type("text/html"));
        assert!(should_compress_content_type("application/json"));
        assert!(should_compress_content_type("application/javascript"));
        assert!(should_compress_content_type("application/xml"));
        assert!(should_compress_content_type("application/rss+xml"));
        assert!(should_compress_content_type("application/atom+xml"));
        assert!(should_compress_content_type("application/xhtml+xml"));
        assert!(should_compress_content_type("image/svg+xml"));
        assert!(!should_compress_content_type("image/png"));
        assert!(!should_compress_content_type("application/octet-stream"));
        assert!(!should_compress_content_type("video/mp4"));
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_compress_data_gzip_magic() {
        let data = b"hello world, hello world, hello world, hello world, hello world";
        let compressed = compress_data(data, 6).unwrap();
        // GZIP magic numbers
        assert_eq!(&compressed[0..2], &[0x1f, 0x8b]);
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_accepts_gzip_encoding() {
        assert!(accepts_gzip_encoding("gzip"));
        assert!(accepts_gzip_encoding("deflate, gzip"));
        assert!(accepts_gzip_encoding("gzip;q=1.0, identity; q=0.5, *;q=0"));
        assert!(accepts_gzip_encoding("gzip;q=0.8, deflate;q=0.5"));
        assert!(!accepts_gzip_encoding("deflate"));
        assert!(accepts_gzip_encoding("*;q=1.0"));
        assert!(!accepts_gzip_encoding("gzip;q=0"));
        assert!(!accepts_gzip_encoding("br"));
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_get_response_body_bytes() {
        let text = "hello";
        let json = serde_json::json!({"a": 1});
        let html = "<h1>hi</h1>";
        let bin = vec![1, 2, 3, 4];

        let mut res = HttpResponse::new();
        res.body = ResponseContentBody::TEXT(text.into());
        assert_eq!(
            get_response_body_bytes(&res),
            Some(text.as_bytes().to_vec())
        );

        res.body = ResponseContentBody::JSON(json.clone());
        assert_eq!(
            get_response_body_bytes(&res),
            serde_json::to_vec(&json).ok()
        );

        res.body = ResponseContentBody::HTML(html.into());
        assert_eq!(
            get_response_body_bytes(&res),
            Some(html.as_bytes().to_vec())
        );

        res.body = ResponseContentBody::BINARY(bin.clone().into());
        assert_eq!(get_response_body_bytes(&res), Some(bin));
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_set_response_body_sets_binary() {
        let mut res = HttpResponse::new();
        let compressed = vec![1, 2, 3, 4, 5];
        set_response_body(&mut res, compressed.clone()).unwrap();
        match &res.body {
            ResponseContentBody::BINARY(b) => assert_eq!(b.as_ref(), &compressed[..]),
            _ => panic!("Body should be BINARY"),
        }
    }

    #[cfg(feature = "compression")]
    #[tokio::test]
    async fn test_compression_middleware_compresses_when_appropriate() {
        let mw = compression(Some(CompressionConfig {
            threshold: 10,
            level: 6,
        }));

        let mut req = HttpRequest::new();
        req.headers
            .insert("Accept-Encoding".to_string(), "gzip".to_string());

        let body = "hello hello hello hello hello hello hello hello";
        let res = make_response_with_body(ResponseContentBody::TEXT(body.into()), "text/plain");

        let (_, res_opt) = mw(req, res).await;
        assert!(res_opt.is_some());
        let res = res_opt.unwrap();
        // Should be compressed
        match &res.body {
            ResponseContentBody::BINARY(b) => {
                // GZIP magic
                assert_eq!(&b[0..2], &[0x1f, 0x8b]);
            }
            _ => panic!("Body should be BINARY"),
        }
        // Should have Content-Encoding header
        assert_eq!(res.headers.get("Content-Encoding"), Some("gzip"));
        // Should have Vary header
        assert_eq!(res.headers.get("Vary"), Some("Accept-Encoding"));
    }

    #[cfg(feature = "compression")]
    #[tokio::test]
    async fn test_compression_middleware_skips_if_no_gzip_accept() {
        let mw = compression(Some(CompressionConfig {
            threshold: 10,
            level: 6,
        }));

        let req = HttpRequest::default();
        let body = "hello hello hello hello hello hello hello hello";
        let res = make_response_with_body(ResponseContentBody::TEXT(body.into()), "text/plain");

        let (_req_out, res_opt) = mw(req, res).await;
        assert!(res_opt.is_none());
    }

    #[cfg(feature = "compression")]
    #[tokio::test]
    async fn test_compression_middleware_skips_if_content_type_not_compressible() {
        let mw = compression(Some(CompressionConfig {
            threshold: 10,
            level: 6,
        }));

        let mut req = HttpRequest::default();
        req.headers
            .insert("Accept-Encoding".to_string(), "gzip".to_string());

        let bin = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let res = make_response_with_body(
            ResponseContentBody::BINARY(bin.into()),
            "application/octet-stream",
        );

        let (_req_out, res_opt) = mw(req, res).await;
        assert!(res_opt.is_none());
    }

    #[cfg(feature = "compression")]
    #[tokio::test]
    async fn test_compression_middleware_skips_if_body_too_small() {
        let mw = compression(Some(CompressionConfig {
            threshold: 100,
            level: 6,
        }));

        let mut req = HttpRequest::default();
        req.headers
            .insert("Accept-Encoding".to_string(), "gzip".to_string());

        let body = "short";
        let res = make_response_with_body(ResponseContentBody::TEXT(body.into()), "text/plain");

        let (_req_out, res_opt) = mw(req, res).await;
        assert!(res_opt.is_none());
    }

    #[cfg(feature = "compression")]
    #[tokio::test]
    async fn test_compression_middleware_skips_if_already_encoded() {
        let mw = compression(Some(CompressionConfig {
            threshold: 10,
            level: 6,
        }));

        let mut req = HttpRequest::default();
        req.headers
            .insert("Accept-Encoding".to_string(), "gzip".to_string());

        let body = "hello hello hello hello hello hello hello hello";
        let mut res = make_response_with_body(ResponseContentBody::TEXT(body.into()), "text/plain");
        res.headers
            .insert("Content-Encoding".to_string(), "gzip".to_string());

        let (_req_out, res_opt) = mw(req, res).await;
        assert!(res_opt.is_none());
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_compress_data() {
        let original = b"Hello, World! ".repeat(100);
        let compressed = compress_data(&original, 6).unwrap();

        // Compressed data should be smaller than original for repetitive content
        assert!(compressed.len() < original.len());

        // Should have gzip magic numbers at the beginning
        assert_eq!(&compressed[0..2], &[0x1f, 0x8b]);
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_compression_config_default() {
        let config = CompressionConfig::default();
        assert_eq!(config.threshold, 1024);
        assert_eq!(config.level, 6);
    }
}
