use crate::{
    app::{Middleware, api_error::ApiError},
    req::{HttpRequest, query_params::QueryParams},
    res::HttpResponse,
};
use hyper::{Body, Request};
use url::form_urlencoded::Serializer;

pub(crate) async fn exec_middleware(
    mut req: Request<Body>,
    middleware: Middleware,
) -> Result<Request<Body>, ApiError> {
    let mw_func = middleware.func;

    let our_res = HttpResponse::new();
    let our_req = HttpRequest::from_hyper_request(&mut req)
        .await
        .map_err(ApiError::from)?;

    if path_matches(middleware.path.as_str(), our_req.path.as_str()) {
        let (modified_req, maybe_res) = mw_func(our_req, our_res).await;

        match maybe_res {
            None => {
                return modified_req.to_hyper_request().map_err(ApiError::from);
            }
            Some(res) => {
                return Err(ApiError::Generic(res));
            }
        }
    } else {
        our_req.to_hyper_request().map_err(ApiError::from)
    }
}

pub(crate) fn path_matches(prefix: &str, path: &str) -> bool {
    let is_slash = prefix == "/" || prefix.ends_with('/');
    if is_slash {
        path == prefix
            || path.starts_with(&(prefix.to_string()))
            || path.starts_with(&(prefix.to_string() + "/"))
    } else {
        path == prefix || path.starts_with(&(prefix.to_string() + "/"))
    }
}

pub(crate) fn get_all_query(queries: &QueryParams) -> String {
    let mut ser = Serializer::new(String::new());
    for (k, v) in queries.iter() {
        ser.append_pair(k, v);
    }
    ser.finish()
}
