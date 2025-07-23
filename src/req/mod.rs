use std::collections::HashMap;

pub struct HttpRequest {
    origin_url: String,
    params: HashMap<String, String>,
}

impl HttpRequest {
    pub fn new() -> Self {
        HttpRequest {
            origin_url: String::new(),
            params: HashMap::new(),
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

        println!("{:?}", params);

        HttpRequest { params, origin_url }
    }
}
