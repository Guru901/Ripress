#![warn(missing_docs)]
use http_body_util::Full;
use hyper::body::Bytes;

use crate::res::HttpResponse;
use std::convert::Infallible;

#[derive(Debug)]
pub enum ApiError {
    Generic(HttpResponse),
    WebSocketUpgrade(hyper::Response<Full<Bytes>>),
}

unsafe impl Sync for ApiError {}

impl From<HttpResponse> for ApiError {
    fn from(res: HttpResponse) -> Self {
        ApiError::Generic(res)
    }
}

impl From<Infallible> for ApiError {
    fn from(_: Infallible) -> Self {
        ApiError::Generic(
            HttpResponse::new()
                .internal_server_error()
                .text("Unhandled error"),
        )
    }
}

impl From<hyper::Error> for ApiError {
    fn from(err: hyper::Error) -> Self {
        let message = err.to_string();

        let status = if err.is_user() {
            400
        } else if err.is_canceled() {
            504
        } else {
            500
        };

        eprintln!("hyper error: {}", err);

        ApiError::Generic(HttpResponse::new().status(status).text(message))
    }
}

impl From<ApiError> for Box<dyn std::error::Error + Send> {
    fn from(error: ApiError) -> Self {
        Box::new(error)
    }
}

impl From<Box<dyn std::error::Error>> for ApiError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        eprintln!("internal error: {}", error);

        ApiError::Generic(
            HttpResponse::new()
                .internal_server_error()
                .text(error.to_string()),
        )
    }
}

impl std::error::Error for ApiError {}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApiError::Generic(msg) => {
                write!(f, "Middleware Error: {:?}", msg)
            }
            ApiError::WebSocketUpgrade(response) => {
                write!(f, "WebSocket upgrade error: {:?}", response)
            }
        }
    }
}
