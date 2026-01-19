use std::net::{IpAddr, Ipv4Addr};

use ahash::AHashMap;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{header::HOST, Request};
use routerify_ng::RequestInfo;
use serde_json::Value;

use crate::{
    app::api_error::ApiError,
    helpers::{
        determine_content_type_request, extract_boundary, get_all_query, parse_multipart_form,
    },
    req::{
        body::{FormData, RequestBody, RequestBodyContent, RequestBodyType, TextData},
        origin_url::Url,
        query_params::QueryParams,
        request_data::RequestData,
        request_headers::RequestHeaders,
        route_params::RouteParams,
        HttpRequest,
    },
    types::HttpMethods,
};

impl HttpRequest {
    #[doc(hidden)]
    pub async fn from_hyper_request(req: &mut Request<Full<Bytes>>) -> Result<Self, ApiError> {
        let origin_url = match req.uri().authority() {
            Some(authority) => {
                let scheme = req.uri().scheme_str().unwrap_or("http");
                Url::new(format!("{}://{}", scheme, authority))
            }
            None => {
                let uri_string = req
                    .headers()
                    .get(HOST)
                    .and_then(|host| host.to_str().ok())
                    .map(|host| {
                        let scheme = "http";
                        format!("{}://{}", scheme, host)
                    })
                    .unwrap_or(String::new());

                Url::new(uri_string)
            }
        };

        let query_string = req.uri().query().unwrap_or("");
        let queries = url::form_urlencoded::parse(query_string.as_bytes())
            .filter_map(|(key, value)| Some((key.to_string(), value.to_string())));
        let query = QueryParams::from_iterator(queries);

        let method = HttpMethods::from(req.method());
        let path = req.uri().path().to_string();

        // Extract header values BEFORE taking ownership
        // These are just &str references that we'll copy to String
        let cookie_str_opt = req
            .headers()
            .get(hyper::header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string()); // Convert to owned String

        let x_forwarded_for_str = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("127.0.0.1")
            .to_string(); // Convert to owned String

        let x_forwarded_proto_str = req
            .headers()
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("http")
            .to_string(); // Convert to owned String

        let content_type_str_opt = req
            .headers()
            .get(hyper::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string()); // Convert to owned String

        let xhr_header_opt = req
            .headers()
            .get("x-requested-with")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string()); // Convert to owned String

        // NOW we can take ownership of the HeaderMap
        let headers = RequestHeaders::from_header_map(std::mem::take(req.headers_mut()));

        // Parse IP
        let ip = x_forwarded_for_str
            .split(',')
            .next()
            .map(|s| s.trim())
            .and_then(|s| s.parse::<IpAddr>().ok())
            .unwrap();

        // Parse cookies from cached header value
        let mut cookies_map = AHashMap::new();
        if let Some(cookie_str) = &cookie_str_opt {
            for cookie_part in cookie_str.split(';') {
                let trimmed = cookie_part.trim();
                if let Some(equal_pos) = trimmed.find('=') {
                    let name = &trimmed[..equal_pos];
                    let value = &trimmed[equal_pos + 1..];
                    if !name.is_empty() {
                        cookies_map.insert(name.to_string(), value.to_string());
                    }
                }
            }
        }

        let mut data = RequestData::new();
        if let Some(ext_data) = req.extensions().get::<RequestData>() {
            data = ext_data.clone();
        }

        // Use cached content-type header
        let content_type = content_type_str_opt
            .as_deref()
            .map(determine_content_type_request)
            .unwrap_or(RequestBodyType::EMPTY);

        let request_body = match content_type {
            RequestBodyType::FORM => {
                let collected = req.body_mut().collect().await?;
                let body_bytes = collected.to_bytes();
                let body_string = std::str::from_utf8(&body_bytes);
                if let Err(e) = body_string {
                    eprintln!("Error parsing form data: {}", e);
                    eprintln!("Defaulting to empty form data");
                }
                match FormData::from_query_string(&body_string.unwrap()) {
                    Ok(fd) => RequestBody::new_form(fd),
                    Err(_e) => RequestBody::new_form(FormData::new()),
                }
            }
            RequestBodyType::MultipartForm => {
                let collected = req.body_mut().collect().await?;
                let body_bytes = collected.to_bytes();

                // Use cached content-type instead of another header lookup
                let boundary = content_type_str_opt
                    .as_deref()
                    .filter(|ct| ct.to_lowercase().contains("multipart/form-data"))
                    .and_then(|ct| extract_boundary(&ct));

                let (fields, file_parts) = if let Some(boundary) = boundary {
                    let (field_refs, files) = parse_multipart_form(&body_bytes, &boundary);
                    // Convert borrowed fields to owned strings
                    let owned_fields = field_refs
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect::<Vec<(String, String)>>();
                    (owned_fields, files)
                } else {
                    let body_string = String::from_utf8_lossy(&body_bytes);
                    match FormData::from_query_string(&body_string) {
                        Ok(fd) => {
                            // Extract owned strings instead of references
                            let form_fields = fd
                                .iter()
                                .map(|(k, v)| (k.to_string(), v.to_string()))
                                .collect::<Vec<(String, String)>>();
                            (form_fields, Vec::new())
                        }
                        Err(_e) => (Vec::new(), Vec::new()),
                    }
                };

                let mut form_data = FormData::new();
                for (key, value) in &fields {
                    form_data.insert(key, value);
                }

                if !file_parts.is_empty() {
                    RequestBody::new_binary_with_form_fields(body_bytes, form_data)
                } else {
                    RequestBody::new_form(form_data)
                }
            }
            RequestBodyType::JSON => {
                let collected = req.body_mut().collect().await?;
                let body_bytes = collected.to_bytes();
                let body_json = match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                    Ok(json) => json,
                    Err(e) => {
                        eprintln!("Error parsing JSON: {}", e);
                        eprintln!("Defaulting to null JSON");
                        Value::Null
                    }
                };
                RequestBody::new_json(body_json)
            }
            RequestBodyType::TEXT => {
                let collected = req.body_mut().collect().await?;
                let body_bytes = collected.to_bytes();
                match TextData::from_bytes(body_bytes.as_ref().to_vec()) {
                    Ok(text) => RequestBody::new_text(text),
                    Err(_) => RequestBody::new_binary(body_bytes),
                }
            }
            RequestBodyType::BINARY => {
                let collected = req.body_mut().collect().await?;
                let body_bytes = collected.to_bytes();
                RequestBody::new_binary(body_bytes)
            }
            RequestBodyType::EMPTY => RequestBody {
                content: RequestBodyContent::EMPTY,
                content_type: RequestBodyType::EMPTY,
            },
        };

        let is_secure = x_forwarded_proto_str == "https";
        let xhr = xhr_header_opt
            .as_deref()
            .map_or(false, |h| h == "XMLHttpRequest");

        Ok(HttpRequest {
            params: RouteParams::new(),
            query,
            origin_url,
            method,
            ip,
            path,
            protocol: x_forwarded_proto_str,
            headers,
            data,
            body: request_body,
            cookies: cookies_map,
            xhr,
            is_secure,
        })
    }
    pub(crate) fn from_request_info(req_info: &RequestInfo) -> Self {
        let mut headers = RequestHeaders::new();

        req_info.headers().iter().for_each(|(key, value)| {
            if let Ok(v) = value.to_str() {
                headers.insert(key.as_str(), v);
            }
        });

        let method = HttpMethods::from(req_info.method());
        let origin_url = match req_info.uri().authority() {
            Some(authority) => {
                let scheme = req_info.uri().scheme_str().unwrap_or("http");
                Url::new(format!("{}://{}", scheme, authority))
            }
            None => {
                let uri_string = req_info
                    .headers()
                    .get(HOST)
                    .and_then(|host: &hyper::header::HeaderValue| host.to_str().ok())
                    .map(|host| {
                        // Determine scheme (you might want to check for TLS context)
                        let scheme = "http"; // or "https" if using TLS
                        format!("{}://{}", scheme, host)
                    })
                    .unwrap_or(String::new());

                Url::new(uri_string)
            }
        };

        let query_string = req_info.uri().query().unwrap_or("");

        let queries = url::form_urlencoded::parse(query_string.as_bytes())
            .filter_map(|(key, value)| Some((key.to_string(), value.to_string())))
            .collect::<AHashMap<String, String>>();

        let query = QueryParams::from_ahashmap(queries);
        let params = RouteParams::new();

        let mut cookies_map = AHashMap::new();
        let cookies = Self::get_cookies_from_req_info(&req_info);

        cookies.iter().for_each(|cookie| {
            let (name, value) = (cookie.name(), cookie.value());
            cookies_map.insert(name.to_string(), value.to_string());
        });

        let ip = req_info
            .headers()
            .get("X-Forwarded-For")
            .and_then(|val: &hyper::header::HeaderValue| val.to_str().ok())
            .map(|s: &str| s.split(',').next().unwrap_or("").trim().to_string())
            .unwrap_or(String::new());

        let ip = match ip.parse::<IpAddr>() {
            Ok(ip) => ip,
            Err(_) => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        };

        let protocol = req_info
            .headers()
            .get("x-forwarded-proto")
            .and_then(|val| val.to_str().ok())
            .unwrap_or("http")
            .to_string();

        let is_secure = protocol == String::from("https");

        let xhr_header = headers.get("x-requested-with").unwrap_or("");
        let xhr = xhr_header == "XMLHttpRequest";

        // Try to get RequestData from the request info
        let mut data = RequestData::new();
        if let Some(ext_data) = req_info.data::<RequestData>() {
            data = ext_data.clone();
        }

        Self {
            body: RequestBody {
                content: RequestBodyContent::EMPTY,
                content_type: RequestBodyType::EMPTY,
            },
            cookies: cookies_map,
            headers,
            is_secure,
            method,
            origin_url,
            params,
            path: req_info.uri().path().to_string(),
            query,
            xhr,
            data,
            ip,
            protocol,
        }
    }

    #[cfg(feature = "with-wynd")]
    #[doc(hidden)]
    pub fn to_hyper_request(&self) -> Result<Request<Full<Bytes>>, Box<dyn std::error::Error>> {
        let path = if self.path.is_empty() {
            "/".to_string()
        } else if !self.path.starts_with('/') {
            return Err("Path must start with '/'".into());
        } else {
            self.path.clone()
        };

        let mut uri_builder = path.to_string();
        if !self.query.is_empty() {
            uri_builder.push('?');
            uri_builder.push_str(&get_all_query(&self.query));
        }

        let uri: hyper::Uri = uri_builder
            .parse()
            .map_err(|e| format!("Failed to parse URI '{}': {}", uri_builder, e))?;

        let mut builder = Request::builder()
            .method(self.method.to_string().as_str())
            .uri(uri);

        // Add headers
        if let Some(headers) = builder.headers_mut() {
            // Add all headers
            if !self.headers.is_empty() {
                // If all header names and values are valid, batch convert and insert (to minimize branching)
                for (name, value) in self.headers.iter() {
                    if headers.contains_key(name) {
                        headers.append(name, value.clone());
                    } else {
                        headers.insert(name, value.clone());
                    }
                }
            }

            if !self.cookies.is_empty() && !headers.contains_key(hyper::header::COOKIE) {
                let cookie_str: String = self
                    .cookies
                    .iter()
                    .map(|(name, value)| format!("{}={}", name, value))
                    .collect::<Vec<_>>()
                    .join("; ");
                headers.insert(
                    hyper::header::COOKIE,
                    hyper::header::HeaderValue::from_str(&cookie_str)?,
                );
            }
        }

        let data = self.get_all_data();

        if let Some(ext) = builder.extensions_mut() {
            ext.insert(data.clone());
        }
        let body = match &self.body.content {
            RequestBodyContent::JSON(json) => {
                let json_str = serde_json::to_string(json)?;
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "application/json".parse()?);
                Full::from(hyper::body::Bytes::from(json_str))
            }
            RequestBodyContent::TEXT(text) => {
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "text/plain".parse()?);
                Full::from(hyper::body::Bytes::from(text.as_bytes().to_vec()))
            }
            RequestBodyContent::FORM(form) => {
                let form_str = form.to_string();
                builder.headers_mut().unwrap().insert(
                    hyper::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded".parse()?,
                );
                Full::from(hyper::body::Bytes::from(form_str))
            }
            RequestBodyContent::BINARY(bytes) => {
                builder.headers_mut().unwrap().insert(
                    hyper::header::CONTENT_TYPE,
                    "application/octet-stream".parse()?,
                );
                Full::from(bytes.clone())
            }
            RequestBodyContent::BinaryWithFields(bytes, _form_data) => {
                // For multipart forms with files, we send the binary data
                // but the form fields are accessible via form_data()
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "multipart/form-data".parse()?);
                Full::from(bytes.clone())
            }
            RequestBodyContent::EMPTY => Full::from(Bytes::new()),
        };
        let request = builder.body(body).unwrap();
        Ok(request)
    }

    #[cfg(not(feature = "with-wynd"))]
    #[doc(hidden)]
    pub fn to_hyper_request(&self) -> Result<Request<Full<Bytes>>, Box<dyn std::error::Error>> {
        let mut path = if self.path.is_empty() {
            "/".to_string()
        } else if !self.path.starts_with('/') {
            return Err("Path must start with '/'".into());
        } else {
            self.path.clone()
        };
        if !self.query.is_empty() {
            path.push('?');
            path.push_str(&get_all_query(&self.query));
        }

        let uri: hyper::Uri = path
            .parse()
            .map_err(|e| format!("Failed to parse URI '{}': {}", path, e))?;

        let mut builder = Request::builder()
            .method(self.method.to_string().as_str())
            .uri(uri);

        // Add headers
        if let Some(headers) = builder.headers_mut() {
            // Add all headers
            if !self.headers.is_empty() {
                // If all header names and values are valid, batch convert and insert (to minimize branching)
                for (name, value) in self.headers.iter() {
                    if headers.contains_key(name) {
                        headers.append(name, value.clone());
                    } else {
                        headers.insert(name, value.clone());
                    }
                }
            }

            if !self.cookies.is_empty() && !headers.contains_key(hyper::header::COOKIE) {
                let cookie_str = {
                    let cap = self
                        .cookies
                        .iter()
                        .map(|(k, v)| k.len() + v.len() + 2)
                        .sum::<usize>()
                        .saturating_sub(2);

                    let mut s = String::with_capacity(cap);
                    let mut first = true;

                    for (name, value) in &self.cookies {
                        if !first {
                            s.push_str("; ");
                        }
                        first = false;
                        s.push_str(name);
                        s.push('=');
                        s.push_str(value);
                    }
                    s
                };
                let cookie = cookie_str.as_bytes();
                headers.insert(
                    hyper::header::COOKIE,
                    hyper::header::HeaderValue::from_bytes(&cookie)?,
                );
            }
        }
        let data = self.get_all_data();
        if let Some(ext) = builder.extensions_mut() {
            ext.insert(data.clone());
        }
        let body = match &self.body.content {
            RequestBodyContent::JSON(json) => {
                let json_str = serde_json::to_string(json)?;

                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "application/json".parse()?);
                Full::from(Bytes::from(json_str))
            }
            RequestBodyContent::TEXT(text) => {
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "text/plain".parse()?);
                Full::from(Bytes::from(text.as_bytes().to_vec()))
            }
            RequestBodyContent::FORM(form) => {
                builder.headers_mut().unwrap().insert(
                    hyper::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded".parse()?,
                );
                Full::from(Bytes::from(form.to_string()))
            }
            RequestBodyContent::BINARY(bytes) => {
                builder.headers_mut().unwrap().insert(
                    hyper::header::CONTENT_TYPE,
                    "application/octet-stream".parse()?,
                );
                Full::from(bytes.clone())
            }
            RequestBodyContent::BinaryWithFields(bytes, _form_data) => {
                // For multipart forms with files, we send the binary data
                // but the form fields are accessible via form_data()
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "multipart/form-data".parse()?);
                Full::from(bytes.clone())
            }
            RequestBodyContent::EMPTY => Full::from(Bytes::new()),
        };
        let request = builder.body(body)?;

        Ok(request)
    }
}
