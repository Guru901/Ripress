use std::collections::HashMap;

pub struct HttpRequest {
    params: HashMap<String, String>,
    queries: HashMap<String, String>,
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

    pub fn from_actix_request(req: &actix_web::HttpRequest) -> Self {
        let mut queries = HashMap::new();
        if let Some(query_string) = req.query_string().split("?").next() {
            query_string.split("&").for_each(|pair| {
                if let Some((key, value)) = pair.split_once("=") {
                    queries.insert(key.to_string(), value.to_string());
                }
            });
        }

        HttpRequest {
            params: req
                .match_info()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            queries,
        }
    }
}
