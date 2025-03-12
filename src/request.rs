use futures_util::stream::StreamExt;
use std::{collections::HashMap, future::Future};

#[derive(Debug, Clone)]
enum RequestBodyType {
    JSON,
    TEXT,
    FORM,
}

impl Copy for RequestBodyType {}

#[derive(Debug)]
struct RequestBody {
    content: String,
    content_type: RequestBodyType,
}

#[derive(Debug)]
pub struct HttpRequest {
    params: HashMap<String, String>,
    queries: HashMap<String, String>,
    body: RequestBody,
}

impl HttpRequest {
    pub fn get_params(&self, param_name: &str) -> Option<String> {
        let param = self.params.get(param_name).map(|v| v.to_string());

        if let Some(param) = param {
            return Some(param);
        }

        return None;
    }

    pub fn get_query(&self, query_name: &str) -> Option<String> {
        let query = self.queries.get(query_name).map(|v| v.to_string());

        if let Some(query) = query {
            return Some(query);
        }

        return None;
    }

    pub fn get_body(&self) -> Option<String> {
        let body = self.body.content.clone();
        Some(body)
    }

    pub fn from_actix_request<'a>(
        req: &'a actix_web::HttpRequest,
        payload: &'a mut actix_web::web::Payload,
    ) -> impl Future<Output = Result<Self, actix_web::Error>> + use<'a> {
        let mut queries = HashMap::new();
        if let Some(query_string) = req.query_string().split("?").next() {
            query_string.split("&").for_each(|pair| {
                if let Some((key, value)) = pair.split_once("=") {
                    queries.insert(key.to_string(), value.to_string());
                }
            });
        };

        async move {
            let mut body = actix_web::web::BytesMut::new();

            while let Some(chunk) = payload.next().await {
                let chunk = chunk?;
                if (body.len() + chunk.len()) > 262_144 {
                    return Err(actix_web::error::ErrorBadRequest("Body too large"));
                }
                body.extend_from_slice(&chunk);
            }

            let body_string = match std::str::from_utf8(&body) {
                Ok(s) => s.to_string(),
                Err(_) => return Err(actix_web::error::ErrorBadRequest("Invalid UTF-8 sequence")),
            };

            return Ok(HttpRequest {
                params: req
                    .match_info()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                queries,
                body: RequestBody {
                    content: body_string,
                    content_type: RequestBodyType::TEXT,
                },
            });
        }
    }
}

fn determine_content_type(req: &actix_web::HttpRequest) -> RequestBodyType {
    if let Some(content_type) = req.headers().get("content-type") {
        if let Ok(content_type_str) = content_type.to_str() {
            if content_type_str.contains("application/json") {
                return RequestBodyType::JSON;
            } else if content_type_str.contains("application/x-www-form-urlencoded") {
                return RequestBodyType::FORM;
            }
        }
    }
    RequestBodyType::TEXT
}
