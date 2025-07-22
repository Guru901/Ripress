use crate::req::HttpRequest;
use crate::res::HttpResponse;
use serde::Serialize;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

pub enum ResponseContentBody {
    TEXT(String),
    JSON(serde_json::Value),
}

impl ResponseContentBody {
    pub fn new_text<T: Into<String>>(text: T) -> Self {
        ResponseContentBody::TEXT(text.into())
    }

    pub fn new_json<T: Serialize>(json: T) -> Self {
        let value = serde_json::to_value(json).expect("Failed to serialize to JSON");
        ResponseContentBody::JSON(value)
    }
}
pub enum ResponseContentType {
    TEXT,
    JSON,
}

pub type Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>;
pub type Handler = Arc<dyn Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static>;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    HEAD,
}
pub type Routes = HashMap<HttpMethod, (String, Handler)>;
