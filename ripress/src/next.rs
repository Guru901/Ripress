use crate::{req::HttpRequest, res::HttpResponse};

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
        return (req, Some(res));
    }
}
