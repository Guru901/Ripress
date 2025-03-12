use futures_util::stream::StreamExt;
use std::{collections::HashMap, future::Future};

#[derive(Debug, Clone, PartialEq)]
enum RequestBodyType {
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

#[derive(Debug, Clone)]
struct RequestBody {
    content: RequestBodyContent,
    content_type: RequestBodyType,
}

#[derive(Debug)]
pub struct HttpRequest {
    params: HashMap<String, String>,
    queries: HashMap<String, String>,
    body: RequestBody,
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
        }
    }

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

    pub fn json<J>(&self) -> Result<J, String>
    where
        J: serde::de::DeserializeOwned + serde::Serialize,
    {
        let body = self.body.clone();

        if body.content_type == RequestBodyType::JSON {
            if let RequestBodyContent::JSON(json_value) = body.content {
                match serde_json::from_value::<J>(json_value) {
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
        let body = self.body.clone();

        if body.content_type == RequestBodyType::TEXT {
            if let RequestBodyContent::TEXT(text_value) = body.content {
                Ok(text_value)
            } else {
                Err(String::from("Invalid JSON content"))
            }
        } else {
            Err(String::from("Wrong body type"))
        }
    }

    pub fn form_data(&self) -> Result<HashMap<String, String>, String> {
        let mut form_data: HashMap<String, String> = HashMap::new();
        let body = self.body.clone();

        if body.content_type == RequestBodyType::FORM {
            if let RequestBodyContent::FORM(text_value) = body.content {
                text_value.split("&").for_each(|pair| {
                    if let Some((key, value)) = pair.split_once("=") {
                        form_data.insert(key.to_string(), value.to_string());
                    }
                });
                Ok(form_data)
            } else {
                Err(String::from("Invalid JSON content"))
            }
        } else {
            Err(String::from("Wrong body type"))
        }
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

            let content_type = determine_content_type(&req);

            match content_type {
                RequestBodyType::FORM => {
                    let body_string = match std::str::from_utf8(&body) {
                        Ok(s) => s.to_string(),
                        Err(_) => {
                            return Err(actix_web::error::ErrorBadRequest("Invalid UTF-8 sequence"))
                        }
                    };
                    return Ok(HttpRequest {
                        params: req
                            .match_info()
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect(),
                        queries,
                        body: RequestBody {
                            content: RequestBodyContent::FORM(body_string),
                            content_type: RequestBodyType::FORM,
                        },
                    });
                }
                RequestBodyType::JSON => {
                    let body_json = match std::str::from_utf8(&body) {
                        Ok(s) => serde_json::from_str(s),
                        Err(_) => {
                            return Err(actix_web::error::ErrorBadRequest("Invalid UTF-8 sequence"))
                        }
                    };
                    return Ok(HttpRequest {
                        params: req
                            .match_info()
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect(),
                        queries,
                        body: RequestBody {
                            content: RequestBodyContent::JSON(body_json.unwrap()),
                            content_type: RequestBodyType::JSON,
                        },
                    });
                }
                RequestBodyType::TEXT => {
                    let body_string = match std::str::from_utf8(&body) {
                        Ok(s) => s.to_string(),
                        Err(_) => {
                            return Err(actix_web::error::ErrorBadRequest("Invalid UTF-8 sequence"))
                        }
                    };

                    return Ok(HttpRequest {
                        params: req
                            .match_info()
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect(),
                        queries,
                        body: RequestBody {
                            content: RequestBodyContent::TEXT(body_string),
                            content_type: RequestBodyType::TEXT,
                        },
                    });
                }
            }
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
            } else {
                return RequestBodyType::TEXT;
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
