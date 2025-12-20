#[cfg(test)]
mod test {
    use hyper::Method;

    use crate::{req::HttpRequest, res::HttpResponse, types::HttpMethods};

    #[test]
    fn test_httpmethods_display() {
        assert_eq!(HttpMethods::GET.to_string(), "GET");
        assert_eq!(HttpMethods::POST.to_string(), "POST");
        assert_eq!(HttpMethods::PUT.to_string(), "PUT");
        assert_eq!(HttpMethods::DELETE.to_string(), "DELETE");
        assert_eq!(HttpMethods::PATCH.to_string(), "PATCH");
        assert_eq!(HttpMethods::OPTIONS.to_string(), "OPTIONS");
        assert_eq!(HttpMethods::HEAD.to_string(), "HEAD");
    }

    #[test]
    fn test_httpmethods_from() {
        let method = HttpMethods::from(&Method::GET);
        assert_eq!(method, HttpMethods::GET);

        let method = HttpMethods::from(&Method::POST);
        assert_eq!(method, HttpMethods::POST);

        let method = HttpMethods::from(&Method::PUT);
        assert_eq!(method, HttpMethods::PUT);

        let method = HttpMethods::from(&Method::DELETE);
        assert_eq!(method, HttpMethods::DELETE);

        let method = HttpMethods::from(&Method::PATCH);
        assert_eq!(method, HttpMethods::PATCH);

        let method = HttpMethods::from(&Method::OPTIONS);
        assert_eq!(method, HttpMethods::OPTIONS);

        let method = HttpMethods::from(&Method::HEAD);
        assert_eq!(method, HttpMethods::HEAD);

        let method = HttpMethods::from(&Method::CONNECT);
        assert_eq!(method, HttpMethods::GET);

        let method = HttpMethods::from(&Method::TRACE);
        assert_eq!(method, HttpMethods::GET);
    }

    #[test]
    fn test_status_code_helpers() {
        let response = HttpResponse::new().accepted();
        assert_eq!(response.status_code.as_u16(), 202);
        assert_eq!(response.status_code.canonical_reason(), "Accepted");

        let response = HttpResponse::new().no_content();
        assert_eq!(response.status_code.as_u16(), 204);
        assert_eq!(response.status_code.canonical_reason(), "No Content");

        let response = HttpResponse::new().forbidden();
        assert_eq!(response.status_code.as_u16(), 403);
        assert_eq!(response.status_code.canonical_reason(), "Forbidden");

        let response = HttpResponse::new().method_not_allowed();
        assert_eq!(response.status_code.as_u16(), 405);
        assert_eq!(
            response.status_code.canonical_reason(),
            "Method Not Allowed"
        );

        let response = HttpResponse::new().conflict();
        assert_eq!(response.status_code.as_u16(), 409);
        assert_eq!(response.status_code.canonical_reason(), "Conflict");

        let response = HttpResponse::new().not_implemented();
        assert_eq!(response.status_code.as_u16(), 501);
        assert_eq!(response.status_code.canonical_reason(), "Not Implemented");

        let response = HttpResponse::new().bad_gateway();
        assert_eq!(response.status_code.as_u16(), 502);
        assert_eq!(response.status_code.canonical_reason(), "Bad Gateway");

        let response = HttpResponse::new().service_unavailable();
        assert_eq!(response.status_code.as_u16(), 503);
        assert_eq!(
            response.status_code.canonical_reason(),
            "Service Unavailable"
        );
    }

    #[test]
    fn test_get_method() {
        let mut req = HttpRequest::new();

        req.set_method(HttpMethods::GET);
        assert_eq!(req.method, HttpMethods::GET);

        req.set_method(HttpMethods::POST);
        assert_eq!(req.method, HttpMethods::POST);

        req.set_method(HttpMethods::PUT);
        assert_eq!(req.method, HttpMethods::PUT);

        req.set_method(HttpMethods::DELETE);
        assert_eq!(req.method, HttpMethods::DELETE);

        req.set_method(HttpMethods::OPTIONS);
        assert_eq!(req.method, HttpMethods::OPTIONS);

        req.set_method(HttpMethods::DELETE);
        assert_ne!(req.method, HttpMethods::GET);
    }
}
