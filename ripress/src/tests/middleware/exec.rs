#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        app::api_error::ApiError,
        helpers::exec_pre_middleware,
        middlewares::{Middleware, MiddlewareType},
        req::HttpRequest,
        res::HttpResponse,
    };

    #[cfg(feature = "with-wynd")]
    use crate::helpers::exec_wynd_middleware;
    #[cfg(feature = "with-wynd")]
    use crate::middlewares::WyndMiddleware;
    use bytes::Bytes;
    use http_body_util::Full;
    use hyper::Request;

    fn make_request(path: &str) -> Request<Full<Bytes>> {
        let full_req: Request<Full<Bytes>> = Request::builder()
            .uri(path)
            .body(Full::from(Bytes::new()))
            .unwrap();

        let (parts, _) = full_req.into_parts();
        let full_body: Full<Bytes> = Full::from(Bytes::new());
        let full_request = Request::from_parts(parts, full_body);

        let ptr = Box::into_raw(Box::new(full_request)) as *mut Request<Full<Bytes>>;
        unsafe { *Box::from_raw(ptr) }
    }
    fn passthrough_pre_middleware() -> Arc<Middleware> {
        Arc::new(Middleware {
            path: "/".to_string(),
            func: Arc::new(|req: HttpRequest, _: HttpResponse| {
                Box::pin(async move { (req, None) })
            }),
            middleware_type: MiddlewareType::Pre,
        })
    }

    fn blocking_pre_middleware() -> Arc<Middleware> {
        Arc::new(Middleware {
            path: "/block".to_string(),
            func: Arc::new(|req: HttpRequest, _res: HttpResponse| {
                Box::pin(async move {
                    let res = HttpResponse::new().ok().text("blocked!");
                    (req, Some(res))
                })
            }),
            middleware_type: MiddlewareType::Pre,
        })
    }

    #[tokio::test]
    async fn test_exec_pre_middleware_pass_through() {
        let req = make_request("/foo");
        let mw = passthrough_pre_middleware();

        let res = exec_pre_middleware(req, mw).await;
        assert!(res.is_ok());
        let req = res.unwrap();
        assert_eq!(req.uri(), "/foo");
    }

    #[tokio::test]
    async fn test_exec_pre_middleware_blocking() {
        let req = make_request("/block");
        let mw = blocking_pre_middleware();

        let res = exec_pre_middleware(req, mw).await;
        assert!(res.is_err());

        match res {
            Err(ApiError::Generic(resp)) => {
                assert_eq!(resp.status_code.as_u16(), 200);
            }
            _ => panic!("Expected ApiError::Generic"),
        }
    }

    #[cfg(feature = "with-wynd")]
    #[tokio::test]
    async fn test_exec_wynd_middleware_error_continues() {
        use crate::helpers::exec_wynd_middleware;

        let req = make_request("/wynd");

        let mw = Arc::new(WyndMiddleware {
            path: "/wynd".to_string(),
            func: Arc::new(|_req| {
                Box::pin(async move {
                    use hyper::Response;
                    Ok(Response::builder()
                        .status(400)
                        .body(Full::new(Bytes::new()))
                        .unwrap())
                })
            }),
        });

        let res = exec_wynd_middleware(req, mw).await;
        assert!(!res.is_ok());
    }

    #[cfg(feature = "with-wynd")]
    #[tokio::test]
    async fn test_exec_wynd_middleware_success_blocks() {
        let req = make_request("/wynd");

        let mw = Arc::new(WyndMiddleware {
            path: "/wynd".to_string(),
            func: Arc::new(|_req| {
                Box::pin(async move {
                    use hyper::Response;
                    Ok(Response::new(Full::new(Bytes::from("stopped"))))
                })
            }),
        });

        let res = exec_wynd_middleware(req, mw).await;
        assert!(res.is_err());
    }
}
