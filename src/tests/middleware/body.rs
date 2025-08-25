#[cfg(test)]
mod test {
    use crate::{
        middlewares::body_limit::body_limit,
        req::{
            HttpRequest,
            body::{RequestBody, RequestBodyContent},
        },
        res::{HttpResponse, response_status::StatusCode},
    };

    const DEFAULT_BODY_LIMIT: usize = 1024 * 1024; // 1 MB

    fn make_req_with_body(body: Vec<u8>) -> HttpRequest {
        HttpRequest {
            body: RequestBody::new_binary(body),
            ..Default::default()
        }
    }

    fn make_res() -> HttpResponse {
        HttpResponse::default()
    }

    #[tokio::test]
    async fn test_body_within_limit() {
        let limit = 10;
        let body = vec![1, 2, 3, 4, 5];
        let req = make_req_with_body(body.clone());
        let res = make_res();

        let middleware = body_limit(Some(limit));
        let (req_out, resp_opt) = middleware(req, res).await;

        assert_eq!(
            req_out.body.content,
            RequestBodyContent::BINARY(body.into())
        );
        assert!(resp_opt.is_none());
    }

    #[tokio::test]
    async fn test_body_exceeds_limit() {
        let limit = 5;
        let body = vec![1, 2, 3, 4, 5, 6, 7];
        let req = make_req_with_body(body.clone());
        let res = make_res();

        let middleware = body_limit(Some(limit));
        let (req_out, resp_opt) = middleware(req, res).await;

        assert_eq!(
            req_out.body.content,
            RequestBodyContent::BINARY(body.into())
        );
        assert!(resp_opt.is_some());
        let resp = resp_opt.unwrap();
        assert_eq!(resp.status_code, StatusCode::PayloadTooLarge);
    }

    #[tokio::test]
    async fn test_default_limit() {
        let default_limit = DEFAULT_BODY_LIMIT;
        let body = vec![0u8; default_limit + 1];
        let req = make_req_with_body(body.clone());
        let res = make_res();

        let middleware = body_limit(None);
        let (req_out, resp_opt) = middleware(req, res).await;

        assert_eq!(req_out.body.content.len(), default_limit + 1);
        assert!(resp_opt.is_some());
        let resp = resp_opt.unwrap();
        assert_eq!(resp.status_code, StatusCode::PayloadTooLarge);
    }

    #[tokio::test]
    async fn test_body_exactly_at_limit() {
        let limit = 8;
        let body = vec![1u8; limit];
        let req = make_req_with_body(body.clone());
        let res = make_res();

        let middleware = body_limit(Some(limit));
        let (req_out, resp_opt) = middleware(req, res).await;

        assert_eq!(
            req_out.body.content,
            RequestBodyContent::BINARY(body.into())
        );
        assert!(resp_opt.is_none());
    }
}
