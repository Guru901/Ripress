#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::{
        next::{Next, PENDING_COOKIES, PENDING_HEADERS},
        req::HttpRequest,
        res::HttpResponse,
    };

    /// Mirrors what `App::listen()` does around every request:
    ///  1. Opens task-local scopes for PENDING_HEADERS / PENDING_COOKIES.
    ///  2. Runs the provided closure (which plays the role of "middleware + handler").
    ///  3. Drains the pending stores into the final response before returning.
    async fn run_in_request_scope<F, Fut>(f: F) -> HttpResponse
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = HttpResponse>,
    {
        let fut = async move {
            let mut response = f().await;

            let _ = PENDING_HEADERS.try_with(|pending| {
                for (k, v) in pending.borrow_mut().drain(..) {
                    response = std::mem::take(&mut response).set_header(k, v);
                }
            });
            let _ = PENDING_COOKIES.try_with(|pending| {
                for cookie in pending.borrow_mut().drain(..) {
                    response = std::mem::take(&mut response).set_cookie_raw(cookie);
                }
            });

            response
        };

        PENDING_HEADERS
            .scope(
                RefCell::new(Vec::new()),
                PENDING_COOKIES.scope(RefCell::new(Vec::new()), fut),
            )
            .await
    }

    // ── Signal: next.call() must return None ─────────────────────────────────

    #[tokio::test]
    async fn test_next_call_returns_none_to_signal_continue() {
        PENDING_HEADERS
            .scope(
                RefCell::new(Vec::new()),
                PENDING_COOKIES.scope(
                    RefCell::new(Vec::new()),
                    async {
                        let res = HttpResponse::new().set_header("x-test", "value");
                        let (_req, maybe_res) =
                            Next::default().call(HttpRequest::new(), res).await;
                        assert!(
                            maybe_res.is_none(),
                            "next.call() must return None to signal request should continue"
                        );
                    },
                ),
            )
            .await;
    }

    // ── Header preservation ───────────────────────────────────────────────────

    #[tokio::test]
    async fn test_single_header_set_in_middleware_appears_in_final_response() {
        let response = run_in_request_scope(|| async {
            // Middleware: set a header, then call next (no short-circuit)
            let mw_res = HttpResponse::new().set_header("x-request-id", "abc-123");
            let (req, _) = Next::default().call(HttpRequest::new(), mw_res).await;
            drop(req);

            // Handler: returns a fresh response with no custom headers
            HttpResponse::new().ok().text("hello")
        })
        .await;

        assert_eq!(
            response.headers.get("x-request-id"),
            Some("abc-123"),
            "Header set by middleware should survive into the final response"
        );
    }

    #[tokio::test]
    async fn test_multiple_headers_set_in_middleware_all_appear_in_final_response() {
        let response = run_in_request_scope(|| async {
            let mw_res = HttpResponse::new()
                .set_header("x-request-id", "abc-123")
                .set_header("x-correlation-id", "xyz-789");
            let (req, _) = Next::default().call(HttpRequest::new(), mw_res).await;
            drop(req);

            HttpResponse::new().ok().text("hello")
        })
        .await;

        assert_eq!(response.headers.get("x-request-id"), Some("abc-123"));
        assert_eq!(response.headers.get("x-correlation-id"), Some("xyz-789"));
    }

    #[tokio::test]
    async fn test_middleware_header_and_handler_header_both_appear_in_final_response() {
        let response = run_in_request_scope(|| async {
            let mw_res = HttpResponse::new().set_header("x-from-middleware", "mw-value");
            let (req, _) = Next::default().call(HttpRequest::new(), mw_res).await;
            drop(req);

            // Handler sets its own distinct header
            HttpResponse::new()
                .ok()
                .set_header("x-from-handler", "handler-value")
                .text("hello")
        })
        .await;

        assert_eq!(
            response.headers.get("x-from-middleware"),
            Some("mw-value"),
            "Middleware header must be present"
        );
        assert_eq!(
            response.headers.get("x-from-handler"),
            Some("handler-value"),
            "Handler header must also be present"
        );
    }

    // ── Cookie preservation ───────────────────────────────────────────────────

    #[tokio::test]
    async fn test_single_cookie_set_in_middleware_appears_in_final_response() {
        let response = run_in_request_scope(|| async {
            let mw_res = HttpResponse::new().set_cookie("session", "tok123", None);
            let (req, _) = Next::default().call(HttpRequest::new(), mw_res).await;
            drop(req);

            HttpResponse::new().ok().text("hello")
        })
        .await;

        assert_eq!(
            response.get_cookie("session"),
            Some("tok123"),
            "Cookie set by middleware should survive into the final response"
        );
    }

    #[tokio::test]
    async fn test_multiple_cookies_set_in_middleware_all_appear_in_final_response() {
        let response = run_in_request_scope(|| async {
            let mw_res = HttpResponse::new()
                .set_cookie("session", "tok123", None)
                .set_cookie("csrf", "csrf_val", None);
            let (req, _) = Next::default().call(HttpRequest::new(), mw_res).await;
            drop(req);

            HttpResponse::new().ok().text("hello")
        })
        .await;

        assert_eq!(response.get_cookie("session"), Some("tok123"));
        assert_eq!(response.get_cookie("csrf"), Some("csrf_val"));
    }

    #[tokio::test]
    async fn test_middleware_cookie_and_handler_cookie_both_appear_in_final_response() {
        let response = run_in_request_scope(|| async {
            let mw_res = HttpResponse::new().set_cookie("mw-cookie", "mw-val", None);
            let (req, _) = Next::default().call(HttpRequest::new(), mw_res).await;
            drop(req);

            HttpResponse::new()
                .ok()
                .set_cookie("handler-cookie", "handler-val", None)
                .text("hello")
        })
        .await;

        assert_eq!(response.get_cookie("mw-cookie"), Some("mw-val"));
        assert_eq!(response.get_cookie("handler-cookie"), Some("handler-val"));
    }

    // ── Short-circuit: pending state must remain empty ────────────────────────

    #[tokio::test]
    async fn test_no_pending_injected_when_middleware_short_circuits() {
        // next.call() is deliberately NOT called — simulates a short-circuiting
        // middleware that returns Some(res) directly.
        let response = run_in_request_scope(|| async {
            HttpResponse::new().ok().text("handler response")
        })
        .await;

        assert!(
            response.headers.get("x-injected").is_none(),
            "No headers should be injected when next.call() was never called"
        );
        assert!(
            response.cookies.is_empty(),
            "No cookies should be injected when next.call() was never called"
        );
    }

    // ── Combined: headers + cookies together ──────────────────────────────────

    #[tokio::test]
    async fn test_headers_and_cookies_both_preserved_after_next_call() {
        let response = run_in_request_scope(|| async {
            let mw_res = HttpResponse::new()
                .set_header("x-trace-id", "trace-99")
                .set_cookie("auth", "bearer-token", None);
            let (req, _) = Next::default().call(HttpRequest::new(), mw_res).await;
            drop(req);

            HttpResponse::new().ok().text("ok")
        })
        .await;

        assert_eq!(response.headers.get("x-trace-id"), Some("trace-99"));
        assert_eq!(response.get_cookie("auth"), Some("bearer-token"));
    }
}