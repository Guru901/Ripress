#![warn(missing_docs)]

use crate::{
    helpers::get_all_query_params,
    types::{HttpMethods, HttpRequestError, RequestBody, RequestBodyContent, RequestBodyType},
};
use cookie::Cookie;
use hyper::{Body, Method, Request, body::to_bytes, header::HOST};
use mime::Mime;
use routerify::ext::RequestExt;
use serde_json::Value;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
};

/// A struct that represents the request headers.
/// And it's methods.
pub mod request_headers;

/// A struct that represents the origin url of the request.
/// And it's methods.
pub mod origin_url;

/// A struct that represents the query parameters of the request.
/// And it's methods.
pub mod query_params;

/// A struct that represents the route parameters of the request.
/// And it's methods.
pub mod route_params;

use origin_url::Url;
use query_params::QueryParams;
use request_headers::RequestHeaders;
use route_params::RouteParams;

/// Represents an incoming HTTP request with comprehensive access to request data.
///
/// The HttpRequest struct provides methods to access and manipulate all aspects
/// of an HTTP request including headers, cookies, query parameters, route parameters,
/// and request body content.
///
/// ## Examples
///
/// Basic usage:
/// ```rust
/// use ripress::context::HttpRequest;
///
/// let req = HttpRequest::new();
/// println!("Method: {:?}", req.method);
/// println!("Path: {}", req.path);
/// println!("Client IP: {:?}", req.ip);
/// ```
///
/// Working with JSON body:
/// ```rust
/// use ripress::context::HttpRequest;
/// use serde::{Deserialize, Serialize};
///
/// ##[derive(Deserialize, Serialize)]
/// struct User {
///     name: String,
///     age: u32
/// }
///
/// let req = HttpRequest::new();
/// if let Ok(user) = req.json::<User>() {
///     println!("User: {} is {} years old", user.name, user.age);
/// }
/// ```
///

#[derive(Clone)]
pub struct HttpRequest {
    /// Dynamic route parameters extracted from the URL.
    pub params: RouteParams,

    /// Query parameters from the request URL.
    pub query_params: QueryParams,

    /// The full URL of the incoming request.
    pub origin_url: Url,

    /// The HTTP method used for the request (e.g., GET, POST, PUT, DELETE).
    pub method: HttpMethods,

    /// The IP address of the client making the request.
    pub ip: IpAddr,

    /// The requested endpoint path.
    pub path: String,

    /// Protocol of the request (HTTP or HTTPs)
    pub protocol: String,

    /// Boolean indicating if the request was made with AJAX (XMLHttpRequest).
    pub xhr: bool,

    /// Boolean indicating if the request was made with Https
    pub is_secure: bool,

    /// The request's headers
    pub headers: RequestHeaders,

    /// The request's cookies
    cookies: HashMap<String, String>,

    // The Data set by middleware in the request to be used in the route handler
    data: HashMap<String, String>,

    /// The request body, which may contain JSON, text, or form data.
    body: RequestBody,
}

impl HttpRequest {
    /// Creates a new empty HTTP request instance.
    ///
    /// ## Returns
    ///
    /// Returns a new `HttpRequest` with default values.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::context::HttpRequest;
    /// use ripress::types::HttpMethods;
    ///
    /// let req = HttpRequest::new();
    /// assert_eq!(req.method, HttpMethods::GET);
    /// ```

    pub fn new() -> Self {
        HttpRequest {
            origin_url: Url::new(""),
            params: RouteParams::new(),
            query_params: QueryParams::new(),
            method: HttpMethods::GET,
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            path: String::new(),
            protocol: String::new(),
            headers: RequestHeaders::new(),
            data: HashMap::new(),
            body: RequestBody::new_text(String::new()),
            cookies: HashMap::new(),
            xhr: false,
            is_secure: false,
        }
    }

    /// Retrieves a cookie value by name.
    ///
    /// ## Arguments
    ///
    /// * `name` - The name of the cookie to retrieve
    ///
    /// ## Returns
    ///
    /// Returns `Ok(&str)` with the cookie value if found, or
    /// `Err(HttpRequestError::MissingCookie)` if not found.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::context::HttpRequest;
    ///
    /// let req = HttpRequest::new();
    /// match req.get_cookie("session_id") {
    ///     Ok(session) => println!("Session ID: {}", session),
    ///     Err(e) => println!("No session cookie found: {:?}", e)
    /// }
    /// ```

    pub fn get_cookie(&self, name: &str) -> Result<&str, HttpRequestError> {
        let cookie = self.cookies.get(name);

        match cookie {
            Some(cookie_str) => Ok(cookie_str),
            None => Err(HttpRequestError::MissingCookie(name.to_string())),
        }
    }

    /// Adds data from the middleware into the request.
    ///
    /// ## Arguments
    ///
    /// * `key` - The key of the data to retrieve
    /// * `value` - The value of the data to retrieve
    ///
    /// ## Example
    /// ```
    /// let mut req = ripress::context::HttpRequest::new();
    /// req.set_data("id", "123");
    /// let id = req.get_data("id");
    /// println!("Id: {:?}", id);
    /// ```

    pub fn set_data<T: Into<String>>(&mut self, data_key: T, data_value: T) {
        self.data.insert(data_key.into(), data_value.into());
    }

    pub fn get_all_data(&self) -> &HashMap<String, String> {
        &self.data
    }

    /// Returns data stored in the request by the middleware.
    ///
    /// ## Arguments
    ///
    /// * `key` - The key of the data to retrieve
    ///
    /// ## Returns
    ///
    /// Returns `Option<&String>` with the data value if found, or `None` if not found.
    ///
    /// ## Example
    /// ```
    /// let req = ripress::context::HttpRequest::new();
    /// let id = req.get_data("id");
    /// println!("Id: {:?}", id);
    /// ```

    pub fn get_data<T: Into<String>>(&self, data_key: T) -> Option<&String> {
        self.data.get(&data_key.into())
    }

    /// Checks if the request body matches a specific content type.
    ///
    /// ## Arguments
    ///
    /// * `content_type` - The `RequestBodyType` to check against
    ///
    /// ## Returns
    ///
    /// Returns `true` if the content type matches, `false` otherwise.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::{context::HttpRequest, types::RequestBodyType};
    ///
    /// let req = HttpRequest::new();
    /// if req.is(RequestBodyType::JSON) {
    ///     // Handle JSON content
    /// }
    /// ```

    pub fn is(&self, content_type: RequestBodyType) -> bool {
        self.body.content_type == content_type
    }

    /// Deserializes the request body as JSON into the specified type.
    ///
    /// ## Type Parameters
    ///
    /// * `J` - The type to deserialize into, must implement `DeserializeOwned`
    ///
    /// ## Returns
    ///
    /// Returns `Ok(J)` with the deserialized value if successful, or
    /// `Err(String)` with an error message if deserialization fails.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::context::HttpRequest;
    /// use serde::{Deserialize, Serialize};
    ///
    /// ##[derive(Deserialize, Serialize)]
    /// struct LoginData {
    ///     username: String,
    ///     password: String
    /// }
    ///
    /// let req = HttpRequest::new();
    /// match req.json::<LoginData>() {
    ///     Ok(data) => println!("Login attempt for: {}", data.username),
    ///     Err(e) => println!("Invalid login data: {}", e)
    /// }
    /// ```

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

    /// Returns request's text body.
    ///
    /// ## Example
    /// ```
    /// let req = ripress::context::HttpRequest::new();
    /// let text = req.text().unwrap();
    /// println!("text : {:?}", text);
    /// ```
    ///
    /// This function returns the text body of the request.
    /// Returns an `Result<String>`, where `Ok(String)` contains the body if it is valid text, or `Err(error)` if it is not.

    pub fn text(&self) -> Result<String, String> {
        let body = &self.body;

        if body.content_type == RequestBodyType::TEXT {
            if let RequestBodyContent::TEXT(ref text_value) = body.content {
                Ok(text_value.clone().to_owned())
            } else {
                Err(String::from("Invalid text content"))
            }
        } else {
            Err(String::from("Wrong body type"))
        }
    }

    /// Returns request's form_data body.
    ///
    /// ## Example
    /// ```no_run
    /// let req = ripress::context::HttpRequest::new();
    /// // Let' say form data was sent as key=value and key2=value2
    /// let form_data = req.form_data().unwrap();
    /// println!("key = : {:?}", form_data.get("key"));
    /// println!("key2 = : {:?}", form_data.get("key2"));
    /// ```
    ///
    /// This function returns a HashMap of the form data.
    /// Returns an `Result<HashMap<String, String>>`, where `Ok(HashMap<String, String>)` contains the form_data if it is valid form data, or `Err(error)` if it is not.

    pub fn form_data(&self) -> Result<HashMap<String, String>, String> {
        let mut form_data: HashMap<String, String> = HashMap::new();
        let body = &self.body;

        if body.content_type == RequestBodyType::FORM {
            if let RequestBodyContent::FORM(text_value) = &body.content {
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

    pub(crate) fn set_param(&mut self, key: &str, value: &str) {
        self.params.insert(key.to_string(), value.to_string());
    }

    pub(crate) fn determine_content_type(content_type: &str) -> RequestBodyType {
        match content_type.parse::<Mime>() {
            Ok(mime_type) => match (mime_type.type_(), mime_type.subtype()) {
                (mime::APPLICATION, mime::JSON) => RequestBodyType::JSON,
                (mime::APPLICATION, subtype) if subtype == "x-www-form-urlencoded" => {
                    RequestBodyType::FORM
                }
                (mime::MULTIPART, mime::FORM_DATA) => RequestBodyType::FORM,
                (mime::TEXT, _) => RequestBodyType::TEXT,
                // Handle JSON variants
                (mime::APPLICATION, subtype) if subtype.as_str().ends_with("+json") => {
                    RequestBodyType::JSON
                }
                _ => RequestBodyType::TEXT,
            },
            Err(_) => RequestBodyType::TEXT, // Fallback for invalid MIME types
        }
    }

    fn get_cookies(req: &Request<Body>) -> Vec<Cookie<'_>> {
        let mut cookies = Vec::new();

        if let Some(header_value) = req.headers().get("cookie") {
            if let Ok(header_str) = header_value.to_str() {
                for cookie_str in header_str.split(';') {
                    if let Ok(cookie) = Cookie::parse(cookie_str.trim()) {
                        cookies.push(cookie);
                    }
                }
            }
        }

        cookies
    }

    pub(crate) async fn from_hyper_request(
        req: &mut Request<hyper::body::Body>,
    ) -> Result<Self, hyper::Error> {
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
                        // Determine scheme (you might want to check for TLS context)
                        let scheme = "http"; // or "https" if using TLS
                        format!("{}://{}", scheme, host)
                    })
                    .unwrap_or(String::new());

                Url::new(uri_string)
            }
        };

        let query_string = req.uri().query().unwrap_or("");

        let queries = url::form_urlencoded::parse(query_string.as_bytes())
            .filter_map(|(key, value)| Some((key.to_string(), value.to_string())))
            .collect::<HashMap<String, String>>();

        let query_params = QueryParams::from_map(queries);

        let method = match req.method() {
            &Method::GET => HttpMethods::GET,
            &Method::POST => HttpMethods::POST,
            &Method::PUT => HttpMethods::PUT,
            &Method::DELETE => HttpMethods::DELETE,
            &Method::PATCH => HttpMethods::PATCH,
            &Method::HEAD => HttpMethods::HEAD,
            &Method::OPTIONS => HttpMethods::OPTIONS,
            _ => HttpMethods::GET,
        };

        let ip = req
            .headers()
            .get("X-Forwarded-For")
            .and_then(|val| val.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .unwrap_or(String::new());

        let ip = match ip.parse::<IpAddr>() {
            Ok(ip) => ip,
            Err(_) => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        };

        let path = req.uri().path().to_string();

        let mut cookies_map = HashMap::new();
        let cookies = Self::get_cookies(&req);

        cookies.iter().for_each(|cookie| {
            let (name, value) = (cookie.name(), cookie.value());
            cookies_map.insert(name.to_string(), value.to_string());
        });

        let mut headers: HashMap<String, String> = HashMap::new();

        req.headers().iter().for_each(|(key, value)| {
            headers.insert(key.to_string(), value.to_str().unwrap_or("").to_string());
        });

        let headers = RequestHeaders::_from_map(headers);

        let mut params = HashMap::new();

        if let Some(param_routerify) = req.data::<routerify::RouteParams>() {
            println!("Params: {:?}", param_routerify);
            param_routerify.iter().for_each(|(key, value)| {
                params.insert(key.to_string(), value.to_string());
            });
        }

        let params = RouteParams::from_map(params);

        let mut data = HashMap::new();

        if let Some(ext_data) = req.extensions().get::<HashMap<String, String>>() {
            data = ext_data.clone()
        }

        let content_type = req
            .headers()
            .get("Content-Type")
            .and_then(|val| val.to_str().ok())
            .map(Self::determine_content_type)
            .unwrap_or(RequestBodyType::EMPTY);

        let protocol = req
            .headers()
            .get("x-forwarded-proto")
            .and_then(|val| val.to_str().ok())
            .unwrap_or("http")
            .to_string();

        let request_body = match content_type {
            RequestBodyType::FORM => {
                let body_bytes = to_bytes(req.body_mut()).await;

                let body_string = match body_bytes {
                    Ok(bytes) => std::str::from_utf8(&bytes).unwrap_or("").to_string(),
                    Err(err) => return Err(err),
                };

                RequestBody::new_form(body_string)
            }
            RequestBodyType::JSON => {
                let body_bytes = to_bytes(req.body_mut()).await;
                let body_json = match body_bytes {
                    Ok(bytes) => {
                        let s = match std::str::from_utf8(&bytes) {
                            Ok(s) => s,
                            Err(_) => "",
                        };
                        match serde_json::from_str::<serde_json::Value>(s) {
                            Ok(json) => json,
                            Err(_) => Value::Null,
                        }
                    }
                    Err(err) => return Err(err),
                };

                RequestBody::new_json(body_json)
            }
            RequestBodyType::TEXT => {
                let body_bytes = to_bytes(req.body_mut()).await;

                let body_string = match body_bytes {
                    Ok(bytes) => String::from_utf8((&bytes).to_vec()).unwrap_or(String::new()),
                    Err(err) => return Err(err),
                };

                RequestBody::new_text(body_string)
            }
            RequestBodyType::EMPTY => RequestBody {
                content: RequestBodyContent::EMPTY,
                content_type: RequestBodyType::EMPTY,
            },
        };

        let is_secure = protocol == String::from("https");
        let xhr_header = headers.get("X-Requested-With").unwrap_or("");
        let xhr = xhr_header == "XMLHttpRequest";

        Ok(HttpRequest {
            params,
            query_params,
            origin_url,
            method,
            ip,
            path,
            protocol,
            headers,
            data,
            body: request_body,
            cookies: cookies_map,
            xhr,
            is_secure,
        })
    }

    pub(crate) fn to_hyper_request(&self) -> Result<Request<Body>, Box<dyn std::error::Error>> {
        let path = if self.path.is_empty() {
            "/".to_string()
        } else if !self.path.starts_with('/') {
            return Err("Path must start with '/'".into());
        } else {
            self.path.clone()
        };

        let mut uri_builder = path.to_string();
        if !self.query_params.is_empty() {
            uri_builder.push('?');
            uri_builder.push_str(&get_all_query_params(&self.query_params));
        }

        let uri: hyper::Uri = uri_builder
            .parse()
            .map_err(|e| format!("Failed to parse URI '{}': {}", uri_builder, e))?;

        let mut builder = Request::builder()
            .method(self.method.to_string().as_str())
            .uri(uri);

        // Rest of the code remains the same...
        if let Some(headers) = builder.headers_mut() {
            //
            for header in self.headers.iter() {
                if let Ok(header_name) = hyper::header::HeaderName::from_bytes(header.0.as_bytes())
                {
                    if let Ok(header_value) = hyper::header::HeaderValue::from_str(header.1) {
                        headers.insert(header_name, header_value);
                    }
                }

                let cookie_str: String = self
                    .cookies
                    .iter()
                    .map(|(name, value)| format!("{}={}", name, value))
                    .collect::<Vec<_>>()
                    .join("; ");
                headers.insert(hyper::header::COOKIE, cookie_str.parse()?);
            }
        }

        let data = self.get_all_data();

        if let Some(ext) = builder.extensions_mut() {
            ext.insert(data.clone());
        }

        let body = match &self.body.content {
            RequestBodyContent::JSON(json) => {
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "application/json".parse()?);
                Body::from(serde_json::to_string(json)?)
            }
            RequestBodyContent::TEXT(text) => {
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "text/plain".parse()?);
                Body::from(text.clone())
            }
            RequestBodyContent::FORM(form) => {
                builder.headers_mut().unwrap().insert(
                    hyper::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded".parse()?,
                );
                Body::from(form.clone())
            }
            RequestBodyContent::EMPTY => Body::empty(),
        };

        Ok(builder.body(body)?)
    }
}

#[cfg(test)]
impl HttpRequest {
    pub(crate) fn set_query(&mut self, key: &str, value: &str) {
        self.query_params.insert(key.to_string(), value.to_string());
    }

    pub(crate) fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub(crate) fn set_cookie(&mut self, key: &str, value: &str) {
        self.cookies.insert(key.to_string(), value.to_string());
    }

    pub(crate) fn set_json<J>(&mut self, json: J, content_type: RequestBodyType)
    where
        J: serde::de::DeserializeOwned + serde::Serialize,
    {
        self.body.content_type = content_type;
        self.body.content = RequestBodyContent::JSON(serde_json::to_value(json).unwrap());
    }

    pub(crate) fn set_text<T: Into<String>>(&mut self, text: T, content_type: RequestBodyType) {
        self.body.content_type = content_type;
        self.body.content = RequestBodyContent::TEXT(text.into())
    }

    pub(crate) fn set_form(
        &mut self,
        key: &'static str,
        value: &'static str,
        content_type: RequestBodyType,
    ) {
        self.body.content_type = content_type;

        match &mut self.body.content {
            RequestBodyContent::FORM(existing) => {
                let mut new_form = existing.to_string();
                new_form.push('&');
                new_form.push_str(&format!("{key}={value}"));
                self.body.content = RequestBodyContent::FORM(new_form)
            }
            _ => {
                let form_data = format!("{key}={value}");
                self.body.content = RequestBodyContent::FORM(form_data)
            }
        }
    }

    pub(crate) fn set_content_type(&mut self, content_type: RequestBodyType) {
        self.body.content_type = content_type;
    }

    pub(crate) fn set_method(&mut self, method: HttpMethods) {
        self.method = method;
    }

    pub(crate) fn set_path(&mut self, path: String) {
        self.path = path;
    }

    pub(crate) fn set_origin_url(&mut self, origin_url: Url) {
        self.origin_url = origin_url;
    }
}
