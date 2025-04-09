use crate::{context::HttpResponse, request::HttpRequest};
use serde::Serialize;
use std::{
    collections::HashMap,
    fmt::{self, Display},
    future::Future,
    pin::Pin,
    sync::Arc,
};

// HttpRequest types

#[derive(Debug, Clone, PartialEq)]
pub enum RequestBodyType {
    JSON,
    TEXT,
    FORM,
    EMPTY,
}

impl Copy for RequestBodyType {}

#[derive(Debug, Clone)]
pub enum RequestBodyContent {
    TEXT(String),
    JSON(serde_json::Value),
    FORM(String),
    EMPTY,
}

#[derive(Debug, PartialEq)]
pub enum HttpRequestError {
    MissingCookie(String),
    MissingParam(String),
    MissingHeader(String),
    MissingQuery(String),
    InvalidJson(String),
}

impl std::fmt::Display for HttpRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpRequestError::MissingCookie(cookie) => write!(f, "Cookie {} doesn't exist", cookie),
            HttpRequestError::MissingParam(param) => write!(f, "Param {} doesn't exist", param),
            HttpRequestError::MissingHeader(header) => write!(f, "Header {} doesn't exist", header),
            HttpRequestError::MissingQuery(query) => write!(f, "Query {} doesn't exist", query),
            HttpRequestError::InvalidJson(err) => write!(f, "Invalid json: {}", err),
        }
    }
}

// HttpResponse types

#[derive(PartialEq, Debug, Clone)]
pub enum ResponseContentType {
    JSON,
    TEXT,
    HTML,
}

#[derive(Serialize, PartialEq, Debug, Clone)]
pub(crate) enum ResponseContentBody {
    JSON(serde_json::Value),
    TEXT(String),
    HTML(String),
}

impl ResponseContentBody {
    pub fn new_text<T: Into<String>>(text: T) -> Self {
        ResponseContentBody::TEXT(text.into())
    }

    pub fn new_html<T: Into<String>>(text: T) -> Self {
        ResponseContentBody::HTML(text.into())
    }
}

#[derive(Debug, PartialEq)]
pub enum HttpResponseError {
    MissingHeader(String),
}

impl std::fmt::Display for HttpResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpResponseError::MissingHeader(header) => write!(f, "Header {} doesnt exist", header),
        }
    }
}

// App types

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum HttpMethods {
    GET,
    PUT,
    POST,
    DELETE,
    PATCH,
    HEAD,
}

impl Display for HttpMethods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let method = match self {
            HttpMethods::GET => "GET",
            HttpMethods::PUT => "PUT",
            HttpMethods::POST => "POST",
            HttpMethods::DELETE => "DELETE",
            HttpMethods::PATCH => "PATCH",
            HttpMethods::HEAD => "HEAD",
        };
        write!(f, "{}", method)
    }
}

pub type Fut = Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>;
pub type FutMiddleware =
    Pin<Box<dyn Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static>>;
pub type Handler = Arc<dyn Fn(&mut HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static>;
pub type HandlerMiddleware =
    Arc<dyn Fn(&mut HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static>;
pub(crate) type Routes = HashMap<&'static str, HashMap<HttpMethods, Handler>>;

pub trait Middleware: Send + Sync + 'static {
    fn handle(
        &self,
        req: &mut HttpRequest,
        res: HttpResponse,
        next: Next,
    ) -> Pin<Box<dyn Future<Output = (HttpRequest, Option<HttpResponse>)> + Send + 'static>>;

    // Add this method to allow cloning of Box<dyn Middleware>
    fn clone_box(&self) -> Box<dyn Middleware>;
}

// Implement Clone for Box<dyn Middleware>
impl Clone for Box<dyn Middleware> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub struct Next {
    pub middleware: Vec<Box<dyn Middleware>>,
    pub handler: HandlerMiddleware,
}

impl Next {
    pub fn new() -> Self {
        Next {
            middleware: Vec::new(),
            handler: Arc::new(|_, _| Box::pin(async { (HttpRequest::new(), None) })),
        }
    }
    pub async fn run(
        self,
        req: &mut HttpRequest,
        res: HttpResponse,
    ) -> (HttpRequest, Option<HttpResponse>) {
        if let Some((current, rest)) = self.middleware.split_first() {
            // Call the next middleware
            let next = Next {
                middleware: rest.to_vec(),
                handler: self.handler.clone(),
            };
            current.handle(req, res, next).await
        } else {
            // No more middleware, call the handler
            (self.handler)(req, res).await
        }
    }
}
#[derive(Debug)]
pub enum ApiError {
    Generic(String, u16),
}

impl std::error::Error for ApiError {}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::Generic(string, status_code) => {
                write!(f, "Generic: {} {}", status_code, string)
            }
        }
    }
}
