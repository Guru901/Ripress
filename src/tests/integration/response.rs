use super::spawn_test_server;
use reqwest::Client;

const PORT: u16 = 31002;
const BASE: &str = "http://127.0.0.1:31002";

#[tokio::test]
async fn response_headers_status_redirects_and_cors() {
    let handle = spawn_test_server(PORT).await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let client = Client::builder().build().unwrap();

    // Get Cookie
    let r = client
        .get(format!("{}/get-cookie-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let set_cookie = r.headers().get_all("set-cookie");
    let mut found = false;
    for v in set_cookie.iter() {
        if v.to_str().unwrap().contains("test-cookie=value") {
            found = true;
            break;
        }
    }
    assert!(found);

    // Set multiple cookies
    let r = client
        .get(format!("{}/multiple-cookies-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let cookies = r.headers().get_all("set-cookie");
    let s = cookies
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect::<Vec<_>>()
        .join(",");
    assert!(s.contains("session=abc123"));
    assert!(s.contains("theme=dark"));
    assert!(s.contains("lang=en"));

    // Cookie with options
    let r = client
        .get(format!("{}/cookie-options-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let s = r
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect::<Vec<_>>()
        .join(";");
    assert!(s.contains("secure-cookie=value"));
    assert!(s.to_lowercase().contains("httponly"));
    assert!(s.to_lowercase().contains("secure"));
    assert!(s.contains("SameSite=Strict") || s.contains("samesite=strict"));

    // Custom headers
    let r = client
        .get(format!("{}/custom-headers-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let h = r.headers();
    assert_eq!(h.get("x-custom-header").unwrap(), "custom-value");
    assert_eq!(h.get("x-api-version").unwrap(), "1.0");
    assert_eq!(h.get("x-powered-by").unwrap(), "Ripress");

    // Status 201
    let r = client
        .post(format!("{}/created-test", BASE))
        .json(&serde_json::json!({"name":"test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 201);
    let v: serde_json::Value = r.json().await.unwrap();
    assert_eq!(v["created"], true);

    // Custom status message (teapot)
    let r = client
        .get(format!("{}/custom-status-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 418);

    // Redirects
    let r = client
        .get(format!("{}/redirect-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200); // reqwest follows redirects by default

    let r = client
        .get(format!("{}/permanent-redirect-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);

    // No content response
    let r = client
        .delete(format!("{}/no-content-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 204);
    let txt = r.text().await.unwrap();
    assert_eq!(txt, "");

    // Multiple headers
    let r = client
        .get(format!("{}/multiple-headers-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let h = r.headers();
    assert_eq!(h.get("x-header-1").unwrap(), "value1");
    assert_eq!(h.get("x-header-2").unwrap(), "value2");
    assert_eq!(h.get("x-header-3").unwrap(), "value3");

    // CORS headers via OPTIONS fetch
    let r = client
        .request(reqwest::Method::OPTIONS, format!("{}/cors-test", BASE))
        .header("Origin", "https://example.com")
        .header("Access-Control-Request-Method", "POST")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let h = r.headers();
    assert_eq!(
        h.get("access-control-allow-origin").unwrap(),
        "https://example.com"
    );
    assert!(h.get("access-control-allow-methods").is_some());
    assert!(h.get("access-control-allow-headers").is_some());

    // Binary response
    let r = client
        .get(format!("{}/binary-test", BASE))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let ct = r
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.contains("application/octet-stream"));
    let bytes = r.bytes().await.unwrap();
    assert!(bytes.len() > 0);

    handle.abort();
}
