use crate::res::HttpResponse;
use std::convert::Infallible;

#[derive(Debug)]
pub(crate) enum ApiError {
    Generic(HttpResponse),
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
        // Log the internal error and return a generic 500
        eprintln!("hyper error: {}", err);

        ApiError::Generic(
            HttpResponse::new()
                .internal_server_error()
                .text(err.to_string()),
        )
    }
}

impl From<ApiError> for Box<dyn std::error::Error + Send> {
    fn from(error: ApiError) -> Self {
        Box::new(error)
    }
}

impl From<Box<dyn std::error::Error>> for ApiError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        // Log the internal error and return a generic 500
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
        }
    }
}
