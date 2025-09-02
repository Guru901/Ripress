#![warn(missing_docs)]

use crate::{
    helpers::{extract_boundary, get_all_query, parse_multipart_form},
    req::body::{FormData, RequestBody, RequestBodyContent, RequestBodyType, TextData},
    types::{HttpMethods, ResponseContentType},
};
use cookie::Cookie;
use hyper::{Body, Request, body::to_bytes, header::HOST};
use mime::Mime;
use routerify::{RequestInfo, ext::RequestExt};
use serde_json::Value;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
};
#[cfg(feature = "with-wynd")]
use tokio::io::{AsyncRead, AsyncWrite};

/// A struct that represents the request headers.
/// And it's methods.
pub mod request_headers;

/// A struct that represents the origin url of the request.
/// And it's methods.
pub mod origin_url;

/// A struct that represents the query parameters of the request.
/// And it's methods.
pub mod query_params;

/// Structs that represents the body of the requests.
/// And it's methods.
pub mod body;

/// A struct that represents the route parameters of the request.
/// And it's methods.
pub mod route_params;

/// A struct that represents the request data of the request.
/// And it's methods.
pub mod request_data;

use request_data::RequestData;

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

#[derive(Clone, Debug)]
pub struct HttpRequest {
    /// Dynamic route parameters extracted from the URL.
    pub params: RouteParams,

    /// Query parameters from the request URL.
    pub query: QueryParams,

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
    pub(crate) cookies: HashMap<String, String>,

    /// The Data set by middleware in the request to be used in the route handler
    pub data: RequestData,

    /// The request body, which may contain JSON, text, or form data or binary data.
    pub(crate) body: RequestBody,
}

impl Default for HttpRequest {
    fn default() -> Self {
        Self::new()
    }
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
            query: QueryParams::new(),
            method: HttpMethods::GET,
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            path: String::new(),
            protocol: String::new(),
            headers: RequestHeaders::new(),
            data: RequestData::new(),
            body: RequestBody {
                content: RequestBodyContent::EMPTY,
                content_type: RequestBodyType::EMPTY,
            },
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
    /// Returns `Some(&String)` with the cookie value if found, or `None` if not found.
    ///
    /// ## Example
    /// ```rust
    /// use ripress::context::HttpRequest;
    ///
    /// let req = HttpRequest::new();
    ///
    /// match req.get_cookie("session_id") {
    ///     Some(session) => println!("Session ID: {}", session),
    ///     None => println!("No session cookie found")
    /// }
    /// ```

    pub fn get_cookie(&self, name: &str) -> Option<&String> {
        self.cookies.get(name)
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

    /// Returns all data stored in the request by the middleware.
    ///
    /// ## Returns
    ///
    /// Returns `&HashMap<String, String>` with the data.
    ///
    /// ## Example
    /// ```
    /// let req = ripress::context::HttpRequest::new();
    ///
    /// let data = req.get_all_data();
    ///
    /// println!("Data: {}", data);
    /// ```

    pub fn get_all_data(&self) -> &RequestData {
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

    pub fn get_data<T: Into<String>>(&self, data_key: T) -> Option<String> {
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
    /// use ripress::{context::HttpRequest};
    /// use ripress::req::body::RequestBodyType;
    ///
    /// let req = HttpRequest::new();
    /// if req.is(RequestBodyType::JSON) {
    ///     // Handle JSON content
    /// }
    /// ```

    pub fn is(&self, content_type: RequestBodyType) -> bool {
        self.body.content_type == content_type
    }

    /// Returns a read-only view of the raw request body when it is binary.
    ///
    /// Returns:
    /// - `Ok(&[u8])` when `content_type` is `RequestBodyType::BINARY`.
    /// - `Err("Invalid Binary Content")` if the internal variant does not match the declared type.
    /// - `Err(...)` describing the actual content type if it is not binary.
    ///
    /// ## Example
    /// ```no_run
    /// let req = ripress::context::HttpRequest::new();
    /// if let Ok(bytes) = req.bytes() {
    ///     // process bytes...
    /// }
    /// ```

    pub fn bytes(&self) -> Result<&[u8], String> {
        let body = &self.body;

        if body.content_type == RequestBodyType::BINARY {
            match &body.content {
                RequestBodyContent::BINARY(bytes) => Ok(bytes.as_ref()),
                RequestBodyContent::BinaryWithFields(bytes, _) => Ok(bytes.as_ref()),
                _ => Err(String::from("Invalid Binary Content")),
            }
        } else {
            Err(format!(
                "Wrong body type, expected binary and found {}",
                body.content_type.to_string(),
            ))
        }
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
    /// ```no_run
    /// let req = ripress::context::HttpRequest::new();
    /// let text = req.text().unwrap();
    /// println!("text : {:?}", text);
    /// ```
    ///
    /// This function returns the text body of the request.
    /// Returns an `Result<String>`, where `Ok(String)` contains the body if it is valid text, or `Err(error)` if it is not.

    pub fn text(&self) -> Result<&str, String> {
        let body = &self.body;

        if body.content_type == RequestBodyType::TEXT {
            if let RequestBodyContent::TEXT(ref text_value) = body.content {
                let value = text_value.as_str();
                match value {
                    Ok(value) => Ok(value),
                    Err(err) => Err(err.to_string()),
                }
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

    pub fn form_data(&self) -> Result<&FormData, String> {
        let body = &self.body;

        match body.content_type {
            RequestBodyType::FORM => {
                if let RequestBodyContent::FORM(form_data) = &body.content {
                    Ok(form_data)
                } else {
                    Err(String::from("Invalid form content"))
                }
            }
            RequestBodyType::BINARY => {
                // Check if this binary content also contains form fields
                if let RequestBodyContent::BinaryWithFields(_, form_data) = &body.content {
                    Ok(form_data)
                } else {
                    Err(String::from("Binary content without form fields"))
                }
            }
            _ => Err(String::from("Wrong body type")),
        }
    }

    /// Inserts a key-value pair into the request's form data.
    ///
    /// If the current body is not `FORM`, this will initialize an empty `FormData`
    /// and set the body's content type to `FORM` before inserting the field.
    /// This is useful for middlewares that wish to expose computed values through
    /// the `form_data()` API, such as attaching file upload metadata.
    pub fn insert_form_field(&mut self, key: &str, value: &str) {
        // Ensure body is of FORM type and contains a FormData map
        if self.body.content_type != RequestBodyType::FORM {
            self.body.content_type = RequestBodyType::FORM;
            self.body.content = RequestBodyContent::FORM(FormData::new());
        }

        if let RequestBodyContent::FORM(ref mut form_data) = self.body.content {
            form_data.insert(key.to_string(), value.to_string());
        }
    }

    pub(crate) fn set_param(&mut self, key: &str, value: &str) {
        self.params.insert(key.to_string(), value.to_string());
    }

    fn get_cookies_from_req(req: &Request<Body>) -> Vec<Cookie<'_>> {
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

    fn get_cookies_from_req_info(req: &RequestInfo) -> Vec<Cookie<'_>> {
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

        let query = QueryParams::from_map(queries);

        let method = HttpMethods::from(req.method());

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
        let cookies = Self::get_cookies_from_req(&req);

        cookies.iter().for_each(|cookie| {
            let (name, value) = (cookie.name(), cookie.value());
            cookies_map.insert(name.to_string(), value.to_string());
        });

        let mut headers: HashMap<String, String> = HashMap::new();

        req.headers().iter().for_each(|(key, value)| {
            if let Ok(header_value) = value.to_str() {
                headers.insert(key.to_string(), header_value.to_string());
            }
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

        let mut data = RequestData::new();

        if let Some(ext_data) = req.extensions().get::<RequestData>() {
            data = ext_data.clone();
        }

        let content_type = req
            .headers()
            .get(hyper::header::CONTENT_TYPE)
            .and_then(|val| val.to_str().ok())
            .map(determine_content_type_request)
            .unwrap_or(RequestBodyType::EMPTY);

        let protocol = req
            .headers()
            .get("x-forwarded-proto")
            .and_then(|val| val.to_str().ok())
            .unwrap_or("http")
            .to_string();

        let request_body = match content_type {
            RequestBodyType::FORM => {
                // Read the full body bytes first, then branch depending on content subtype
                let body_bytes = to_bytes(req.body_mut()).await;

                match body_bytes {
                    Ok(bytes) => {
                        let body_string = std::str::from_utf8(&bytes).unwrap_or("").to_string();
                        match FormData::from_query_string(&body_string) {
                            Ok(fd) => RequestBody::new_form(fd),
                            Err(_e) => {
                                // Prefer logging via `log`/`tracing` instead of printing.
                                RequestBody::new_form(FormData::new())
                            }
                        }
                    }
                    Err(err) => return Err(err),
                }
            }
            RequestBodyType::MultipartForm => {
                let body_bytes = to_bytes(req.body_mut()).await;

                match body_bytes {
                    Ok(bytes) => {
                        // Get content type header to extract boundary
                        let content_type_header = req
                            .headers()
                            .get(hyper::header::CONTENT_TYPE)
                            .and_then(|val| val.to_str().ok())
                            .unwrap_or_default();

                        let boundary = if content_type_header
                            .to_lowercase()
                            .contains("multipart/form-data")
                        {
                            extract_boundary(&content_type_header)
                        } else {
                            None
                        };

                        // Parse multipart/form-data using the same logic as the middleware
                        let (fields, file_parts) = if let Some(boundary) = boundary {
                            parse_multipart_form(&bytes, &boundary)
                        } else {
                            // If not multipart, try to parse as form data using the same method
                            let body_string = std::str::from_utf8(&bytes).unwrap_or("").to_string();
                            match FormData::from_query_string(&body_string) {
                                Ok(fd) => {
                                    // Convert FormData to fields vector for consistency
                                    let form_fields = fd
                                        .iter()
                                        .map(|(k, v)| (k.to_string(), v.to_string()))
                                        .collect::<Vec<(String, String)>>();
                                    (form_fields, Vec::new())
                                }
                                Err(_e) => (Vec::new(), Vec::new()),
                            }
                        };

                        // Convert fields back to FormData
                        let mut form_data = FormData::new();
                        for (key, value) in fields {
                            form_data.insert(key, value);
                        }

                        // INTELLIGENT DECISION:
                        // - Always extract and make text fields accessible via form_data()
                        // - If there are file parts, also preserve raw bytes as BINARY for middleware processing
                        // - If no file parts, just use FORM content
                        if !file_parts.is_empty() {
                            // Has files: preserve raw bytes for middleware AND make text fields accessible
                            // We'll set the body as BINARY but also insert the text fields into form_data
                            // This way both the middleware can process files AND form fields are accessible
                            RequestBody::new_binary_with_form_fields(bytes, form_data)
                        } else {
                            // No files: just use form data
                            RequestBody::new_form(form_data)
                        }
                    }
                    Err(err) => return Err(err),
                }
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
                match body_bytes {
                    Ok(bytes) => match TextData::from_bytes(bytes.as_ref().to_vec()) {
                        Ok(text) => RequestBody::new_text(text),
                        Err(_) => RequestBody::new_binary(bytes),
                    },
                    Err(err) => return Err(err),
                }
            }
            RequestBodyType::BINARY => {
                let body_bytes = to_bytes(req.body_mut()).await;

                match body_bytes {
                    Ok(bytes) => RequestBody::new_binary(bytes),
                    Err(err) => return Err(err),
                }
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
            query: query,
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

    pub(crate) fn from_request_info(req_info: RequestInfo) -> Self {
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

        let query_string = req_info.uri().query().unwrap_or("");

        let queries = url::form_urlencoded::parse(query_string.as_bytes())
            .filter_map(|(key, value)| Some((key.to_string(), value.to_string())))
            .collect::<HashMap<String, String>>();

        let query = QueryParams::from_map(queries);
        let mut params = RouteParams::new();

        if let Some(param_routerify) = req_info.data::<routerify::RouteParams>() {
            println!("Params: {:?}", param_routerify);
            param_routerify.iter().for_each(|(key, value)| {
                params.insert(key.to_string(), value.to_string());
            });
        }

        let mut cookies_map = HashMap::new();
        let cookies = Self::get_cookies_from_req_info(&req_info);

        cookies.iter().for_each(|cookie| {
            let (name, value) = (cookie.name(), cookie.value());
            cookies_map.insert(name.to_string(), value.to_string());
        });

        let ip = req_info
            .headers()
            .get("X-Forwarded-For")
            .and_then(|val| val.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
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

    pub(crate) fn to_hyper_request(&self) -> Result<Request<Body>, Box<dyn std::error::Error>> {
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
                Body::from(text.as_bytes().to_vec())
            }
            RequestBodyContent::FORM(form) => {
                builder.headers_mut().unwrap().insert(
                    hyper::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded".parse()?,
                );
                Body::from(form.to_string().clone())
            }
            RequestBodyContent::BINARY(bytes) => {
                builder.headers_mut().unwrap().insert(
                    hyper::header::CONTENT_TYPE,
                    "application/octet-stream".parse()?,
                );
                Body::from(bytes.clone())
            }
            RequestBodyContent::BinaryWithFields(bytes, _form_data) => {
                // For multipart forms with files, we send the binary data
                // but the form fields are accessible via form_data()
                builder
                    .headers_mut()
                    .unwrap()
                    .insert(hyper::header::CONTENT_TYPE, "multipart/form-data".parse()?);
                Body::from(bytes.clone())
            }
            RequestBodyContent::EMPTY => Body::empty(),
        };

        Ok(builder.body(body)?)
    }
}

#[cfg(test)]
impl HttpRequest {
    pub(crate) fn set_query(&mut self, key: &str, value: &str) {
        self.query.insert(key.to_string(), value.to_string());
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

    pub(crate) fn set_text(&mut self, text: TextData, content_type: RequestBodyType) {
        self.body.content_type = content_type;
        self.body.content = RequestBodyContent::TEXT(text)
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
                existing.insert(key, value);
            }
            _ => {
                let mut form_data = FormData::new();
                form_data.insert(key, value);

                self.body.content = RequestBodyContent::FORM(form_data)
            }
        }
    }

    pub(crate) fn set_content_type(&mut self, content_type: RequestBodyType) {
        self.body.content_type = content_type;
    }

    pub(crate) fn set_binary(&mut self, bytes: Vec<u8>) {
        self.body.content_type = RequestBodyType::BINARY;
        self.body.content = RequestBodyContent::BINARY(bytes.into());
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

pub(crate) fn determine_content_type_request(content_type: &str) -> RequestBodyType {
    match content_type.parse::<Mime>() {
        Ok(mime_type) => match (mime_type.type_(), mime_type.subtype()) {
            (mime::APPLICATION, mime::JSON) => RequestBodyType::JSON,
            (mime::APPLICATION, subtype) if subtype == "x-www-form-urlencoded" => {
                RequestBodyType::FORM
            }
            // Remove the incorrect line that was matching application/form-data as multipart
            (mime::MULTIPART, subtype) if subtype == "form-data" => RequestBodyType::MultipartForm,
            (mime::TEXT, _) => RequestBodyType::TEXT,
            // Handle JSON variants
            (mime::APPLICATION, subtype) if subtype.as_str().ends_with("+json") => {
                RequestBodyType::JSON
            }
            (mime::APPLICATION, subtype)
                if subtype == "xml" || subtype.as_str().ends_with("+xml") =>
            {
                RequestBodyType::TEXT
            }
            _ => RequestBodyType::BINARY,
        },
        Err(_) => RequestBodyType::BINARY, // Fallback for invalid MIME types
    }
}

pub(crate) fn determine_content_type_response(content_type: &str) -> ResponseContentType {
    match content_type.parse::<Mime>() {
        Ok(mime_type) => match (mime_type.type_(), mime_type.subtype()) {
            (mime::APPLICATION, mime::JSON) => ResponseContentType::JSON,
            (mime::TEXT, _) => ResponseContentType::TEXT,
            (mime::APPLICATION, subtype) if subtype.as_str().ends_with("+json") => {
                ResponseContentType::JSON
            }
            (mime::APPLICATION, subtype)
                if subtype == "xml" || subtype.as_str().ends_with("+xml") =>
            {
                ResponseContentType::TEXT
            }
            _ => ResponseContentType::BINARY,
        },
        Err(_) => ResponseContentType::BINARY, // Fallback for invalid MIME types
    }
}

#[cfg(feature = "with-wynd")]
impl AsyncWrite for HttpRequest {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        todo!()
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        todo!()
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        todo!()
    }
}

#[cfg(feature = "with-wynd")]
impl AsyncRead for HttpRequest {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        todo!()
    }
}

// Or remove these impls entirely and introduce a dedicated UpgradedIo
// wrapper over hyper::upgrade::Upgraded that correctly implements AsyncRead/AsyncWrite.
