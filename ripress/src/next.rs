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

#[derive(Clone, Default)]
pub struct Next;

impl Next {
    pub async fn call(
        &self,
        req: HttpRequest,
        res: HttpResponse,
    ) -> (HttpRequest, Option<HttpResponse>) {
        let cookies = res.cookies;
        let headers = res.headers;

        PENDING_HEADERS.try_with(|pending| {
            let mut pending = pending.borrow_mut();

            for (k, v) in headers.iter() {
                pending.push((k.to_string(), v.to_string()));
            }
        }).expect("Failed to access task-local storage for pending headers");
        PENDING_COOKIES.try_with(move |pending| {
            let mut pending = pending.borrow_mut();

            for cookie in cookies {
                pending.push(cookie);
            }
        }).expect("Failed to access task-local storage for pending cookies");
        (req, None)
    }
}
