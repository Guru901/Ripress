#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        middlewares::{Middleware, MiddlewareType},
        req::HttpRequest,
        res::HttpResponse,
    };

    #[cfg(feature = "with-wynd")]
    use crate::helpers::exec_wynd_middleware;
    #[cfg(feature = "with-wynd")]
    use crate::middlewares::WyndMiddleware;
    #[cfg(feature = "with-wynd")]
    use bytes::Bytes;
    #[cfg(feature = "with-wynd")]
    use http_body_util::Full;
    #[cfg(feature = "with-wynd")]
    use hyper::{Request, body::Incoming};

    // Dummy middleware function that just passes through
    fn passthrough_middleware() -> Middleware {
        Middleware {
            path: "/".to_string(),
            func: Arc::new(|req: HttpRequest, _: HttpResponse| {
                Box::pin(async move { (req, None) })
            }),
            middleware_type: MiddlewareType::Pre,
        }
    }

    // Dummy middleware that short-circuits with a response
    fn blocking_middleware() -> Middleware {
        Middleware {
            path: "/block".to_string(),
            func: Arc::new(|req: HttpRequest, _res: HttpResponse| {
                Box::pin(async move {
                    let res = HttpResponse::new().ok().text("blocked!");
                    (req, Some(res))
                })
            }),
            middleware_type: MiddlewareType::Pre,
        }
    }

    // #[tokio::test]
    // async fn test_exec_pre_middleware_pass_through() {
    //     let req = make_request("/foo");
    //     let mw = passthrough_middleware();

    //     let res = exec_pre_middleware(req, mw).await;
    //     assert!(res.is_ok());
    //     let req = res.unwrap();
    //     assert_eq!(req.uri(), "/foo");
    // }

    // #[tokio::test]
    // async fn test_exec_pre_middleware_blocking() {
    //     let req = make_request("/block");
    //     let mw = blocking_middleware();

    //     let res = exec_pre_middleware(req, mw).await;
    //     assert!(res.is_err());

    //     match res {
    //         Err(ApiError::Generic(resp)) => {
    //             assert_eq!(resp.status_code.as_u16(), 200);
    //             // Optional: read body string here if needed
    //         }
    //         _ => panic!("Expected ApiError::Generic"),
    //     }
    // }

    #[cfg(feature = "with-wynd")]
    #[tokio::test]
    async fn test_exec_wynd_middleware_error_continues() {
        use crate::helpers::exec_wynd_middleware;

        let req = make_request("/wynd");

        let mw = WyndMiddleware {
            path: "/wynd".to_string(),
            func: Arc::new(|_req: Request<Incoming>| {
                Box::pin(async move {
                    // Instead of returning ApiError, return Ok with a response to match the expected type

                    use hyper::Response;
                    Ok(Response::builder()
                        .status(400)
                        .body(Full::new(Bytes::new()))
                        .unwrap())
                })
            }),
        };

        let res = exec_wynd_middleware(req, mw).await;
        assert!(!res.is_ok()); // request should continue
    }

    #[cfg(feature = "with-wynd")]
    #[tokio::test]
    async fn test_exec_wynd_middleware_success_blocks() {
        let req = make_request("/wynd");

        let mw = WyndMiddleware {
            path: "/wynd".to_string(),
            func: Arc::new(|_req: Request<Incoming>| {
                Box::pin(async move {
                    use hyper::Response;
                    Ok(Response::new(Full::new(Bytes::from("stopped"))))
                })
            }),
        };

        let res = exec_wynd_middleware(req, mw).await;
        assert!(res.is_err()); // should block
    }
}
