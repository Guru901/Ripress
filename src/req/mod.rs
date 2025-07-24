use crate::types::HttpMethod;
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
    data: HashMap<String, String>,
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

        let mut headers = HashMap::new();

        req.headers().iter().for_each(|f| {
            let header_name = f.0.to_string();
            let header_value = f.1.to_str().unwrap().to_string();
            headers.insert(header_name, header_value);
        });

        let query_string = req.query_string();

        let query_params = url::form_urlencoded::parse(query_string.as_bytes())
            .filter_map(|(key, value)| Some((key.to_string(), value.to_string())))
            .collect::<HashMap<String, String>>();

        HttpRequest {
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
        }
    }
}
