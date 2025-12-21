#[cfg(test)]
mod response_cookies_tests {
    use crate::res::response_cookie::{CookieOptions, CookieSameSiteOptions};
    use crate::res::HttpResponse;

    #[tokio::test]
    async fn test_set_cookie_basic() {
        let res = HttpResponse::new().set_cookie("session", "abc123", None);
        let res = res.to_hyper_response().await.unwrap();

        let headers = res.headers();
        let cookies: Vec<&str> = headers
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect();
        assert_eq!(cookies.len(), 1);
    }

    #[tokio::test]
    async fn test_set_cookie_with_default_options() {
        let res = HttpResponse::new().set_cookie("token", "xyz789", Some(CookieOptions::default()));
        let res = res.to_hyper_response().await.unwrap();

        let headers = res.headers();
        let cookie_str = headers.get("set-cookie").unwrap().to_str().unwrap();

        assert!(cookie_str.contains("token=xyz789"));
        assert!(cookie_str.contains("HttpOnly"));
        assert!(cookie_str.contains("Secure"));
        assert!(cookie_str.contains("SameSite=None"));
        assert!(cookie_str.contains("Path=/"));
    }

    #[tokio::test]
    async fn test_set_cookie_with_custom_path() {
        let options = CookieOptions {
            path: Some("/api"),
            ..Default::default()
        };

        let res = HttpResponse::new().set_cookie("api_token", "token123", Some(options));
        let res = res.to_hyper_response().await.unwrap();

        let headers = res.headers();
        let cookie_str = headers.get("set-cookie").unwrap().to_str().unwrap();

        assert!(cookie_str.contains("Path=/api"));
    }

    #[tokio::test]
    async fn test_set_cookie_with_domain() {
        let options = CookieOptions {
            domain: Some("example.com"),
            ..Default::default()
        };

        let res = HttpResponse::new().set_cookie("user", "john", Some(options));
        let res = res.to_hyper_response().await.unwrap();

        let headers = res.headers();
        let cookie_str = headers.get("set-cookie").unwrap().to_str().unwrap();

        assert!(cookie_str.contains("Domain=example.com"));
    }

    #[tokio::test]
    async fn test_set_cookie_with_max_age() {
        let options = CookieOptions {
            max_age: Some(3600), // 1 hour
            ..Default::default()
        };

        let res = HttpResponse::new().set_cookie("temp", "data", Some(options));
        let res = res.to_hyper_response().await.unwrap();

        let cookie_str = res.headers().get("set-cookie").unwrap().to_str().unwrap();

        assert!(cookie_str.contains("Max-Age=3600"));
    }

    #[tokio::test]
    async fn test_set_cookie_with_expires() {
        let options = CookieOptions {
            expires: Some(1735689600), // Some future timestamp
            ..Default::default()
        };

        let res = HttpResponse::new().set_cookie("temp", "data", Some(options));
        let res = res.to_hyper_response().await.unwrap();

        let cookie_str = res.headers().get("set-cookie").unwrap().to_str().unwrap();

        assert!(cookie_str.contains("Expires="));
    }

    #[tokio::test]
    async fn test_set_cookie_samesite_strict() {
        let options = CookieOptions {
            same_site: CookieSameSiteOptions::Strict,
            ..Default::default()
        };

        let res = HttpResponse::new().set_cookie("temp", "data", Some(options));
        let res = res.to_hyper_response().await.unwrap();

        let cookie_str = res.headers().get("set-cookie").unwrap().to_str().unwrap();

        assert!(cookie_str.contains("SameSite=Strict"));
    }

    #[tokio::test]
    async fn test_set_cookie_samesite_lax() {
        let options = CookieOptions {
            same_site: CookieSameSiteOptions::Lax,
            ..Default::default()
        };

        let res = HttpResponse::new().set_cookie("temp", "data", Some(options));
        let res = res.to_hyper_response().await.unwrap();

        let cookie_str = res.headers().get("set-cookie").unwrap().to_str().unwrap();

        assert!(cookie_str.contains("SameSite=Lax"));
    }

    #[tokio::test]
    async fn test_set_cookie_not_secure() {
        let options = CookieOptions {
            secure: false,
            ..Default::default()
        };

        let res = HttpResponse::new().set_cookie("temp", "data", Some(options));
        let res = res.to_hyper_response().await.unwrap();

        let cookie_str = res.headers().get("set-cookie").unwrap().to_str().unwrap();

        assert!(!cookie_str.contains("Secure"));
    }

    #[tokio::test]
    async fn test_set_cookie_not_http_only() {
        let options = CookieOptions {
            http_only: false,
            ..Default::default()
        };

        let res = HttpResponse::new().set_cookie("temp", "data", Some(options));
        let res = res.to_hyper_response().await.unwrap();

        let cookie_str = res.headers().get("set-cookie").unwrap().to_str().unwrap();

        assert!(!cookie_str.contains("HttpOnly"));
    }

    #[tokio::test]
    async fn test_set_multiple_cookies() {
        let res = HttpResponse::new()
            .set_cookie("cookie1", "value1", None)
            .set_cookie("cookie2", "value2", None)
            .set_cookie("cookie3", "value3", None);

        let res = res.to_hyper_response().await.unwrap();

        let cookie_str: Vec<String> = res
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .collect();

        assert!(cookie_str.len() == 3);
    }

    #[tokio::test]
    async fn test_clear_cookie_basic() {
        let res = HttpResponse::new().clear_cookie("session");

        let res = res.to_hyper_response().await.unwrap();

        let headers = res.headers();
        let cookie_str = headers.get("set-cookie").unwrap().to_str().unwrap();

        assert!(cookie_str.contains("session="));
        assert!(cookie_str.contains("Max-Age=0"));
    }

    #[tokio::test]
    async fn test_set_and_clear_cookie_chain() {
        let res = HttpResponse::new()
            .set_cookie("temp", "data", None)
            .clear_cookie("old");

        let res = res.to_hyper_response().await.unwrap();

        let headers = res.headers();
        let cookies: Vec<&str> = headers
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect();

        assert_eq!(cookies.len(), 2);
    }

    #[tokio::test]
    async fn test_cookie_with_all_options() {
        let options = CookieOptions {
            http_only: true,
            secure: true,
            same_site: CookieSameSiteOptions::Strict,
            path: Some("/admin"),
            domain: Some("example.com"),
            max_age: Some(7200),
            expires: None,
        };

        let res = HttpResponse::new().set_cookie("admin_session", "secure_token", Some(options));
        let res = res.to_hyper_response().await.unwrap();

        let headers = res.headers();
        let cookie_str = headers.get("set-cookie").unwrap().to_str().unwrap();

        assert!(cookie_str.contains("admin_session=secure_token"));
        assert!(cookie_str.contains("HttpOnly"));
        assert!(cookie_str.contains("Secure"));
        assert!(cookie_str.contains("SameSite=Strict"));
        assert!(cookie_str.contains("Path=/admin"));
        assert!(cookie_str.contains("Domain=example.com"));
        assert!(cookie_str.contains("Max-Age=7200"));
    }

    #[tokio::test]
    async fn test_cookie_special_characters_in_value() {
        let res = HttpResponse::new().set_cookie("data", "hello world!", None);
        let res = res.to_hyper_response().await.unwrap();

        let headers = res.headers();
        let cookie_str = headers.get("set-cookie").unwrap().to_str().unwrap();

        assert!(cookie_str.contains("data="));
    }
}
