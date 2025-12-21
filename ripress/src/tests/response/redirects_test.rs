#[cfg(test)]
mod response_redirects_tests {
    use crate::res::HttpResponse;

    #[test]
    fn test_redirect_basic() {
        let res = HttpResponse::new().redirect("/login");

        assert_eq!(res.status_code(), 302);
        assert_eq!(res.headers.get("location").unwrap(), "/login");
    }

    #[test]
    fn test_redirect_absolute_url() {
        let res = HttpResponse::new().redirect("https://example.com/page");

        assert_eq!(res.status_code(), 302);
        assert_eq!(
            res.headers.get("location").unwrap(),
            "https://example.com/page"
        );
    }

    #[test]
    fn test_permanent_redirect() {
        let res = HttpResponse::new().permanent_redirect("/new-location");

        assert_eq!(res.status_code(), 301);
        assert_eq!(res.headers.get("location").unwrap(), "/new-location");
    }

    #[test]
    fn test_redirect_with_query_params() {
        let res = HttpResponse::new().redirect("/search?q=rust&page=2");

        assert_eq!(res.status_code(), 302);
        assert_eq!(
            res.headers.get("location").unwrap(),
            "/search?q=rust&page=2"
        );
    }

    #[test]
    fn test_redirect_with_fragment() {
        let res = HttpResponse::new().redirect("/docs#section-1");

        assert_eq!(res.status_code(), 302);
        assert_eq!(res.headers.get("location").unwrap(), "/docs#section-1");
    }

    #[test]
    fn test_permanent_redirect_absolute_url() {
        let res = HttpResponse::new().permanent_redirect("https://new-domain.com/");

        assert_eq!(res.status_code(), 301);
        assert_eq!(
            res.headers.get("location").unwrap(),
            "https://new-domain.com/"
        );
    }

    #[test]
    fn test_redirect_root() {
        let res = HttpResponse::new().redirect("/");

        assert_eq!(res.status_code(), 302);
        assert_eq!(res.headers.get("location").unwrap(), "/");
    }

    #[test]
    fn test_redirect_relative_path() {
        let res = HttpResponse::new().redirect("../parent");

        assert_eq!(res.status_code(), 302);
        assert_eq!(res.headers.get("location").unwrap(), "../parent");
    }

    #[test]
    fn test_redirect_empty_path() {
        let res = HttpResponse::new().redirect("");

        assert_eq!(res.status_code(), 302);
        assert_eq!(res.headers.get("location").unwrap(), "");
    }
}
