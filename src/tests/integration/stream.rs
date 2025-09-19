use super::spawn_test_server;

const PORT: u16 = 31003;
const BASE: &str = "http://127.0.0.1:31003";

#[tokio::test]
async fn stream_text_and_json() {
    let handle = spawn_test_server(PORT).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let r = reqwest::get(format!("{}/stream-text", BASE)).await.unwrap();
    assert_eq!(r.status(), 200);
    let headers = r.headers();
    assert_eq!(headers.get("transfer-encoding").unwrap(), "chunked");
    let ct = headers
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(
        ct.contains("text/event-stream")
            || ct.contains("text/plain")
            || ct.contains("application/octet-stream")
    );
    let text = r.text().await.unwrap();
    assert!(text.contains("chunk1"));
    assert!(text.contains("chunk2"));
    assert!(text.contains("chunk3"));

    let r = reqwest::get(format!("{}/stream-json", BASE)).await.unwrap();
    assert_eq!(r.status(), 200);
    let headers = r.headers();
    assert_eq!(headers.get("transfer-encoding").unwrap(), "chunked");
    let ct = headers
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(
        ct.contains("text/event-stream")
            || ct.contains("application/json")
            || ct.contains("application/octet-stream")
    );
    let text = r.text().await.unwrap();
    assert!(text.contains("{\"id\":1"));
    assert!(text.contains("{\"id\":2"));
    assert!(text.contains("{\"id\":3"));

    handle.abort();
}
