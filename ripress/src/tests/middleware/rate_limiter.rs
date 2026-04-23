#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::time::sleep;

    use crate::{
        middlewares::rate_limiter::{rate_limiter, RateLimiterConfig},
        next::Next,
        req::{request_headers::RequestHeaders, HttpRequest},
        res::HttpResponse,
    };

    fn mock_req() -> HttpRequest {
        HttpRequest {
            headers: RequestHeaders::new(),
            ..Default::default()
        }
    }

    fn mock_res() -> HttpResponse {
        HttpResponse::new()
    }

    fn make_next() -> Next {
        Next {}
    }

    #[tokio::test]
    async fn allows_requests_within_limit() {
        let mw = rate_limiter(Some(RateLimiterConfig {
            max_requests: 3,
            window_ms: Duration::from_millis(1000),
            ..Default::default()
        }));

        let req = mock_req();
        let res = mock_res();
        let next = make_next();

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        assert!(resp.is_none());

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        assert!(resp.is_none());

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        assert!(resp.is_none());
    }

    #[tokio::test]
    async fn blocks_requests_over_limit() {
        let mw = rate_limiter(Some(RateLimiterConfig {
            max_requests: 2,
            window_ms: Duration::from_millis(1000),
            message: "Rate limit exceeded".to_string(),
            ..Default::default()
        }));

        let req = mock_req();
        let res = mock_res();
        let next = make_next();

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        assert!(resp.is_none());

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        assert!(resp.is_none());

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        assert!(resp.is_some());

        let resp = resp.unwrap();
        assert_eq!(
            resp.status_code,
            crate::res::response_status::StatusCode::TooManyRequests
        );

        assert_eq!(
            resp.headers.get("Retry-After").map(|v| v.to_string()),
            Some("0".to_string())
        );

        assert_eq!(
            resp.headers
                .get("X-RateLimit-Remaining")
                .map(|v| v.to_string()),
            Some("0".to_string())
        );
    }

    #[tokio::test]
    async fn resets_after_window() {
        let mw = rate_limiter(Some(RateLimiterConfig {
            max_requests: 1,
            window_ms: Duration::from_millis(100),
            ..Default::default()
        }));

        let req = mock_req();
        let res = mock_res();
        let next = make_next();

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        assert!(resp.is_none());

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        assert!(resp.is_some());

        sleep(Duration::from_millis(120)).await;

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        assert!(resp.is_none());
    }

    #[tokio::test]
    async fn uses_proxy_header_when_enabled() {
        let mw = rate_limiter(Some(RateLimiterConfig {
            max_requests: 1,
            window_ms: Duration::from_millis(1000),
            proxy: true,
            ..Default::default()
        }));

        let mut req = mock_req();
        req.headers.insert("X-Forwarded-For", "8.8.8.8");
        let res = mock_res();
        let next = make_next();

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        assert!(resp.is_none());

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        assert!(resp.is_some());
    }

    #[tokio::test]
    async fn sets_rate_limit_headers() {
        let mw = rate_limiter(Some(RateLimiterConfig {
            max_requests: 2,
            window_ms: Duration::from_millis(1000),
            ..Default::default()
        }));

        let req = mock_req();
        let res = mock_res();
        let next = make_next();

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        assert!(resp.is_none());

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        assert!(resp.is_none());

        let (_req, resp) = mw(req.clone(), res.clone(), next.clone()).await;
        let resp = resp.unwrap();
        assert_eq!(
            resp.headers.get("X-RateLimit-Limit").map(|v| v.to_string()),
            Some("2".to_string())
        );
        assert_eq!(
            resp.headers
                .get("X-RateLimit-Remaining")
                .map(|v| v.to_string()),
            Some("0".to_string())
        );
        assert!(resp.headers.get("X-RateLimit-Reset").is_some());
        assert!(resp.headers.get("Retry-After").is_some());
    }
}
