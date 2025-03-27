#[tokio::test]
async fn test_stream_response() {
    use bytes::Bytes;
    use futures::StreamExt;

    let test_data = vec![1, 2, 3, 4, 5];
    let stream = stream::iter(test_data.clone()).map(|byte| Ok(Bytes::from(vec![byte])));

    let response = HttpResponse::new().write(stream);

    assert!(response.is_stream);

    // Collect the stream to verify contents
    let mut collected = Vec::new();
    let mut stream = response.stream;
    while let Some(Ok(bytes)) = stream.next().await {
        collected.extend_from_slice(&bytes);
    }

    assert_eq!(collected, test_data);
}
