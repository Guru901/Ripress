use crate::types::{HttpMethod, RequestBody, RequestBodyContent, RequestBodyType};
use actix_web::HttpMessage;
use futures::StreamExt;
use std::collections::HashMap;
pub struct HttpRequest {
    params: HashMap<String, String>,
    query_params: HashMap<String, String>,
    pub origin_url: String,
    pub method: HttpMethod,
    pub ip: String,
    pub path: String,
    pub protocol: String,
    pub is_secure: bool,
    pub headers: HashMap<String, String>,
    pub cookies: HashMap<String, String>,
    data: HashMap<String, String>,
    body: RequestBody,
}

impl HttpRequest {
    pub fn new() -> Self {
        HttpRequest {
            origin_url: String::new(),
            params: HashMap::new(),
            query_params: HashMap::new(),
            method: HttpMethod::GET,
            ip: String::new(),
            path: String::new(),
            protocol: String::new(),
            is_secure: false,
            headers: HashMap::new(),
            data: HashMap::new(),
            body: RequestBody::new_text(""),
            cookies: HashMap::new(),
        }
    }

    pub fn get_param(&self, param_name: &str) -> Option<&str> {
        match self.params.get(param_name) {
            Some(param) => Some(param.as_str()),
            None => None,
        }
    }

    pub fn get_query_params(&self, query_param_name: &str) -> Option<&str> {
        match self.query_params.get(query_param_name) {
            Some(query_param) => Some(query_param.as_str()),
            None => None,
        }
    }

    pub fn set_data<T: Into<String>>(&mut self, data_key: T, data_value: T) {
        self.data.insert(data_key.into(), data_value.into());
    }

    pub fn get_data<T: Into<String>>(&mut self, data_key: T) -> Option<&String> {
        self.data.get(&data_key.into())
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
                serde_urlencoded::from_str::<HashMap<String, String>>(text_value)
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .for_each(|(k, v)| {
                        form_data.insert(k, v);
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
        let origin_url = req.full_url().to_string();

        let params: HashMap<String, String> = req
            .match_info()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let method = req.method().as_str();

        let method = match method {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "HEAD" => HttpMethod::HEAD,
            "PUT" => HttpMethod::PUT,
            _ => HttpMethod::GET,
        };

        let path = req.path().to_string();

        let ip = req
            .headers()
            .get("X-Forwarded-For")
            .and_then(|val| val.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .unwrap_or_else(|| {
                req.peer_addr()
                    .map(|addr| addr.ip().to_string())
                    .unwrap_or("unknown".to_string())
            });
        let protocol = req.connection_info().scheme().to_string();
        let is_secure;

        if protocol == "https" {
            is_secure = true
        } else {
            is_secure = false
        }

        let mut headers = HashMap::new();

        req.headers().iter().for_each(|f| {
            let header_name = f.0.to_string();
            let header_value = f.1.to_str().unwrap().to_string();
            headers.insert(header_name, header_value);
        });

        let mut cookies: HashMap<String, String> = HashMap::new();

        req.cookies().iter().for_each(|cookie| {
            if let Some(first_cookie) = cookie.get(0) {
                let (name, value) = (first_cookie.name(), first_cookie.value());
                cookies.insert(name.to_string(), value.to_string());
            }
        });

        let query_string = req.query_string();

        let query_params = url::form_urlencoded::parse(query_string.as_bytes())
            .filter_map(|(key, value)| Some((key.to_string(), value.to_string())))
            .collect::<HashMap<String, String>>();

        let mut body = actix_web::web::BytesMut::new();

        let content_type = match req.content_type() {
            "application/json" => RequestBodyType::JSON,
            "application/x-www-form-urlencoded" => RequestBodyType::FORM,
            _ => RequestBodyType::TEXT,
        };

        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            if (body.len() + chunk.len()) > 262_144 {
                return Err(actix_web::error::ErrorBadRequest("Body too large"));
            }
            body.extend_from_slice(&chunk);
        }

        let request_body = match content_type {
            RequestBodyType::FORM => {
                let form_data = match std::str::from_utf8(&body) {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        return Err(actix_web::error::ErrorBadRequest("Invalid UTF-8 sequence"));
                    }
                };

                RequestBody::new_form(form_data)
            }
            RequestBodyType::JSON => {
                let body_json = match std::str::from_utf8(&body) {
                    Ok(s) => match serde_json::from_str::<serde_json::Value>(s) {
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

                RequestBody::new_json(body_json)
            }
            RequestBodyType::TEXT => {
                let body_string = match std::str::from_utf8(&body) {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        return Err(actix_web::error::ErrorBadRequest("Invalid UTF-8 sequence"));
                    }
                };

                RequestBody::new_text(body_string)

                // RequestBody:: {
                //     content: RequestBodyContent::TEXT(body_string),
                //     content_type: RequestBodyType::TEXT,
                // }
            }
        };

        Ok(HttpRequest {
            params,
            query_params,
            origin_url,
            method,
            ip,
            path,
            protocol,
            is_secure,
            headers,
            data: HashMap::new(),
            body: request_body,
            cookies
        })
    }
}
