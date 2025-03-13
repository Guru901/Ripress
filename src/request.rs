use crate::types::{RequestBodyContent, RequestBodyType};
use futures_util::stream::StreamExt;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct RequestBody {
    content: RequestBodyContent,
    content_type: RequestBodyType,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    params: HashMap<String, String>,
    queries: HashMap<String, String>,
    body: RequestBody,
    ip: String,
    method: String,
    origin_url: String,
    path: String,
}

impl HttpRequest {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
            queries: HashMap::new(),
            body: RequestBody {
                content_type: RequestBodyType::TEXT,
                content: RequestBodyContent::TEXT(String::new()),
            },
            ip: String::new(),
            method: String::new(),
            origin_url: String::new(),
            path: String::new(),
        }
    }

    pub fn is(&self, content_type: RequestBodyType) -> bool {
        self.body.content_type == content_type
    }

    pub fn get_method(&self) -> String {
        self.method.to_string()
    }

    pub fn get_origin_url(&self) -> Option<String> {
        Some(self.origin_url.to_string())
    }

    pub fn get_path(&self) -> Option<String> {
        Some(self.path.to_string())
    }

    pub fn ip(&self) -> Option<String> {
        Some(self.ip.to_string())
    }

    pub fn get_params(&self, param_name: &str) -> Option<String> {
        self.params.get(param_name).map(|v| v.to_string())
    }

    pub fn get_query(&self, query_name: &str) -> Option<String> {
        self.queries.get(query_name).map(|v| v.to_string())
    }

    pub fn json<J>(&self) -> Result<J, String>
    where
        J: serde::de::DeserializeOwned + serde::Serialize,
    {
        let body = &self.body;

        if body.content_type == RequestBodyType::JSON {
            if let RequestBodyContent::JSON(ref json_value) = body.content {
                match serde_json::from_value::<J>(json_value.clone()) {
                    Ok(serialized) => Ok(serialized),
                    Err(e) => Err(format!("Failed to deserialize JSON: {}", e)),
                }
            } else {
                Err(String::from("Invalid JSON content"))
            }
        } else {
            Err(String::from("Wrong body type"))
        }
    }

    pub fn text(&self) -> Result<String, String> {
        let body = &self.body;

        if body.content_type == RequestBodyType::TEXT {
            if let RequestBodyContent::TEXT(ref text_value) = body.content {
                Ok(text_value.clone())
            } else {
                Err(String::from("Invalid text content"))
            }
        } else {
            Err(String::from("Wrong body type"))
        }
    }

    pub fn form_data(&self) -> Result<HashMap<String, String>, String> {
        let mut form_data: HashMap<String, String> = HashMap::new();
        let body = &self.body;

        if body.content_type == RequestBodyType::FORM {
            if let RequestBodyContent::FORM(ref text_value) = body.content {
                text_value.split("&").for_each(|pair| {
                    if let Some((key, value)) = pair.split_once("=") {
                        form_data.insert(key.to_string(), value.to_string());
                    }
                });
                Ok(form_data)
            } else {
                Err(String::from("Invalid form content"))
            }
        } else {
            Err(String::from("Wrong body type"))
        }
    }

    pub async fn from_actix_request(
        req: actix_web::HttpRequest,
        mut payload: actix_web::web::Payload,
    ) -> Result<Self, actix_web::Error> {
        // Extract all necessary data from the request early
        let mut queries = HashMap::new();
        let query_string = req.query_string();
        if !query_string.is_empty() {
            query_string.split("&").for_each(|pair| {
                if let Some((key, value)) = pair.split_once("=") {
                    queries.insert(key.to_string(), value.to_string());
                }
            });
        }

        let ip = get_real_ip(&req);
        let method = req.method().to_string();
        let origin_url = req.uri().to_string();
        let path = req.path().to_string();

        let params: HashMap<String, String> = req
            .match_info()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let content_type = determine_content_type(&req);

        // Read the body
        let mut body = actix_web::web::BytesMut::new();
        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            if (body.len() + chunk.len()) > 262_144 {
                return Err(actix_web::error::ErrorBadRequest("Body too large"));
            }
            body.extend_from_slice(&chunk);
        }

        let request_body = match content_type {
            RequestBodyType::FORM => {
                let body_string = match std::str::from_utf8(&body) {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        return Err(actix_web::error::ErrorBadRequest("Invalid UTF-8 sequence"));
                    }
                };

                RequestBody {
                    content: RequestBodyContent::FORM(body_string),
                    content_type: RequestBodyType::FORM,
                }
            }
            RequestBodyType::JSON => {
                let body_json = match std::str::from_utf8(&body) {
                    Ok(s) => match serde_json::from_str(s) {
                        Ok(json) => json,
                        Err(e) => {
                            return Err(actix_web::error::ErrorBadRequest(format!(
                                "Invalid JSON: {}",
                                e
                            )));
                        }
                    },
                    Err(_) => {
                        return Err(actix_web::error::ErrorBadRequest("Invalid UTF-8 sequence"));
                    }
                };

                RequestBody {
                    content: RequestBodyContent::JSON(body_json),
                    content_type: RequestBodyType::JSON,
                }
            }
            RequestBodyType::TEXT => {
                let body_string = match std::str::from_utf8(&body) {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        return Err(actix_web::error::ErrorBadRequest("Invalid UTF-8 sequence"));
                    }
                };

                RequestBody {
                    content: RequestBodyContent::TEXT(body_string),
                    content_type: RequestBodyType::TEXT,
                }
            }
        };

        Ok(HttpRequest {
            params,
            queries,
            body: request_body,
            ip,
            method,
            origin_url,
            path,
        })
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

#[cfg(test)]
impl HttpRequest {
    pub fn set_query(&mut self, key: &str, value: &str) {
        self.queries.insert(key.to_string(), value.to_string());
    }

    pub fn set_param(&mut self, key: &str, value: &str) {
        self.params.insert(key.to_string(), value.to_string());
    }

    pub fn set_json<J>(&mut self, json: J)
    where
        J: serde::de::DeserializeOwned + serde::Serialize,
    {
        self.body.content_type = RequestBodyType::JSON;
        self.body.content = RequestBodyContent::JSON(serde_json::to_value(json).unwrap());
    }

    pub fn set_text(&mut self, text: &str) {
        self.body.content_type = RequestBodyType::TEXT;
        self.body.content = RequestBodyContent::TEXT(text.to_string());
    }

    pub fn set_form(&mut self, key: &str, value: &str) {
        self.body.content_type = RequestBodyType::FORM;

        match &mut self.body.content {
            RequestBodyContent::FORM(existing) => {
                existing.push('&');
                existing.push_str(&format!("{key}={value}"));
            }
            _ => {
                self.body.content = RequestBodyContent::FORM(format!("{key}={value}"));
            }
        }
    }
}

fn get_real_ip(req: &actix_web::HttpRequest) -> String {
    req.headers()
        .get("X-Forwarded-For")
        .and_then(|val| val.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .unwrap_or_else(|| {
            req.peer_addr()
                .map(|addr| addr.ip().to_string())
                .unwrap_or("unknown".to_string())
        })
}
