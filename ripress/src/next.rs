use crate::{
    req::HttpRequest,
    res::{response_cookie::Cookie, HttpResponse},
};
use std::cell::RefCell;

tokio::task_local! {
    // Stores (header_name, header_value) pairs set by middleware via next.call()
    pub(crate) static PENDING_HEADERS: RefCell<Vec<(String, String)>>;
    // Stores Set-Cookie strings set by middleware via next.call()
    pub(crate) static PENDING_COOKIES: RefCell<Vec<Cookie>>;
}

#[derive(Clone)]
pub struct Next {}

impl Next {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn call(
        &self,
        req: HttpRequest,
        res: HttpResponse,
    ) -> (HttpRequest, Option<HttpResponse>) {
        let cookies = res.cookies;
        let headers = res.headers;

        let _ = PENDING_HEADERS.try_with(|pending| {
            let mut pending = pending.borrow_mut();

            for (k, v) in headers.iter() {
                pending.push((k.to_string(), v.to_string()));
            }
        });
        let _ = PENDING_COOKIES.try_with(move |pending| {
            let mut pending = pending.borrow_mut();

            for cookie in cookies {
                pending.push(cookie);
            }
        });
        (req, None)
    }
}
