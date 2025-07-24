use crate::req::HttpRequest;
use crate::res::HttpResponse;
use serde::Serialize;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RequestBody {
    pub content: RequestBodyContent,
    pub content_type: RequestBodyType,
}

impl RequestBody {
    pub fn new_text<T: Into<String>>(text: T) -> Self {
        RequestBody {
            content_type: RequestBodyType::TEXT,
            content: RequestBodyContent::TEXT(text.into()),
        }
    }

    pub fn new_form<T: Into<String>>(form_data: T) -> Self {
        RequestBody {
            content_type: RequestBodyType::FORM,
            content: RequestBodyContent::FORM(form_data.into()),
        }
    }

    pub fn new_json<T: Into<serde_json::Value>>(json: T) -> Self {
        RequestBody {
            content_type: RequestBodyType::JSON,
            content: RequestBodyContent::JSON(json.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestBodyType {
    JSON,
    TEXT,
    FORM,
}

impl Copy for RequestBodyType {}

#[derive(Debug, Clone)]
pub enum RequestBodyContent {
    TEXT(String),
    JSON(serde_json::Value),
    FORM(String),
}

pub enum ResponseContentBody {
    TEXT(String),
    HTML(String),
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
    pub fn new_html<T: Into<String>>(html: T) -> Self {
        ResponseContentBody::HTML(html.into())
    }
}
pub enum ResponseContentType {
    TEXT,
    JSON,
    HTML,
}

pub type Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>;

pub type Handler = Arc<dyn Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static>;

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    HEAD,
    DELETE,
    PATCH,
}

pub type Routes = HashMap<String, (HttpMethod, Handler)>;

#[derive(Debug, PartialEq)]
pub enum HttpRequestError {
    MissingCookie(String),
    MissingParam(String),
    MissingHeader(String),
    MissingQuery(String),
}

impl std::fmt::Display for HttpRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpRequestError::MissingCookie(cookie) => write!(f, "Cookie {} doesn't exist", cookie),
            HttpRequestError::MissingParam(param) => write!(f, "Param {} doesn't exist", param),
            HttpRequestError::MissingHeader(header) => write!(f, "Header {} doesn't exist", header),
            HttpRequestError::MissingQuery(query) => write!(f, "Query {} doesn't exist", query),
        }
    }
}
