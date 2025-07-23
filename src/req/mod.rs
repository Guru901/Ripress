use crate::types::HttpMethod;
use std::collections::HashMap;

pub struct HttpRequest {
    origin_url: String,
    params: HashMap<String, String>,
    method: HttpMethod,
    ip: String,
    path: String,
    protocol: String,
    is_secure: bool,
}

impl HttpRequest {
    pub fn new() -> Self {
        HttpRequest {
            origin_url: String::new(),
            params: HashMap::new(),
            method: HttpMethod::GET,
            ip: String::new(),
            path: String::new(),
            protocol: String::new(),
            is_secure: false,
        }
    }

    pub fn get_param(&self, param_name: &str) -> Option<&str> {
        match self.params.get(param_name) {
            Some(param) => Some(param.as_str()),
            None => None,
        }
    }

    pub fn get_origin_url(&self) -> &String {
        &self.origin_url
    }

    pub fn get_method(&self) -> &HttpMethod {
        &self.method
    }

    pub fn get_ip(&self) -> &String {
        &self.ip
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }

    pub fn get_protocol(&self) -> &String {
        &self.protocol
    }
    
    pub fn is_secure (&self) -> &bool {
        &self.is_secure
    }

    pub async fn from_actix_request(
        req: actix_web::HttpRequest,
        _payload: actix_web::web::Payload,
    ) -> Self {
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

        HttpRequest {
            params,
            origin_url,
            method,
            ip,
            path,
            protocol,
            is_secure,
        }
    }
}
