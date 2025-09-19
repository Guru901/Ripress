use super::spawn_test_server;
use reqwest::Client;
use serde_json::Value;

const PORT: u16 = 31001;
const BASE: &str = "http://127.0.0.1:31001";

#[tokio::test]
async fn request_cookies_headers_queries_and_bodies() {
    let handle = spawn_test_server(PORT).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let client = Client::builder().build().unwrap();

    // Set and check cookies
    let r = client
        .get(format!("{}/cookie-test", BASE))
        .header("Cookie", "sessionId=abc123")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["sessionId"], "abc123");

    // Set and check headers
    let r = client
        .get(format!("{}/header-test", BASE))
        .header("Test-Header", "test-value")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["header"], "test-value");

    // Params and query
    let r = client
        .get(format!(
            "{}/param-and-query-test/test?query=test-query",
            BASE
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["param"], "test");
    assert_eq!(body["query"], "test-query");

    // origin_url and path
    let r = client
        .get(format!("{}/origin-url-and-path/test?q=test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    let origin = body["originUrl"].as_str().unwrap();
    assert!(origin == "http://127.0.0.1:31001" || origin == "http://localhost:31001");
    assert_eq!(body["path"], "/origin-url-and-path/test");

    // IP
    let r = client
        .get(format!("{}/ip-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["ip"], "127.0.0.1");

    // JSON body
    let r = client
        .post(format!("{}/json-test", BASE))
        .json(&serde_json::json!({"name":"test","age":123}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["name"], "test");
    assert_eq!(body["age"], 123);

    // text body
    let r = client
        .post(format!("{}/text-test", BASE))
        .header("Content-Type", "text/plain")
        .body("test")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body = r.text().await.unwrap();
    assert_eq!(body, "test");

    // form data (urlencoded)
    let r = client
        .post(format!("{}/form-test", BASE))
        .form(&[("name", "test")])
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["name"], "test");

    // Auth ok
    let r = client
        .get(format!("{}/auth", BASE))
        .header("Cookie", "token=123abc")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.text().await.unwrap(), "123abc");

    // Auth unauthorized
    let r = client.get(format!("{}/auth", BASE)).send().await.unwrap();
    assert_eq!(r.status(), 401);
    assert_eq!(r.text().await.unwrap(), "unauthorized");

    // Multiple query parameters
    let r = client
        .get(format!("{}/multi-query?name=john&age=25&city=NYC", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["name"], "john");
    assert_eq!(body["age"], "25");
    assert_eq!(body["city"], "NYC");

    // Multiple route parameters
    let r = client
        .get(format!("{}/users/123/posts/456", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["userId"], "123");
    assert_eq!(body["postId"], "456");

    // Multiple cookies
    let r = client
        .get(format!("{}/multi-cookies", BASE))
        .header("Cookie", "user=john; theme=dark; lang=en")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["user"], "john");
    assert_eq!(body["theme"], "dark");
    assert_eq!(body["lang"], "en");

    // Multiple headers
    let r = client
        .get(format!("{}/multi-headers", BASE))
        .header("User-Agent", "TestBot/1.0")
        .header("Accept", "application/json")
        .header("Authorization", "Bearer token123")
        .header("X-Custom-Header", "custom-value")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["userAgent"], "TestBot/1.0");
    assert_eq!(body["accept"], "application/json");
    assert_eq!(body["authorization"], "Bearer token123");
    assert_eq!(body["customHeader"], "custom-value");

    // Request method detection
    let r = client
        .get(format!("{}/method-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body, "GET");

    let r = client
        .post(format!("{}/method-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body, "POST");

    let r = client
        .put(format!("{}/method-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body, "PUT");

    let r = client
        .delete(format!("{}/method-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body, "DELETE");

    // URL-encoded form data
    let r = client
        .post(format!("{}/urlencoded-test", BASE))
        .form(&[
            ("username", "testuser"),
            ("password", "secret123"),
            ("email", "test@example.com"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["username"], "testuser");
    assert_eq!(body["password"], "secret123");
    assert_eq!(body["email"], "test@example.com");

    // Raw body data
    let r = client
        .post(format!("{}/raw-body-test", BASE))
        .header("Content-Type", "text/plain")
        .body("raw text content")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["rawBody"], "raw text content");
    assert_eq!(body["contentType"], "text/plain");

    // Request secure/insecure
    let r = client
        .get(format!("{}/secure-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body, false);

    // xhr detection
    let r = client
        .get(format!("{}/xhr-test", BASE))
        .header("X-Requested-With", "XMLHttpRequest")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["xhr"], true);

    // Empty request body
    let r = client
        .post(format!("{}/empty-body-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body.as_object().unwrap().len(), 0);

    // Invalid JSON in request body
    let r = client
        .post(format!("{}/json-error-test", BASE))
        .header("Content-Type", "application/json")
        .body("{\"invalid\": json}")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 400);

    // Extremely long header value
    let r = client
        .get(format!("{}/long-header-test", BASE))
        .header("X-Long-Header", "x".repeat(8192))
        .send()
        .await
        .unwrap();
    assert!([200, 431].contains(&(r.status().as_u16())));

    // No Content-Type but JSON payload
    let r = client
        .post(format!("{}/no-content-type-test", BASE))
        .body(serde_json::to_string(&serde_json::json!({"test":"data"})).unwrap())
        .send()
        .await
        .unwrap();
    assert!([200, 400, 415].contains(&r.status().as_u16()));

    // Conflicting Content-Type and body
    let r = client
        .post(format!("{}/content-type-mismatch-test", BASE))
        .header("Content-Type", "text/plain")
        .body("{\"test\":\"object\"}")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);

    // Special characters in query
    let r = client
        .get(format!(
            "{}/special-query-test?name=John%20Doe&symbols=%21%40%23%24&unicode=%F0%9F%8C%9F",
            BASE
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["name"], "John Doe");
    assert_eq!(body["symbols"], "!@#$");
    assert_eq!(body["unicode"], "🌟");

    // Large body
    let large = serde_json::json!({
        "data": "x".repeat(1024 * 1024),
        "numbers": (0..1000).collect::<Vec<i32>>()
    });
    let r = client
        .post(format!("{}/large-body-test", BASE))
        .json(&large)
        .send()
        .await
        .unwrap();
    assert!([200, 413].contains(&r.status().as_u16()));

    handle.abort();
}
