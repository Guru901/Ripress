use std::collections::HashMap;

use hyper::{Body, Request};

use crate::{app::Middleware, context::HttpResponse, request::HttpRequest, types::ApiError};

pub fn get_all_query_params(queries: &HashMap<String, String>) -> String {
    let mut query_params = String::new();

    queries.iter().for_each(|(key, value)| {
        query_params.push_str(&format!("{}={}&", key, value));
    });

    query_params
}

pub async fn exec_middleware(
    mut req: Request<Body>,
    middleware: Box<Middleware>,
) -> Result<Request<Body>, ApiError> {
    let mw_func = middleware.func;

    let our_res = HttpResponse::new();
    let mut our_req = HttpRequest::from_hyper_request(&mut req).await.unwrap();

    if our_req.get_path().starts_with(middleware.path.as_str()) {
        let (modified_req, maybe_res) = mw_func(&mut our_req, our_res).await;

        match maybe_res {
            None => {
                return Ok(modified_req.to_hyper_request().unwrap());
            }
            Some(res) => {
                let body: crate::types::ResponseContentBody = res.clone().get_body();
                let status_code = res.status_code;

                match body {
                    crate::types::ResponseContentBody::HTML(html) => {
                        return Err(ApiError::Generic(format!("{}", html), status_code));
                    }
                    crate::types::ResponseContentBody::JSON(json) => {
                        return Err(ApiError::Generic(format!("{}", json), status_code));
                    }
                    crate::types::ResponseContentBody::TEXT(text) => {
                        return Err(ApiError::Generic(format!("{}", text), status_code));
                    }
                }
            }
        }
    } else {
        Ok(our_req.to_hyper_request().unwrap())
    }
}
