#![warn(missing_docs)]
use crate::{
    context::HttpResponse, req::HttpRequest, res::response_status::StatusCode, types::FutMiddleware,
};

/// Middleware for limiting the maximum allowed size of the HTTP request body.
///
/// This middleware checks the length of the incoming request body and rejects requests
/// whose body exceeds the configured limit. If the body is too large, it returns a
/// `413 Payload Too Large` response with a JSON error message.
///
/// # Arguments
///
/// * `config` - An optional maximum size in bytes for the request body. If `None` is provided,
///   the default limit is 1 MB (1,048,576 bytes).
///
/// # Returns
///
/// Returns a middleware function that can be used in the middleware chain. The function
/// takes an [`HttpRequest`] and [`HttpResponse`], and returns a future resolving to a tuple
/// of the request and an optional response. If the body is within the limit, the request
/// proceeds as normal. If the body exceeds the limit, a 413 error response is returned.
///
/// # Example
///
/// ```rust
/// use ripress::app::App;
///
/// // Limit request bodies to 2 MB
/// let mut app = App::new();
/// app.use_body_limit(Some(2 * 1024 * 1024)); // 2 MB
/// ```
///
/// # Error Response
///
/// If the body is too large, the response will be:
///
/// ```json
/// {
///   "error": "Request body too large",
///   "message": "Request body exceeded the configured limit of 1048576 bytes",
///   "limit": 1048576,
///   "received": 2097152
/// }
/// ```
const DEFAULT_BODY_LIMIT: usize = 1024 * 1024; 

pub(crate) fn body_limit(
    config: Option<usize>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static {
    let config = config.unwrap_or(DEFAULT_BODY_LIMIT); 
    move |req: HttpRequest, res| {
        Box::pin(async move {
            let body = req.clone().body.content;

            if body.len() > config {
                eprintln!(
                    "Body limit exceeded: {} bytes > {} bytes",
                    body.len(),
                    config
                );

                return (req, Some(res.status(StatusCode::PayloadTooLarge.as_u16()).json(serde_json::json!({
                    "error": "Request body too large",
                    "message": format!("Request body exceeded the configured limit of {} bytes", config),
                    "limit": config,
                    "received": body.len()
                }))));
            }

            (req, None)
        })
    }
}
