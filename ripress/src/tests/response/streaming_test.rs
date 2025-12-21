#[cfg(test)]
mod response_streaming_tests {
    use crate::res::HttpResponse;
    use bytes::Bytes;
    use futures::stream;

    #[tokio::test]
    async fn test_stream_basic() {
        let data = vec![
            Ok::<Bytes, std::io::Error>(Bytes::from("chunk1")),
            Ok(Bytes::from("chunk2")),
            Ok(Bytes::from("chunk3")),
        ];
        let stream = stream::iter(data);

        let res = HttpResponse::new().ok().write(stream);

        assert!(res.is_stream);
        assert_eq!(res.headers.get("transfer-encoding").unwrap(), "chunked");
        assert_eq!(res.headers.get("cache-control").unwrap(), "no-cache");
    }

    #[tokio::test]
    async fn test_stream_single_chunk() {
        let data = vec![Ok::<Bytes, std::io::Error>(Bytes::from("single chunk"))];
        let stream = stream::iter(data);

        let res = HttpResponse::new().write(stream);

        assert!(res.is_stream);
    }

    #[tokio::test]
    async fn test_stream_empty() {
        let data: Vec<Result<Bytes, std::io::Error>> = vec![];
        let stream = stream::iter(data);

        let res = HttpResponse::new().write(stream);

        assert!(res.is_stream);
    }

    #[tokio::test]
    async fn test_stream_with_status() {
        let data = vec![Ok::<Bytes, std::io::Error>(Bytes::from("data"))];
        let stream = stream::iter(data);

        let res = HttpResponse::new().status(201).write(stream);

        assert!(res.is_stream);
        assert_eq!(res.status_code(), 201);
    }

    #[tokio::test]
    async fn test_stream_with_custom_headers() {
        let data = vec![Ok::<Bytes, std::io::Error>(Bytes::from("data"))];
        let stream = stream::iter(data);

        let res = HttpResponse::new()
            .set_header("x-custom", "value")
            .write(stream);

        assert!(res.is_stream);
        assert_eq!(res.headers.get("x-custom").unwrap(), "value");
    }

    #[tokio::test]
    async fn test_sse_event_format() {
        let events = vec![
            Ok::<Bytes, std::io::Error>(Bytes::from("data: event1\n\n")),
            Ok(Bytes::from("data: event2\n\n")),
            Ok(Bytes::from("data: event3\n\n")),
        ];
        let stream = stream::iter(events);

        let res = HttpResponse::new()
            .set_header("content-type", "text/event-stream")
            .set_header("connection", "keep-alive")
            .write(stream);

        assert!(res.is_stream);
        assert_eq!(
            res.headers.get("content-type").unwrap(),
            "text/event-stream"
        );
        assert_eq!(res.headers.get("connection").unwrap(), "keep-alive");
    }

    #[tokio::test]
    async fn test_stream_large_chunks() {
        let large_data = "x".repeat(10000);
        let data = vec![
            Ok::<Bytes, std::io::Error>(Bytes::from(large_data.clone())),
            Ok(Bytes::from(large_data.clone())),
        ];
        let stream = stream::iter(data);

        let res = HttpResponse::new().write(stream);

        assert!(res.is_stream);
    }

    #[tokio::test]
    async fn test_stream_json_chunks() {
        let data = vec![
            Ok::<Bytes, std::io::Error>(Bytes::from(r#"{"id":1,"name":"Alice"}"#)),
            Ok(Bytes::from(r#"{"id":2,"name":"Bob"}"#)),
        ];
        let stream = stream::iter(data);

        let res = HttpResponse::new()
            .set_header("content-type", "application/x-ndjson")
            .write(stream);

        assert!(res.is_stream);
    }
}
